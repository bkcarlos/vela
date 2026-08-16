mod async_body;
#[cfg(not(target_family = "wasm"))]
pub mod github;
#[cfg(all(not(target_family = "wasm"), feature = "github-download"))]
pub mod github_download;

pub use anyhow::{Result, anyhow};
pub use async_body::{AsyncBody, Inner, Json};
use derive_more::Deref;
pub use http::{self, Method, Request, Response, StatusCode, Uri, request::Builder};
use http::{HeaderName, HeaderValue};

use futures::future::BoxFuture;
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::Arc;
#[cfg(feature = "test-support")]
use std::{any::type_name, fmt};
#[cfg(target_os = "macos")]
use system_configuration::{
    core_foundation::{
        base::CFType,
        dictionary::CFDictionary,
        number::CFNumber,
        string::{CFString, CFStringRef},
    },
    dynamic_store::SCDynamicStoreBuilder,
    sys::schema_definitions::{
        kSCPropNetProxiesHTTPEnable, kSCPropNetProxiesHTTPPort, kSCPropNetProxiesHTTPProxy,
        kSCPropNetProxiesHTTPSEnable, kSCPropNetProxiesHTTPSPort, kSCPropNetProxiesHTTPSProxy,
        kSCPropNetProxiesSOCKSEnable, kSCPropNetProxiesSOCKSPort, kSCPropNetProxiesSOCKSProxy,
    },
};
pub use url::{Host, Url};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RedirectPolicy {
    #[default]
    NoFollow,
    FollowLimit(u32),
    FollowAll,
}
pub struct FollowRedirects(pub bool);

pub trait HttpRequestExt {
    /// Conditionally modify self with the given closure.
    fn when(self, condition: bool, then: impl FnOnce(Self) -> Self) -> Self
    where
        Self: Sized,
    {
        if condition { then(self) } else { self }
    }

    /// Conditionally unwrap and modify self with the given closure, if the given option is Some.
    fn when_some<T>(self, option: Option<T>, then: impl FnOnce(Self, T) -> Self) -> Self
    where
        Self: Sized,
    {
        match option {
            Some(value) => then(self, value),
            None => self,
        }
    }

    /// Whether or not to follow redirects
    fn follow_redirects(self, follow: RedirectPolicy) -> Self;
}

impl HttpRequestExt for http::request::Builder {
    fn follow_redirects(self, follow: RedirectPolicy) -> Self {
        self.extension(follow)
    }
}

/// A set of pre-validated user-supplied HTTP headers.
///
/// Construction (and the per-name validation that goes with it) happens once
/// at settings load time. Cloning is `Arc`-cheap, so providers can hand a copy
/// to each outgoing request without re-parsing or re-allocating.
#[derive(Default, Clone, Debug)]
pub struct CustomHeaders(Arc<[(HeaderName, HeaderValue)]>);

impl CustomHeaders {
    pub fn new(headers: Vec<(HeaderName, HeaderValue)>) -> Self {
        Self(headers.into())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&HeaderName, &HeaderValue)> {
        self.0.iter().map(|(n, v)| (n, v))
    }
}

impl PartialEq for CustomHeaders {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .zip(other.0.iter())
                .all(|(a, b)| a.0 == b.0 && a.1 == b.1)
    }
}

pub trait RequestBuilderExt {
    /// Append every header in `headers` to the request being built.
    fn extra_headers(self, headers: &CustomHeaders) -> Self;
}

impl RequestBuilderExt for http::request::Builder {
    fn extra_headers(mut self, headers: &CustomHeaders) -> Self {
        if headers.is_empty() {
            return self;
        }
        if let Some(map) = self.headers_mut() {
            for (name, value) in headers.iter() {
                map.append(name.clone(), value.clone());
            }
        }
        self
    }
}

pub trait HttpClient: 'static + Send + Sync {
    fn user_agent(&self) -> Option<&HeaderValue>;

    fn proxy(&self) -> Option<Url>;

    fn send(
        &self,
        req: http::Request<AsyncBody>,
    ) -> BoxFuture<'static, anyhow::Result<Response<AsyncBody>>>;

    fn get(
        &self,
        uri: &str,
        body: AsyncBody,
        follow_redirects: bool,
    ) -> BoxFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let request = Builder::new()
            .uri(uri)
            .follow_redirects(if follow_redirects {
                RedirectPolicy::FollowAll
            } else {
                RedirectPolicy::NoFollow
            })
            .body(body);

        match request {
            Ok(request) => self.send(request),
            Err(e) => Box::pin(async move { Err(e.into()) }),
        }
    }

    fn post_json(
        &self,
        uri: &str,
        body: AsyncBody,
    ) -> BoxFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let request = Builder::new()
            .uri(uri)
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(body);

        match request {
            Ok(request) => self.send(request),
            Err(e) => Box::pin(async move { Err(e.into()) }),
        }
    }

    #[cfg(feature = "test-support")]
    fn as_fake(&self) -> &FakeHttpClient {
        panic!("called as_fake on {}", type_name::<Self>())
    }
}

/// An [`HttpClient`] that may have a proxy.
#[derive(Deref)]
pub struct HttpClientWithProxy {
    #[deref]
    client: Arc<dyn HttpClient>,
    proxy: Option<Url>,
}

impl HttpClientWithProxy {
    /// Returns a new [`HttpClientWithProxy`] with the given proxy URL.
    pub fn new(client: Arc<dyn HttpClient>, proxy_url: Option<String>) -> Self {
        let proxy_url = proxy_url
            .and_then(|proxy| proxy.parse().ok())
            .or_else(read_proxy_from_env);

        Self::new_url(client, proxy_url)
    }
    pub fn new_url(client: Arc<dyn HttpClient>, proxy_url: Option<Url>) -> Self {
        Self {
            client,
            proxy: proxy_url,
        }
    }
}

impl HttpClient for HttpClientWithProxy {
    fn send(
        &self,
        req: Request<AsyncBody>,
    ) -> BoxFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        self.client.send(req)
    }

    fn user_agent(&self) -> Option<&HeaderValue> {
        self.client.user_agent()
    }

    fn proxy(&self) -> Option<Url> {
        self.proxy.clone().or_else(|| self.client.proxy())
    }

    #[cfg(feature = "test-support")]
    fn as_fake(&self) -> &FakeHttpClient {
        self.client.as_fake()
    }
}

/// An [`HttpClient`] that has a base URL.
#[derive(Deref)]
pub struct HttpClientWithUrl {
    base_url: Mutex<String>,
    #[deref]
    client: HttpClientWithProxy,
}

impl HttpClientWithUrl {
    /// Returns a new [`HttpClientWithUrl`] with the given base URL.
    pub fn new(
        client: Arc<dyn HttpClient>,
        base_url: impl Into<String>,
        proxy_url: Option<String>,
    ) -> Self {
        let client = HttpClientWithProxy::new(client, proxy_url);

        Self {
            base_url: Mutex::new(base_url.into()),
            client,
        }
    }

    pub fn new_url(
        client: Arc<dyn HttpClient>,
        base_url: impl Into<String>,
        proxy_url: Option<Url>,
    ) -> Self {
        let client = HttpClientWithProxy::new_url(client, proxy_url);

        Self {
            base_url: Mutex::new(base_url.into()),
            client,
        }
    }

    /// Returns the base URL.
    pub fn base_url(&self) -> String {
        self.base_url.lock().clone()
    }

    /// Sets the base URL.
    pub fn set_base_url(&self, base_url: impl Into<String>) {
        let base_url = base_url.into();
        *self.base_url.lock() = base_url;
    }

    /// Builds a URL using the given path.
    pub fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url(), path)
    }

    /// Builds a Vela API URL using the given path.
    pub fn build_vela_api_url(&self, path: &str, query: &[(&str, &str)]) -> Result<Url> {
        let base_url = self.base_url();
        let base_api_url = match base_url.as_ref() {
            "https://vela.dev" => "https://api.vela.dev",
            "https://staging.vela.dev" => "https://api-staging.vela.dev",
            "http://localhost:3000" => "http://localhost:8080",
            other => other,
        };

        Ok(Url::parse_with_params(
            &format!("{}{}", base_api_url, path),
            query,
        )?)
    }

    /// Builds a Vela Cloud URL using the given path.
    pub fn build_vela_cloud_url(&self, path: &str) -> Result<Url> {
        let base_url = self.base_url();
        let base_api_url = match base_url.as_ref() {
            "https://vela.dev" => "https://cloud.vela.dev",
            "https://staging.vela.dev" => "https://cloud.vela.dev",
            "http://localhost:3000" => "http://localhost:8787",
            other => other,
        };

        Ok(Url::parse(&format!("{}{}", base_api_url, path))?)
    }

    /// Builds a Vela Cloud URL using the given path and query params.
    pub fn build_vela_cloud_url_with_query(
        &self,
        path: &str,
        query: impl Serialize,
    ) -> Result<Url> {
        let base_url = self.base_url();
        let base_api_url = match base_url.as_ref() {
            "https://vela.dev" => "https://cloud.vela.dev",
            "https://staging.vela.dev" => "https://cloud.vela.dev",
            "http://localhost:3000" => "http://localhost:8787",
            other => other,
        };
        let query = serde_urlencoded::to_string(&query)?;
        Ok(Url::parse(&format!("{}{}?{}", base_api_url, path, query))?)
    }

    /// Builds a Vela LLM URL using the given path.
    pub fn build_vela_llm_url(&self, path: &str, query: &[(&str, &str)]) -> Result<Url> {
        let base_url = self.base_url();
        let base_api_url = match base_url.as_ref() {
            "https://vela.dev" => "https://cloud.vela.dev",
            "https://staging.vela.dev" => "https://llm-staging.vela.dev",
            "http://localhost:3000" => "http://localhost:8787",
            other => other,
        };

        Ok(Url::parse_with_params(
            &format!("{}{}", base_api_url, path),
            query,
        )?)
    }
}

impl HttpClient for HttpClientWithUrl {
    fn send(
        &self,
        req: Request<AsyncBody>,
    ) -> BoxFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        self.client.send(req)
    }

    fn user_agent(&self) -> Option<&HeaderValue> {
        self.client.user_agent()
    }

    fn proxy(&self) -> Option<Url> {
        self.client.proxy()
    }

    #[cfg(feature = "test-support")]
    fn as_fake(&self) -> &FakeHttpClient {
        self.client.as_fake()
    }
}

pub fn read_proxy_from_env() -> Option<Url> {
    const ENV_VARS: &[&str] = &[
        "ALL_PROXY",
        "all_proxy",
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
    ];

    ENV_VARS
        .iter()
        .find_map(|var| std::env::var(var).ok())
        .and_then(|env| env.parse().ok())
}

/// Read the active macOS proxy directly from SystemConfiguration.
///
/// HTTPS and HTTP entries are HTTP CONNECT proxies even when they carry HTTPS traffic, so their
/// URL scheme is `http`. SOCKS is used only when neither HTTP proxy is enabled.
#[cfg(target_os = "macos")]
pub fn read_proxy_from_system() -> Option<Url> {
    fn setting(
        proxies: &CFDictionary<CFString, CFType>,
        enabled_key: CFStringRef,
        host_key: CFStringRef,
        port_key: CFStringRef,
        scheme: &str,
    ) -> Option<Url> {
        let enabled = proxies
            .find(enabled_key)
            .and_then(|value| value.downcast::<CFNumber>())
            .and_then(|value| value.to_i32())
            == Some(1);
        if !enabled {
            return None;
        }
        let host = proxies
            .find(host_key)
            .and_then(|value| value.downcast::<CFString>())?
            .to_string();
        let port = proxies
            .find(port_key)
            .and_then(|value| value.downcast::<CFNumber>())
            .and_then(|value| value.to_i32());
        let address = match port {
            Some(port) => format!("{scheme}://{host}:{port}"),
            None => format!("{scheme}://{host}"),
        };
        address.parse().ok()
    }

    let store = SCDynamicStoreBuilder::new("Vela").build();
    let proxies = store.get_proxies()?;
    setting(
        &proxies,
        unsafe { kSCPropNetProxiesHTTPSEnable },
        unsafe { kSCPropNetProxiesHTTPSProxy },
        unsafe { kSCPropNetProxiesHTTPSPort },
        "http",
    )
    .or_else(|| {
        setting(
            &proxies,
            unsafe { kSCPropNetProxiesHTTPEnable },
            unsafe { kSCPropNetProxiesHTTPProxy },
            unsafe { kSCPropNetProxiesHTTPPort },
            "http",
        )
    })
    .or_else(|| {
        setting(
            &proxies,
            unsafe { kSCPropNetProxiesSOCKSEnable },
            unsafe { kSCPropNetProxiesSOCKSProxy },
            unsafe { kSCPropNetProxiesSOCKSPort },
            "socks5h",
        )
    })
}

#[cfg(target_os = "windows")]
pub fn read_proxy_from_system() -> Option<Url> {
    const INTERNET_SETTINGS: &str =
        "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings";

    let settings = windows_registry::CURRENT_USER
        .open(INTERNET_SETTINGS)
        .ok()?;
    if settings.get_u32("ProxyEnable").ok()? != 1 {
        return None;
    }
    parse_windows_proxy_server(&settings.get_string("ProxyServer").ok()?)
}

#[cfg(target_os = "windows")]
fn parse_windows_proxy_server(server: &str) -> Option<Url> {
    let mut http_proxy = None;
    let mut https_proxy = None;
    let mut socks_proxy = None;
    let mut unqualified_proxy = None;

    for entry in server
        .split(';')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        if let Some((kind, address)) = entry.split_once('=') {
            match kind.trim().to_ascii_lowercase().as_str() {
                "http" => http_proxy = proxy_url(address, "http"),
                "https" => https_proxy = proxy_url(address, "http"),
                "socks" | "socks5" => socks_proxy = proxy_url(address, "socks5h"),
                _ => {}
            }
        } else {
            unqualified_proxy = proxy_url(entry, "http");
        }
    }

    https_proxy
        .or(http_proxy)
        .or(socks_proxy)
        .or(unqualified_proxy)
}

#[cfg(target_os = "linux")]
pub fn read_proxy_from_system() -> Option<Url> {
    let mode = command_output("gsettings", &["get", "org.gnome.system.proxy", "mode"])
        .map(|mode| mode.trim_matches(['\'', '"']).to_string());
    if mode.as_deref() == Some("manual") {
        return read_gnome_proxy("https", "http")
            .or_else(|| read_gnome_proxy("http", "http"))
            .or_else(|| read_gnome_proxy("socks", "socks5h"));
    }

    let proxy_type = ["kreadconfig6", "kreadconfig5"]
        .iter()
        .find_map(|command| {
            command_output(
                command,
                &["--group", "Proxy Settings", "--key", "ProxyType"],
            )
            .filter(|proxy_type| proxy_type.trim() == "1")
            .map(|_| *command)
        })?;
    for (key, scheme) in [
        ("httpsProxy", "http"),
        ("httpProxy", "http"),
        ("socksProxy", "socks5h"),
    ] {
        let Some(address) =
            command_output(proxy_type, &["--group", "Proxy Settings", "--key", key])
        else {
            continue;
        };
        if let Some(proxy) = proxy_url(address.trim(), scheme) {
            return Some(proxy);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn read_gnome_proxy(kind: &str, scheme: &str) -> Option<Url> {
    let schema = format!("org.gnome.system.proxy.{kind}");
    let host = command_output("gsettings", &["get", &schema, "host"])?;
    let host = host.trim().trim_matches(['\'', '"']);
    if host.is_empty() {
        return None;
    }
    let port = command_output("gsettings", &["get", &schema, "port"])?;
    proxy_url(&format!("{host}:{}", port.trim()), scheme)
}

#[cfg(target_os = "linux")]
#[allow(
    clippy::disallowed_methods,
    reason = "Linux system proxy discovery is synchronous and must complete before constructing the HTTP client"
)]
fn command_output(command: &str, arguments: &[&str]) -> Option<String> {
    let output = std::process::Command::new(command)
        .args(arguments)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn proxy_url(address: &str, default_scheme: &str) -> Option<Url> {
    let address = address.trim();
    if address.is_empty() {
        return None;
    }
    if address.contains("://") {
        address.parse().ok()
    } else {
        format!("{default_scheme}://{address}").parse().ok()
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn read_proxy_from_system() -> Option<Url> {
    None
}

pub fn read_no_proxy_from_env() -> Option<String> {
    const ENV_VARS: &[&str] = &["NO_PROXY", "no_proxy"];

    ENV_VARS.iter().find_map(|var| std::env::var(var).ok())
}

pub struct BlockedHttpClient;

impl BlockedHttpClient {
    pub fn new() -> Self {
        BlockedHttpClient
    }
}

impl HttpClient for BlockedHttpClient {
    fn send(
        &self,
        _req: Request<AsyncBody>,
    ) -> BoxFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        Box::pin(async {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "BlockedHttpClient disallowed request",
            )
            .into())
        })
    }

    fn user_agent(&self) -> Option<&HeaderValue> {
        None
    }

    fn proxy(&self) -> Option<Url> {
        None
    }

    #[cfg(feature = "test-support")]
    fn as_fake(&self) -> &FakeHttpClient {
        panic!("called as_fake on {}", type_name::<Self>())
    }
}

#[cfg(feature = "test-support")]
type FakeHttpHandler = Arc<
    dyn Fn(Request<AsyncBody>) -> BoxFuture<'static, anyhow::Result<Response<AsyncBody>>>
        + Send
        + Sync
        + 'static,
>;

#[cfg(feature = "test-support")]
pub struct FakeHttpClient {
    handler: Mutex<Option<FakeHttpHandler>>,
    user_agent: HeaderValue,
}

#[cfg(feature = "test-support")]
impl FakeHttpClient {
    pub fn create<Fut, F>(handler: F) -> Arc<HttpClientWithUrl>
    where
        Fut: futures::Future<Output = anyhow::Result<Response<AsyncBody>>> + Send + 'static,
        F: Fn(Request<AsyncBody>) -> Fut + Send + Sync + 'static,
    {
        Arc::new(HttpClientWithUrl {
            base_url: Mutex::new("http://test.example".into()),
            client: HttpClientWithProxy {
                client: Arc::new(Self {
                    handler: Mutex::new(Some(Arc::new(move |req| Box::pin(handler(req))))),
                    user_agent: HeaderValue::from_static(type_name::<Self>()),
                }),
                proxy: None,
            },
        })
    }

    pub fn with_404_response() -> Arc<HttpClientWithUrl> {
        log::warn!("Using fake HTTP client with 404 response");
        Self::create(|_| async move {
            Ok(Response::builder()
                .status(404)
                .body(Default::default())
                .unwrap())
        })
    }

    pub fn with_200_response() -> Arc<HttpClientWithUrl> {
        log::warn!("Using fake HTTP client with 200 response");
        Self::create(|_| async move {
            Ok(Response::builder()
                .status(200)
                .body(Default::default())
                .unwrap())
        })
    }

    pub fn replace_handler<Fut, F>(&self, new_handler: F)
    where
        Fut: futures::Future<Output = anyhow::Result<Response<AsyncBody>>> + Send + 'static,
        F: Fn(FakeHttpHandler, Request<AsyncBody>) -> Fut + Send + Sync + 'static,
    {
        let mut handler = self.handler.lock();
        let old_handler = handler.take().unwrap();
        *handler = Some(Arc::new(move |req| {
            Box::pin(new_handler(old_handler.clone(), req))
        }));
    }
}

#[cfg(feature = "test-support")]
impl fmt::Debug for FakeHttpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeHttpClient").finish()
    }
}

#[cfg(feature = "test-support")]
impl HttpClient for FakeHttpClient {
    fn send(
        &self,
        req: Request<AsyncBody>,
    ) -> BoxFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        ((self.handler.lock().as_ref().unwrap())(req)) as _
    }

    fn user_agent(&self) -> Option<&HeaderValue> {
        Some(&self.user_agent)
    }

    fn proxy(&self) -> Option<Url> {
        None
    }

    fn as_fake(&self) -> &FakeHttpClient {
        self
    }
}

use client::VELA_URL_SCHEME;
use gpui::{AsyncApp, actions};

actions!(
    cli,
    [
        /// Registers the vela:// URL scheme handler.
        RegisterVelaScheme
    ]
);

pub async fn register_vela_scheme(cx: &AsyncApp) -> anyhow::Result<()> {
    cx.update(|cx| cx.register_url_scheme(VELA_URL_SCHEME))
        .await
}

use std::{env, fs};
use vela::settings::LspSettings;
use zed_extension_api::{self as vela, LanguageServerId, Result, serde_json::json};

const BINARY_NAME: &str = "vscode-html-language-server";
const SERVER_PATH: &str =
    "node_modules/@vela-industries/vscode-langservers-extracted/bin/vscode-html-language-server";
const PACKAGE_NAME: &str = "@vela-industries/vscode-langservers-extracted";

struct HtmlExtension {
    cached_binary_path: Option<String>,
}

impl HtmlExtension {
    fn server_exists(&self) -> bool {
        fs::metadata(SERVER_PATH).is_ok_and(|stat| stat.is_file())
    }

    fn server_script_path(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        let server_exists = self.server_exists();
        if self.cached_binary_path.is_some() && server_exists {
            return Ok(SERVER_PATH.to_string());
        }

        vela::set_language_server_installation_status(
            language_server_id,
            &vela::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let version = vela::npm_package_latest_version(PACKAGE_NAME)?;

        if !server_exists
            || vela::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version)
        {
            vela::set_language_server_installation_status(
                language_server_id,
                &vela::LanguageServerInstallationStatus::Downloading,
            );
            let result = vela::npm_install_package(PACKAGE_NAME, &version);
            match result {
                Ok(()) => {
                    if !self.server_exists() {
                        Err(format!(
                            "installed package '{PACKAGE_NAME}' did not contain expected path '{SERVER_PATH}'",
                        ))?;
                    }
                }
                Err(error) => {
                    if !self.server_exists() {
                        Err(error)?;
                    }
                }
            }
        }
        Ok(SERVER_PATH.to_string())
    }
}

impl vela::Extension for HtmlExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &vela::Worktree,
    ) -> Result<vela::Command> {
        let server_path = if let Some(path) = worktree.which(BINARY_NAME) {
            return Ok(vela::Command {
                command: path,
                args: vec!["--stdio".to_string()],
                env: Default::default(),
            });
        } else {
            let server_path = self.server_script_path(language_server_id)?;
            env::current_dir()
                .unwrap()
                .join(&server_path)
                .to_string_lossy()
                .to_string()
        };
        self.cached_binary_path = Some(server_path.clone());

        Ok(vela::Command {
            command: vela::node_binary_path()?,
            args: vec![server_path, "--stdio".to_string()],
            env: Default::default(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &vela::Worktree,
    ) -> Result<Option<vela::serde_json::Value>> {
        LspSettings::for_worktree(server_id.as_ref(), worktree)
            .map(|lsp_settings| lsp_settings.settings)
    }

    fn language_server_initialization_options(
        &mut self,
        _server_id: &LanguageServerId,
        _worktree: &vela::Worktree,
    ) -> Result<Option<vela::serde_json::Value>> {
        let initialization_options = json!({"provideFormatter": true });
        Ok(Some(initialization_options))
    }
}

vela::register_extension!(HtmlExtension);

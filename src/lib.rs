mod installer;

use std::collections::BTreeMap;
use zed_extension_api::{
    self as zed, settings::LspSettings, Command, Extension, LanguageServerId, Worktree,
};

const RELEASE_REPOSITORY: Option<&str> = Some("Nazuha26/zed-cpp-docs");

#[derive(serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Settings {
    release_repository: Option<String>,
    clangd_path: Option<String>,
    data_dir: Option<String>,
    descriptions_path: Option<String>,
    auto_setup: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            release_repository: None,
            clangd_path: None,
            data_dir: None,
            descriptions_path: None,
            auto_setup: true,
        }
    }
}

#[derive(Default)]
struct CppDocs {
    installer: installer::Installer,
}

impl CppDocs {
    fn command(&mut self, id: &LanguageServerId, worktree: &Worktree) -> zed::Result<Command> {
        let lsp = LspSettings::for_worktree(id.as_ref(), worktree)?;
        // Explicit commands remain authoritative, including their arguments and environment.
        if let Some(binary) = &lsp.binary {
            if let Some(path) = &binary.path {
                return Ok(Command {
                    command: path.clone(),
                    args: binary.arguments.clone().unwrap_or_default(),
                    env: binary.env.clone().unwrap_or_default().into_iter().collect(),
                });
            }
        }
        let settings: Settings = match lsp.settings {
            Some(value) => zed::serde_json::from_value(value)
                .map_err(|error| format!("C/C++ Docs: invalid extension settings: {error}"))?,
            None => Settings::default(),
        };
        let clangd = settings.clangd_path.or_else(|| worktree.which("clangd")).ok_or(
            "C/C++ Docs: install clangd in this environment or set lsp.cpp-docs.settings.clangd_path.",
        )?;
        let command = if let Some(path) = worktree.which("zed-cpp-docs-proxy") {
            path
        } else {
            let repository = settings.release_repository.as_deref().or(RELEASE_REPOSITORY).ok_or(
                "C/C++ Docs: the release repository has not been configured yet. Set lsp.cpp-docs.settings.release_repository to owner/repo after publishing release binaries, or install zed-cpp-docs-proxy in PATH.",
            )?;
            self.installer.binary(repository, id)?
        };
        let mut args = vec!["--clangd".into(), clangd];
        if let Some(data) = settings.data_dir {
            args.extend(["--data-dir".into(), data]);
        }
        if let Some(descriptions) = settings.descriptions_path {
            args.extend(["--descriptions".into(), descriptions]);
        }
        let mut env: BTreeMap<_, _> = worktree.shell_env().into_iter().collect();
        if let Some(binary) = lsp.binary {
            if let Some(arguments) = binary.arguments {
                args.extend(arguments);
            }
            if let Some(overrides) = binary.env {
                env.extend(overrides);
            }
        }
        let env: Vec<_> = env.into_iter().collect();
        if settings.auto_setup && !args.iter().any(|arg| arg == "--fixtures") {
            zed::set_language_server_installation_status(
                id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let setup = zed::process::Command::new(&command)
                .arg("--ensure-cppreference")
                .args(args.clone())
                .envs(env.clone())
                .output();
            match setup {
                Ok(output) if output.status == Some(0) => {}
                Ok(output) => eprintln!(
                    "C/C++ Docs: documentation setup failed; starting the proxy with its normal offline fallback: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
                Err(error) => eprintln!("C/C++ Docs: documentation setup could not run: {error}. Starting the proxy with its normal offline fallback."),
            }
        }
        zed::set_language_server_installation_status(
            id,
            &zed::LanguageServerInstallationStatus::None,
        );
        Ok(Command { command, args, env })
    }
}

impl Extension for CppDocs {
    fn new() -> Self {
        Self::default()
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &Worktree,
    ) -> zed::Result<Command> {
        let result = self.command(id, worktree);
        if let Err(error) = &result {
            zed::set_language_server_installation_status(
                id,
                &zed::LanguageServerInstallationStatus::Failed(error.clone()),
            );
        }
        result
    }
}

zed::register_extension!(CppDocs);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_setup_can_be_disabled_without_changing_the_release_settings() {
        let settings: Settings =
            zed::serde_json::from_str(r#"{"auto_setup":false,"release_repository":"owner/repo"}"#)
                .unwrap();
        assert!(!settings.auto_setup);
        assert_eq!(settings.release_repository.as_deref(), Some("owner/repo"));
        let defaults: Settings = zed::serde_json::from_str("{}").unwrap();
        assert!(defaults.auto_setup);
        assert!(defaults.release_repository.is_none());
    }
}

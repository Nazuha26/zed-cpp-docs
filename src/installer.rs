use std::fs;
use std::path::{Component, Path, PathBuf};
use zed_extension_api::{self as zed, Architecture, DownloadedFileType, LanguageServerId, Os};

#[derive(Default)]
pub struct Installer {
    cached: Option<(String, String)>,
}

fn safe_component(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-+".contains(&c))
}

fn repository_directory(repository: &str) -> Result<String, String> {
    let (owner, name) = repository
        .split_once('/')
        .ok_or("Expected a GitHub repository in owner/repo form")?;
    if !safe_component(owner) || !safe_component(name) {
        return Err("Expected a GitHub repository in owner/repo form".into());
    }
    // Separate path components avoid collisions between names containing hyphens.
    Ok(format!("servers/{owner}/{name}"))
}

fn platform(os: Os, architecture: Architecture) -> Result<(&'static str, &'static str), String> {
    let target = match (os, architecture) {
        (Os::Windows, Architecture::X8664) => "x86_64-pc-windows-msvc",
        (Os::Windows, Architecture::Aarch64) => "aarch64-pc-windows-msvc",
        (Os::Linux, Architecture::X8664) => "x86_64-unknown-linux-musl",
        (Os::Linux, Architecture::Aarch64) => "aarch64-unknown-linux-musl",
        (Os::Mac, Architecture::X8664) => "x86_64-apple-darwin",
        (Os::Mac, Architecture::Aarch64) => "aarch64-apple-darwin",
        _ => return Err("No release target for this platform; install the proxy in PATH".into()),
    };
    Ok((
        target,
        if os == Os::Windows {
            "zed-cpp-docs-proxy.exe"
        } else {
            "zed-cpp-docs-proxy"
        },
    ))
}

fn cached_binary(root: &Path, executable: &str) -> Option<PathBuf> {
    let relative = fs::read_to_string(root.join("current")).ok()?;
    let relative = Path::new(relative.trim());
    let components: Vec<_> = relative.components().collect();
    if components.len() != 3
        || components
            .iter()
            .any(|part| !matches!(part, Component::Normal(_)))
        || components[0].as_os_str() != "versions"
        || components[2].as_os_str() != executable
    {
        return None;
    }
    let path = root.join(relative);
    path.is_file().then_some(path)
}

impl Installer {
    pub fn binary(&mut self, repository: &str, id: &LanguageServerId) -> Result<String, String> {
        let (os, architecture) = zed::current_platform();
        let (target, executable) = platform(os, architecture)?;
        let root = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(repository_directory(repository)?)
            .join(target);
        let key = root.to_string_lossy().into_owned();
        if let Some((cached_key, binary)) = &self.cached {
            if cached_key == &key && Path::new(binary).is_file() {
                return Ok(binary.clone());
            }
        }
        let fallback = cached_binary(&root, executable);
        zed::set_language_server_installation_status(
            id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let result = self.download(repository, &root, target, executable, os, id);
        let binary = match result {
            Ok(binary) => binary,
            Err(error) => match fallback {
                Some(binary) => {
                    eprintln!("C/C++ Docs: update unavailable ({error}); using the cached server.");
                    binary
                }
                None => return Err(error),
            },
        };
        let binary = binary.to_string_lossy().into_owned();
        self.cached = Some((key, binary.clone()));
        Ok(binary)
    }

    fn download(
        &self,
        repository: &str,
        root: &Path,
        target: &str,
        executable: &str,
        os: Os,
        id: &LanguageServerId,
    ) -> Result<PathBuf, String> {
        let release = zed::latest_github_release(
            repository,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;
        if !safe_component(&release.version) {
            return Err("The release tag cannot be used as a local installation directory".into());
        }
        let directory = root.join("versions").join(&release.version);
        let binary = directory.join(executable);
        let extension = if os == Os::Windows { "zip" } else { "tar.gz" };
        let asset_name = format!("zed-cpp-docs-proxy-{target}.{extension}");
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("Release {} has no {asset_name} asset", release.version))?;
        if !binary.is_file() {
            zed::set_language_server_installation_status(
                id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            fs::create_dir_all(root.join("staging")).map_err(|e| e.to_string())?;
            fs::create_dir_all(root.join("versions")).map_err(|e| e.to_string())?;
            let staging = root.join("staging").join(&release.version);
            if staging.exists() {
                fs::remove_dir_all(&staging).map_err(|e| e.to_string())?;
            }
            let file_type = if os == Os::Windows {
                DownloadedFileType::Zip
            } else {
                DownloadedFileType::GzipTar
            };
            zed::download_file(
                &asset.download_url,
                staging.to_str().ok_or("Non-UTF-8 install path")?,
                file_type,
            )?;
            let staged_binary = staging.join(executable);
            if !staged_binary.is_file() {
                let _ = fs::remove_dir_all(&staging);
                return Err(format!(
                    "The release archive does not contain {executable} at its root"
                ));
            }
            if os != Os::Windows {
                zed::make_file_executable(staged_binary.to_str().ok_or("Non-UTF-8 binary path")?)?;
            }
            if directory.exists() {
                fs::remove_dir_all(&directory).map_err(|e| e.to_string())?;
            }
            fs::rename(&staging, &directory).map_err(|e| e.to_string())?;
        }
        fs::write(
            root.join("current.new"),
            format!("versions/{}/{executable}", release.version),
        )
        .map_err(|e| e.to_string())?;
        fs::rename(root.join("current.new"), root.join("current")).map_err(|e| e.to_string())?;
        Ok(binary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_and_release_names_cannot_escape_the_install_directory() {
        for repository in [
            "",
            "owner",
            "../repo",
            "owner/../repo",
            "https://github.com/owner/repo",
        ] {
            assert!(repository_directory(repository).is_err(), "{repository}");
        }
        assert_ne!(
            repository_directory("a-b/c").unwrap(),
            repository_directory("a/b-c").unwrap()
        );
        for tag in ["", "..", "../v1", "v1/../../elsewhere"] {
            assert!(!safe_component(tag));
        }
        assert!(safe_component("v0.1.0"));
    }

    #[test]
    fn offline_cache_ignores_partial_downloads_and_unsafe_markers() {
        let root =
            std::env::temp_dir().join(format!("cpp-docs-release-cache-{}", std::process::id()));
        fs::create_dir_all(root.join("staging/v1")).unwrap();
        fs::write(root.join("staging/v1/proxy"), b"partial").unwrap();
        fs::write(root.join("current"), "staging/v1/proxy").unwrap();
        assert!(cached_binary(&root, "proxy").is_none());
        fs::write(root.join("current"), "../outside").unwrap();
        assert!(cached_binary(&root, "proxy").is_none());
        fs::create_dir_all(root.join("versions/v1")).unwrap();
        fs::write(root.join("versions/v1/proxy"), b"installed").unwrap();
        fs::write(root.join("current"), "versions/v1/proxy").unwrap();
        assert_eq!(
            cached_binary(&root, "proxy").unwrap(),
            root.join("versions/v1/proxy")
        );
        assert!(cached_binary(&root, "different-proxy").is_none());
        fs::remove_dir_all(root).unwrap();
    }
}

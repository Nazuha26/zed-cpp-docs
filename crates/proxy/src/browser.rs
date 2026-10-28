use crate::docs::{self, Entry};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub struct Browser {
    base: String,
}

impl Browser {
    pub fn start(entries: Vec<Entry>, data: PathBuf) -> std::io::Result<Self> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
        let address = listener.local_addr()?;
        let token = format!(
            "{:016x}{:016x}",
            RandomState::new().build_hasher().finish(),
            RandomState::new().build_hasher().finish()
        );
        let prefix = format!("/{token}/");
        let base = format!("http://127.0.0.1:{}/{}", address.port(), token);
        std::thread::spawn(move || {
            for connection in listener.incoming() {
                let Ok(mut stream) = connection else { break };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
                let mut buffer = [0; 8192];
                let Ok(count) = stream.read(&mut buffer) else {
                    continue;
                };
                let request = String::from_utf8_lossy(&buffer[..count]);
                let mut words = request.lines().next().unwrap_or("").split_whitespace();
                let method = words.next().unwrap_or("");
                let route = words.next().unwrap_or("");
                let suffix = route.strip_prefix(&prefix).unwrap_or("");
                let entry = suffix
                    .strip_prefix("fixture/")
                    .and_then(|index| index.parse::<usize>().ok())
                    .and_then(|index| entries.get(index));
                let mut content_type = "text/html; charset=utf-8";
                let (status, body): (&str, Vec<u8>) = if method == "GET" {
                    if let Some(entry) = entry {
                        ("200 OK", format!("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>{}</title><style>body{{max-width:850px;margin:3rem auto;padding:0 1rem;font:18px/1.6 system-ui;color:#222;background:#fafafa}}pre{{white-space:pre-wrap;background:#eee;padding:1rem;border-radius:8px}}</style><h1>{}</h1><p>This is a local prototype fixture, not the full upstream documentation.</p><pre>{}</pre></html>", escape(&entry.symbol), escape(&entry.symbol), escape(&docs::markdown(entry, None))).into_bytes())
                    } else if let Some((bytes, mime)) = suffix
                        .strip_prefix("files/")
                        .and_then(|path| local_file(&data, path))
                    {
                        content_type = mime;
                        ("200 OK", bytes)
                    } else {
                        ("404 Not Found", b"Not found".to_vec())
                    }
                } else {
                    ("405 Method Not Allowed", b"GET required".to_vec())
                };
                let header = format!("HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'none'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; frame-ancestors 'none'\r\n\r\n", body.len());
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        Ok(Self { base })
    }

    pub fn page(&self, index: usize, entry: &Entry) -> String {
        if entry.fixture {
            format!("{}/fixture/{index}", self.base)
        } else {
            let (path, fragment) = entry
                .html_path
                .split_once('#')
                .unwrap_or((&entry.html_path, ""));
            let path = path
                .split('/')
                .map(|part| {
                    percent_encoding::utf8_percent_encode(part, percent_encoding::NON_ALPHANUMERIC)
                        .to_string()
                })
                .collect::<Vec<_>>()
                .join("/");
            let mut url = format!("{}/files/{path}", self.base);
            if !fragment.is_empty() {
                url.push('#');
                url.push_str(
                    &percent_encoding::utf8_percent_encode(
                        fragment,
                        percent_encoding::NON_ALPHANUMERIC,
                    )
                    .to_string(),
                );
            }
            url
        }
    }
}

fn local_file(data: &Path, path: &str) -> Option<(Vec<u8>, &'static str)> {
    let decoded = percent_encoding::percent_decode_str(path.split('?').next()?)
        .decode_utf8()
        .ok()?;
    let relative = Path::new(decoded.as_ref());
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return None;
    }
    let root = data.canonicalize().ok()?;
    let file = root.join(relative).canonicalize().ok()?;
    if !file.starts_with(&root) || !file.is_file() || file.metadata().ok()?.len() > 16 * 1024 * 1024
    {
        return None;
    }
    let mime = match file.extension()?.to_str()? {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "jpg" | "jpeg" => "image/jpeg",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => return None,
    };
    let bytes = std::fs::read(file).ok()?;
    Some((bytes, mime))
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_urls_escape_operator_names_and_preserve_relative_asset_paths() {
        let mut entry = docs::fixtures().remove(0);
        entry.fixture = false;
        entry.html_path = "book/reference/operator().html#operator[]".into();
        let browser = Browser {
            base: "http://127.0.0.1:1/token".into(),
        };
        let url = browser.page(0, &entry);
        assert!(url.ends_with("book/reference/operator%28%29%2Ehtml#operator%5B%5D"));
        assert_eq!(
            url::Url::parse(&url)
                .unwrap()
                .join("../common/style.css")
                .unwrap()
                .path(),
            "/token/files/book/common/style.css"
        );
    }

    #[test]
    fn local_viewer_rejects_traversal_and_non_document_files() {
        let directory =
            std::env::temp_dir().join(format!("cpp-docs-viewer-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("page.html"), "<p>local</p>").unwrap();
        std::fs::write(directory.join("index.json"), "private index").unwrap();
        assert!(local_file(&directory, "page.html").is_some());
        assert!(local_file(&directory, "index.json").is_none());
        assert!(local_file(&directory, "../page.html").is_none());
        assert!(local_file(&directory, "%2e%2e/page.html").is_none());
        std::fs::remove_file(directory.join("page.html")).unwrap();
        std::fs::remove_file(directory.join("index.json")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}

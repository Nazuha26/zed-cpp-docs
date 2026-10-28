mod browser;
mod docs;
mod notes;
mod store;
mod trace;
mod wire;

use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

enum Event {
    Client(io::Result<Option<Value>>),
    Server(io::Result<Option<Value>>),
}
enum Reply {
    Forward(Value),
    Hover(u64),
    Symbols(u64),
    Declaration(u64),
}

struct Hover {
    original_id: Value,
    uri: String,
    version: Option<Value>,
    original: Option<Value>,
    symbols: Option<Value>,
    declaration: Option<Value>,
    deadline: Instant,
}

struct Router {
    sequence: u64,
    replies: HashMap<String, Reply>,
    hovers: HashMap<u64, Hover>,
    versions: HashMap<String, Value>,
    roots: Vec<url::Url>,
    entries: Vec<docs::Entry>,
    index: HashMap<String, Vec<usize>>,
    markdown_cache: HashMap<usize, String>,
    notices: Vec<String>,
    browser: Option<browser::Browser>,
    symbol_info_supported: bool,
}

impl Router {
    fn new(fixtures: bool, data: &std::path::Path, descriptions: &std::path::Path) -> Self {
        let mut load_error = None;
        let mut entries = if fixtures {
            docs::fixtures()
        } else {
            match store::load(data) {
                Ok(entries) => entries,
                Err(error) => {
                    eprintln!("C/C++ Docs: documentation disabled: {error}. Rebuild with --setup-cppreference. clangd remains active.");
                    load_error = Some(error.to_string());
                    Vec::new()
                }
            }
        };
        trace::event(
            "index",
            json!({"data_dir": data, "cwd": std::env::current_dir().ok(), "executable": std::env::current_exe().ok(), "records": entries.len(), "error": load_error, "localappdata": std::env::var("LOCALAPPDATA").ok(), "home": std::env::var("HOME").ok()}),
        );
        if entries.is_empty() && !fixtures {
            eprintln!("C/C++ Docs: no local index in {}. Run zed-cpp-docs-proxy --setup-cppreference in this environment. clangd remains active.", data.display());
        }
        let mut notices = Vec::new();
        notes::apply(&mut entries, &notes::bundled(), false);
        match notes::load(descriptions) {
            Ok(user) => notes::apply(&mut entries, &user, true),
            Err(error) => {
                let message = format!("C/C++ Docs: could not read user descriptions from {}: {error}. Built-in and cppreference descriptions remain active.", descriptions.display());
                eprintln!("{message}");
                notices.push(message);
            }
        }
        let browser = if entries.is_empty() {
            None
        } else {
            match browser::Browser::start(entries.clone(), data.to_path_buf()) {
                Ok(browser) => Some(browser),
                Err(error) => {
                    eprintln!("C/C++ Docs: local page viewer unavailable: {error}");
                    None
                }
            }
        };
        if let Some(error) = load_error {
            notices.push(format!("C/C++ Docs: could not read the local index: {error}. Ordinary clangd features remain active."));
        }
        if !fixtures && entries.is_empty() {
            notices.push(format!("C/C++ Docs: local documentation is unavailable in {}. Run zed-cpp-docs-proxy --setup-cppreference. Ordinary clangd features remain active.", data.display()));
        }
        let index = docs::index(&entries);
        Self {
            sequence: 0,
            replies: HashMap::new(),
            hovers: HashMap::new(),
            versions: HashMap::new(),
            roots: Vec::new(),
            entries,
            index,
            markdown_cache: HashMap::new(),
            notices,
            browser,
            symbol_info_supported: true,
        }
    }

    fn request(
        &mut self,
        mut message: Value,
        reply: Reply,
        server: &mut impl io::Write,
    ) -> io::Result<()> {
        self.sequence += 1;
        let id = format!("zed-cpp-docs:{}", self.sequence);
        self.replies.insert(id.clone(), reply);
        message["id"] = json!(id);
        wire::write(server, &message)
    }

    fn client(&mut self, message: Value, server: &mut impl io::Write) -> io::Result<bool> {
        let method = message
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        if method == "initialize" {
            self.roots = params
                .get("workspaceFolders")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|folder| folder.get("uri").and_then(Value::as_str))
                .filter_map(|uri| url::Url::parse(uri).ok())
                .collect();
            if let Some(root) = params
                .get("rootUri")
                .and_then(Value::as_str)
                .and_then(|uri| url::Url::parse(uri).ok())
            {
                self.roots.push(root);
            }
        }
        if matches!(
            method.as_str(),
            "textDocument/didOpen" | "textDocument/didChange" | "textDocument/didClose"
        ) {
            if let Some(uri) = params.pointer("/textDocument/uri").and_then(Value::as_str) {
                self.versions.insert(
                    uri.into(),
                    params
                        .pointer("/textDocument/version")
                        .cloned()
                        .unwrap_or(Value::Null),
                );
            }
        }
        if method == "$/cancelRequest" {
            let original_id = &params["id"];
            let ids: Vec<_> = self
                .replies
                .iter()
                .filter_map(|(id, reply)| {
                    let original = match reply {
                        Reply::Forward(original) => Some(original),
                        Reply::Hover(token) | Reply::Symbols(token) | Reply::Declaration(token) => {
                            self.hovers.get(token).map(|hover| &hover.original_id)
                        }
                    };
                    (original == Some(original_id)).then_some(id.clone())
                })
                .collect();
            for id in ids {
                wire::write(
                    server,
                    &json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":id}}),
                )?;
            }
            // Also cancel the local enrichment; clangd remains responsible for the client response.
            for hover in self
                .hovers
                .values_mut()
                .filter(|hover| hover.original_id == *original_id)
            {
                hover.symbols = Some(Value::Null);
                hover.declaration = Some(Value::Null);
            }
            return Ok(false);
        }
        if let Some(original_id) = message.get("id").cloned().filter(|_| !method.is_empty()) {
            if method == "textDocument/hover" && !self.entries.is_empty() {
                let token = self.sequence + 1;
                let uri = params
                    .pointer("/textDocument/uri")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                self.hovers.insert(
                    token,
                    Hover {
                        original_id,
                        version: self.versions.get(&uri).cloned(),
                        uri,
                        original: None,
                        symbols: if self.symbol_info_supported {
                            None
                        } else {
                            Some(Value::Null)
                        },
                        declaration: None,
                        deadline: Instant::now() + Duration::from_millis(800),
                    },
                );
                self.request(message, Reply::Hover(token), server)?;
                if self.symbol_info_supported {
                    self.request(
                        json!({"jsonrpc":"2.0","method":"textDocument/symbolInfo","params":params}),
                        Reply::Symbols(token),
                        server,
                    )?;
                }
                self.request(
                    json!({"jsonrpc":"2.0","method":"textDocument/declaration","params":params}),
                    Reply::Declaration(token),
                    server,
                )?;
            } else {
                self.request(message, Reply::Forward(original_id), server)?;
            }
        } else {
            // Includes client responses to requests initiated by clangd.
            wire::write(server, &message)?;
        }
        Ok(method == "exit")
    }

    fn server(&mut self, mut message: Value, client: &mut impl io::Write) -> io::Result<()> {
        if message.get("method").is_some() {
            return wire::write(client, &message);
        }
        let reply = message
            .get("id")
            .and_then(Value::as_str)
            .and_then(|id| self.replies.remove(id));
        match reply {
            Some(Reply::Forward(original_id)) => {
                message["id"] = original_id;
                wire::write(client, &message)?;
            }
            Some(Reply::Hover(token)) => {
                if let Some(hover) = self.hovers.get_mut(&token) {
                    message["id"] = hover.original_id.clone();
                    hover.original = Some(message);
                    hover.deadline = Instant::now() + Duration::from_millis(300);
                }
            }
            Some(Reply::Symbols(token)) => {
                if message.pointer("/error/code").and_then(Value::as_i64) == Some(-32601) {
                    self.symbol_info_supported = false;
                }
                if let Some(hover) = self.hovers.get_mut(&token) {
                    hover.symbols = Some(message);
                }
            }
            Some(Reply::Declaration(token)) => {
                if let Some(hover) = self.hovers.get_mut(&token) {
                    hover.declaration = Some(message);
                }
            }
            // Late replies to expired internal requests must never reach Zed.
            None => {}
        }
        Ok(())
    }

    fn complete(
        &mut self,
        client: &mut impl io::Write,
        server: &mut impl io::Write,
    ) -> io::Result<()> {
        let ready: Vec<_> = self
            .hovers
            .iter()
            .filter(|(_, hover)| {
                hover.original.is_some()
                    && ((hover.symbols.is_some() && hover.declaration.is_some())
                        || Instant::now() >= hover.deadline)
            })
            .map(|(token, _)| *token)
            .collect();
        for token in ready {
            let hover = self.hovers.remove(&token).unwrap();
            let mut message = hover.original.unwrap();
            let current = self.versions.get(&hover.uri).cloned();
            trace::event(
                "hover",
                json!({"captured_version": hover.version, "current_version": current, "symbols": hover.symbols, "declaration": hover.declaration, "records": self.entries.len()}),
            );
            if current == hover.version
                && message.get("error").is_none()
                && !message["result"].is_null()
            {
                if let Some(entry) = docs::resolve(
                    &self.entries,
                    hover.symbols.as_ref().unwrap_or(&Value::Null),
                    hover.declaration.as_ref().unwrap_or(&Value::Null),
                    &self.roots,
                    &self.index,
                ) {
                    let index = self
                        .entries
                        .iter()
                        .position(|candidate| std::ptr::eq(candidate, entry))
                        .unwrap();
                    let markdown = if let Some(markdown) = self.markdown_cache.get(&index) {
                        markdown.clone()
                    } else {
                        let page = self
                            .browser
                            .as_ref()
                            .map(|browser| browser.page(index, entry));
                        let markdown = docs::markdown(entry, page.as_deref());
                        self.markdown_cache.insert(index, markdown.clone());
                        markdown
                    };
                    message["result"]["contents"] = json!({"kind":"markdown","value":markdown});
                }
            }
            wire::write(client, &message)?;
            let expired: Vec<_> = self
                .replies
                .iter()
                .filter_map(|(id, reply)| match reply {
                    Reply::Symbols(value) | Reply::Declaration(value) if *value == token => {
                        Some(id.clone())
                    }
                    _ => None,
                })
                .collect();
            for id in expired {
                self.replies.remove(&id);
                wire::write(
                    server,
                    &json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":id}}),
                )?;
            }
        }
        Ok(())
    }
}

struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        for _ in 0..20 {
            if matches!(self.0.try_wait(), Ok(Some(_))) {
                return;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn reader(input: impl io::Read + Send + 'static, sender: Sender<Event>, client: bool) {
    std::thread::spawn(move || {
        let mut input = BufReader::new(input);
        loop {
            let message = wire::read(&mut input);
            let finished = !matches!(&message, Ok(Some(_)));
            let event = if client {
                Event::Client(message)
            } else {
                Event::Server(message)
            };
            if sender.send(event).is_err() || finished {
                break;
            }
        }
    });
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut clangd = "clangd".to_string();
    let mut fixtures = false;
    let mut doctor = false;
    let mut data = store::default_dir();
    let mut descriptions = notes::default_path();
    let mut setup = false;
    let mut ensure = false;
    let mut import_archive = None;
    let mut arguments = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--clangd" => clangd = args.next().ok_or("--clangd requires a path")?,
            "--fixtures" => fixtures = true,
            "--doctor" => doctor = true,
            "--data-dir" => data = args.next().ok_or("--data-dir requires a path")?.into(),
            "--descriptions" => {
                descriptions = args.next().ok_or("--descriptions requires a path")?.into()
            }
            "--setup-cppreference" => setup = true,
            "--ensure-cppreference" => ensure = true,
            "--import-cppreference" => {
                import_archive = Some(std::path::PathBuf::from(
                    args.next()
                        .ok_or("--import-cppreference requires an archive path")?,
                ))
            }
            "--" => {
                arguments.extend(args);
                break;
            }
            "--help" => {
                eprintln!("zed-cpp-docs-proxy [--clangd PATH] [--data-dir PATH] [--descriptions PATH] [--fixtures] [--doctor] [-- CLANGD_ARGS]\nSetup only: --setup-cppreference | --ensure-cppreference | --import-cppreference ARCHIVE.\n--ensure-cppreference installs documentation only when the index is missing or empty.\nDescriptions are loaded at server startup: user fields > bundled compact fields > cppreference. Omitted fields inherit; empty strings clear fields.\nDefault user descriptions: {}\nNormal LSP operation never downloads documentation. --fixtures is an explicit prototype test mode.", notes::default_path().display());
                return Ok(());
            }
            _ => arguments.push(arg),
        }
    }
    if setup || import_archive.is_some() {
        if setup {
            import_archive = Some(store::download(&data)?);
        }
        if let Some(archive) = import_archive {
            store::import_cppreference(&data, &archive)?;
        }
        return Ok(());
    }
    if ensure {
        store::ensure(&data)?;
        return Ok(());
    }
    if doctor {
        let version = Command::new(&clangd).arg("--version").output()?;
        if !version.status.success() {
            return Err("clangd --version failed".into());
        }
        let entries = store::load(&data)?;
        let user = notes::load(&descriptions)?;
        println!(
            "{}",
            json!({"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"clangd":String::from_utf8_lossy(&version.stdout),"cppreference_symbols":entries.len(),"data_dir":data,"index_path":data.join("index.json"),"indexed_symbols":entries.len(),"mode":"local index","descriptions_path":descriptions,"bundled_descriptions":notes::bundled().len(),"user_descriptions":user.len()})
        );
        return Ok(());
    }
    let child = Command::new(&clangd).args(arguments).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn()
        .map_err(|error| format!("C/C++ Docs: cannot start clangd '{clangd}' in the LSP environment: {error}. Install clangd or restore the ordinary clangd server in Zed."))?;
    let mut child = Process(child);
    let mut server = child.0.stdin.take().unwrap();
    let stdout = child.0.stdout.take().unwrap();
    let (sender, events): (Sender<Event>, Receiver<Event>) = mpsc::channel();
    reader(io::stdin(), sender.clone(), true);
    reader(stdout, sender, false);
    let mut router = Router::new(fixtures, &data, &descriptions);
    let mut client = io::stdout();
    loop {
        match events.recv_timeout(Duration::from_millis(20)) {
            Ok(Event::Client(Ok(Some(message)))) => {
                let initialized =
                    message.get("method").and_then(Value::as_str) == Some("initialized");
                if router.client(message, &mut server)? {
                    break;
                }
                if initialized {
                    for notice in router.notices.drain(..) {
                        wire::write(
                            &mut client,
                            &json!({"jsonrpc":"2.0","method":"window/showMessage","params":{"type":2,"message":notice}}),
                        )?;
                    }
                }
            }
            Ok(Event::Server(Ok(Some(message)))) => router.server(message, &mut client)?,
            Ok(Event::Client(Ok(None))) => break,
            Ok(Event::Server(Ok(None))) => {
                return Err("clangd closed its output unexpectedly".into())
            }
            Ok(Event::Client(Err(error)) | Event::Server(Err(error))) => return Err(error.into()),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        router.complete(&mut client, &mut server)?;
    }
    drop(server);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn frames(bytes: &[u8]) -> Vec<Value> {
        let mut reader = Cursor::new(bytes);
        let mut values = Vec::new();
        while let Some(value) = wire::read(&mut reader).unwrap() {
            values.push(value);
        }
        values
    }

    #[test]
    fn server_requests_and_client_ids_have_independent_namespaces() {
        let mut router = Router::new(
            false,
            std::path::Path::new("missing-test-index"),
            std::path::Path::new("missing-test-descriptions.json"),
        );
        let mut server = Vec::new();
        let mut client = Vec::new();
        let original_id = json!("zed-cpp-docs:1");
        router
            .client(
                json!({"jsonrpc":"2.0","id":original_id,"method":"initialize","params":{}}),
                &mut server,
            )
            .unwrap();
        let wire_id = frames(&server)[0]["id"].clone();
        let request = json!({"jsonrpc":"2.0","id":wire_id,"method":"workspace/configuration","params":{"items":[]}});
        router.server(request.clone(), &mut client).unwrap();
        assert_eq!(frames(&client), vec![request]);
        assert_eq!(router.replies.len(), 1);
        server.clear();
        router
            .client(
                json!({"jsonrpc":"2.0","id":wire_id,"result":[]}),
                &mut server,
            )
            .unwrap();
        assert_eq!(frames(&server)[0]["id"], wire_id);
        client.clear();
        router
            .server(
                json!({"jsonrpc":"2.0","id":wire_id,"result":{"capabilities":{}}}),
                &mut client,
            )
            .unwrap();
        assert_eq!(frames(&client)[0]["id"], original_id);
    }

    #[test]
    fn timed_out_enrichment_returns_original_and_swallows_late_internal_replies() {
        let mut router = Router::new(
            true,
            std::path::Path::new("missing-test-index"),
            std::path::Path::new("missing-test-descriptions.json"),
        );
        let mut server = Vec::new();
        let mut client = Vec::new();
        router.client(json!({"jsonrpc":"2.0","id":9007199254740993_u64,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///project/main.c"},"position":{"line":1,"character":2}}}), &mut server).unwrap();
        let requests = frames(&server);
        let original =
            json!({"jsonrpc":"2.0","id":requests[0]["id"],"result":{"contents":"original hover"}});
        router.server(original, &mut client).unwrap();
        router.hovers.values_mut().next().unwrap().deadline =
            Instant::now() - Duration::from_millis(1);
        server.clear();
        router.complete(&mut client, &mut server).unwrap();
        assert_eq!(frames(&client)[0]["id"], json!(9007199254740993_u64));
        assert_eq!(frames(&client)[0]["result"]["contents"], "original hover");
        assert_eq!(frames(&server).len(), 2);
        assert!(router.replies.is_empty());
        client.clear();
        for request in &requests[1..] {
            router
                .server(
                    json!({"jsonrpc":"2.0","id":request["id"],"result":[]}),
                    &mut client,
                )
                .unwrap();
        }
        assert!(client.is_empty());
    }

    #[test]
    fn cancellation_reaches_clangd_and_suppresses_enrichment() {
        let mut router = Router::new(
            true,
            std::path::Path::new("missing-test-index"),
            std::path::Path::new("missing-test-descriptions.json"),
        );
        let mut server = Vec::new();
        let mut client = Vec::new();
        router.client(json!({"jsonrpc":"2.0","id":7,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///project/main.c"},"position":{"line":1,"character":2}}}), &mut server).unwrap();
        let requests = frames(&server);
        server.clear();
        router
            .client(
                json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":7}}),
                &mut server,
            )
            .unwrap();
        assert_eq!(frames(&server).len(), 3);
        router.server(json!({"jsonrpc":"2.0","id":requests[0]["id"],"error":{"code":-32800,"message":"cancelled"}}), &mut client).unwrap();
        router.complete(&mut client, &mut server).unwrap();
        assert_eq!(frames(&client)[0]["error"]["code"], -32800);
        assert_eq!(frames(&client)[0]["id"], 7);
        assert!(router.hovers.is_empty());
    }
}

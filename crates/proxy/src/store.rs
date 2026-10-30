use crate::docs::{self, Entry};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const ARCHIVE_URL: &str = "https://github.com/PeterFeicht/cppreference-doc/releases/download/v20250209/html-book-20250209.tar.xz";
pub const ARCHIVE_SHA256: &str = "ac50671a1f52d7f0ab0911d14450eb35e8c2f812edd0e426b2cd1e3d9db91d6f";

fn verify_archive(compressed: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    if compressed.len() > 128 * 1024 * 1024 {
        return Err("cppreference archive exceeds size limit".into());
    }
    let actual = format!("{:x}", Sha256::digest(compressed));
    if actual != ARCHIVE_SHA256 {
        return Err(format!(
            "cppreference archive SHA-256 mismatch: expected {ARCHIVE_SHA256}, got {actual}. The archive was not imported."
        ).into());
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Index {
    schema: u32,
    entries: Vec<Entry>,
}

pub fn default_dir() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_else(|| ".".into()))
            .join("zed-cpp-docs")
    } else if let Some(base) = std::env::var_os("XDG_DATA_HOME") {
        PathBuf::from(base).join("zed-cpp-docs")
    } else {
        PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| ".".into()))
            .join(".local/share/zed-cpp-docs")
    }
}

pub fn load(data: &Path) -> Result<Vec<Entry>, Box<dyn std::error::Error>> {
    let path = data.join("index.json");
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("{}: {error}", path.display()).into()),
    };
    let index: Index = serde_json::from_reader(BufReader::new(file))
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if index.schema != 1 {
        return Err("Unsupported documentation index schema; rebuild the index".into());
    }
    Ok(index
        .entries
        .into_iter()
        .filter(|entry| entry.source.starts_with("cppreference"))
        .collect())
}

fn publish(data: &Path, entries: Vec<Entry>) -> Result<usize, Box<dyn std::error::Error>> {
    let count = entries.len();
    let temporary = data.join("index.json.new");
    let mut file = File::create(&temporary)?;
    serde_json::to_writer(&mut file, &Index { schema: 1, entries })?;
    file.sync_all()?;
    fs::rename(temporary, data.join("index.json"))?;
    Ok(count)
}

struct Lock(PathBuf);
impl Lock {
    fn take(data: &Path) -> io::Result<Self> {
        fs::create_dir_all(data)?;
        let path = data.join("install.lock");
        OpenOptions::new().write(true).create_new(true).open(&path)
            .map_err(|error| io::Error::new(error.kind(), format!("Cannot acquire {}: {error}. Another import may be running; remove a stale lock only after checking.", path.display())))?;
        Ok(Self(path))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn download(data: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(data.join("downloads"))?;
    let path = data.join("downloads/html-book-20250209.tar.xz");
    let partial = data.join(format!(
        "downloads/html-book-20250209.tar.xz.part-{}",
        std::process::id()
    ));
    let curl = if cfg!(windows) { "curl.exe" } else { "curl" };
    let status = Command::new(curl)
        .args([
            "--fail",
            "--location",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--tlsv1.2",
            "--connect-timeout",
            "20",
            "--max-time",
            "600",
            "--max-filesize",
            "134217728",
            "--output",
        ])
        .arg(&partial)
        .arg(ARCHIVE_URL)
        .status()?;
    if !status.success() {
        let _ = fs::remove_file(&partial);
        return Err("cppreference download failed; curl is required only for initial setup. Import an existing archive with --import-cppreference PATH.".into());
    }
    finish_download(&partial, &path)?;
    Ok(path)
}

fn finish_download(partial: &Path, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Err(error) = verify_archive(&fs::read(partial)?) {
        let _ = fs::remove_file(partial);
        return Err(error);
    }
    fs::rename(partial, path)?;
    Ok(())
}

pub fn ensure(data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !load(data)?.is_empty() {
        return Ok(());
    }
    let archive = download(data)?;
    import_cppreference(data, &archive)?;
    Ok(())
}

pub fn import_cppreference(
    data: &Path,
    archive: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let _lock = Lock::take(data)?;
    let compressed = fs::read(archive)?;
    verify_archive(&compressed)?;
    let digest = ARCHIVE_SHA256;
    let directory = format!("cppreference-20250209-{}", &digest[..12]);
    let destination = data.join(&directory);
    let mut expanded = LimitedWriter {
        inner: Vec::new(),
        remaining: 768 * 1024 * 1024,
    };
    lzma_rs::xz_decompress(&mut BufReader::new(compressed.as_slice()), &mut expanded)?;
    let expanded = expanded.inner;
    if !destination.exists() {
        let staging = data.join(format!("{directory}.staging-{}", std::process::id()));
        fs::create_dir(&staging)?;
        let mut tar = tar::Archive::new(Cursor::new(&expanded));
        for item in tar.entries()? {
            let mut item = item?;
            let kind = item.header().entry_type();
            if !kind.is_file() && !kind.is_dir() {
                return Err("Archive contains links or special files".into());
            }
            let path = item.path()?.into_owned();
            if path.is_absolute()
                || path
                    .components()
                    .any(|part| !matches!(part, std::path::Component::Normal(_)))
            {
                return Err("Archive contains an unsafe path".into());
            }
            if !item.unpack_in(&staging)? {
                return Err("Archive path escaped staging directory".into());
            }
        }
        if !staging.join("reference/en/cpp.html").is_file() {
            return Err("Expected cppreference HTML book structure was not found".into());
        }
        fs::write(
            staging.join("archive-sha256.txt"),
            format!("{digest}\n{ARCHIVE_URL}\n"),
        )?;
        fs::rename(staging, &destination)?;
    }
    let mut entries = Vec::new();
    let mut pages = 0;
    let mut tar = tar::Archive::new(Cursor::new(&expanded));
    for item in tar.entries()? {
        let mut item = item?;
        let path = item.path()?.into_owned();
        if !item.header().entry_type().is_file()
            || !path.starts_with("reference/en")
            || path.extension().and_then(|ext| ext.to_str()) != Some("html")
        {
            continue;
        }
        if path
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err("Archive contains an unsafe path".into());
        }
        let relative = format!("{directory}/{}", path.to_string_lossy().replace('\\', "/"));
        let mut html = String::new();
        item.read_to_string(&mut html)?;
        entries.extend(parse_cppreference(&html, &relative));
        pages += 1;
        if pages % 1000 == 0 {
            eprintln!("Indexed {pages} pages so far");
        }
    }
    if entries.is_empty() {
        return Err("No symbol entries found in cppreference archive".into());
    }
    let page_headers: BTreeMap<_, _> = entries
        .iter()
        .filter(|entry| !entry.headers.is_empty())
        .map(|entry| {
            (
                entry
                    .html_path
                    .split('#')
                    .next()
                    .unwrap_or(&entry.html_path)
                    .to_string(),
                entry.headers.clone(),
            )
        })
        .collect();
    for entry in &mut entries {
        if !entry.headers.is_empty() {
            continue;
        }
        let mut parent = Path::new(
            entry
                .html_path
                .split('#')
                .next()
                .unwrap_or(&entry.html_path),
        )
        .parent();
        while let Some(path) = parent {
            let page = format!("{}.html", path.to_string_lossy().replace('\\', "/"));
            if let Some(headers) = page_headers.get(&page) {
                entry.headers = headers.clone();
                break;
            }
            parent = path.parent();
        }
    }
    let mut unique: BTreeMap<(String, String), Entry> = BTreeMap::new();
    for entry in entries {
        let page = entry
            .html_path
            .split('#')
            .next()
            .unwrap_or(&entry.html_path)
            .to_string();
        let key = (entry.symbol.clone(), page);
        if let Some(existing) = unique.get_mut(&key) {
            if existing.overview && !entry.overview {
                *existing = entry;
            } else {
                for header in entry.headers {
                    if !existing.headers.contains(&header) {
                        existing.headers.push(header);
                    }
                }
                if existing.signature == existing.symbol && entry.signature != entry.symbol {
                    existing.signature = entry.signature;
                }
            }
        } else {
            unique.insert(key, entry);
        }
    }
    let count = publish(data, unique.into_values().collect())?;
    eprintln!("Indexed {count} cppreference symbols; archive SHA-256 {digest}");
    Ok(count)
}

struct LimitedWriter<W> {
    inner: W,
    remaining: usize,
}
impl<W: Write> Write for LimitedWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.remaining {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expanded archive exceeds limit",
            ));
        }
        let count = self.inner.write(bytes)?;
        self.remaining -= count;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn selector(css: &str) -> Selector {
    Selector::parse(css).expect("Static selector")
}
fn compact(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn mentions(text: &str, name: &str) -> bool {
    !name.is_empty()
        && text.match_indices(name).any(|(offset, _)| {
            let before = text[..offset].chars().next_back();
            let after = text[offset + name.len()..].chars().next();
            !before.is_some_and(|character| character.is_alphanumeric() || character == '_')
                && !after.is_some_and(|character| character.is_alphanumeric() || character == '_')
        })
}
fn text(element: ElementRef<'_>) -> String {
    compact(&element.text().collect::<String>())
}
fn shorten(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        text.into()
    } else {
        format!("{} …", text.chars().take(limit).collect::<String>())
    }
}

fn section(document: &Html, id: &str) -> String {
    let Some(anchor) = document.select(&selector(&format!("#{id}"))).next() else {
        return String::new();
    };
    let Some(heading) = anchor.parent() else {
        return String::new();
    };
    let mut paragraphs = Vec::new();
    for sibling in heading.next_siblings().filter_map(ElementRef::wrap) {
        if matches!(sibling.value().name(), "h2" | "h3" | "h4") {
            break;
        }
        let value = text(sibling);
        if !value.is_empty() {
            paragraphs.push(value);
        }
        if paragraphs.len() >= 3 {
            break;
        }
    }
    shorten(&paragraphs.join("\n\n"), 650)
}

fn symbol_names(title: &str) -> Vec<String> {
    docs::canonical(title)
        .split(',')
        .map(str::trim)
        .map(|name| {
            if name.contains("operator") {
                name.to_string()
            } else {
                name.split('(').next().unwrap_or(name).trim().to_string()
            }
        })
        .filter(|name| valid_symbol(name))
        .collect()
}

fn valid_symbol(name: &str) -> bool {
    if let Some((prefix, operator)) = name.split_once("operator") {
        let operator = operator.trim();
        return (prefix.is_empty()
            || (prefix.ends_with("::") && valid_symbol(prefix.trim_end_matches("::"))))
            && (matches!(
                operator,
                "()" | "[]"
                    | "+"
                    | "-"
                    | "*"
                    | "/"
                    | "%"
                    | "^"
                    | "&"
                    | "|"
                    | "~"
                    | "!"
                    | "="
                    | "<"
                    | ">"
                    | "+="
                    | "-="
                    | "*="
                    | "/="
                    | "%="
                    | "^="
                    | "&="
                    | "|="
                    | "<<"
                    | ">>"
                    | "<<="
                    | ">>="
                    | "=="
                    | "!="
                    | "<="
                    | ">="
                    | "<=>"
                    | "&&"
                    | "||"
                    | "++"
                    | "--"
                    | ","
                    | "->"
                    | "->*"
                    | "new"
                    | "new[]"
                    | "delete"
                    | "delete[]"
            ) || operator.strip_prefix("\"\"").is_some_and(valid_symbol)
                || (!operator.is_empty()
                    && operator.chars().all(|c| {
                        c.is_alphanumeric() || matches!(c, '_' | ':' | ' ' | '*' | '&')
                    })));
    }
    !name.is_empty()
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | ':' | '~'))
}

fn operator_names(title: &str, signatures: &[String]) -> Vec<String> {
    let title = docs::canonical(title);
    let prefix = title
        .split_once("operator")
        .map(|(prefix, _)| prefix)
        .unwrap_or("");
    let scope = if prefix.ends_with("::") {
        prefix.to_string()
    } else {
        title
            .split_once('(')
            .and_then(|(_, rest)| rest.strip_suffix(')'))
            .and_then(|owner| owner.rsplit_once("::"))
            .filter(|(scope, _)| valid_symbol(scope))
            .map(|(scope, _)| format!("{scope}::"))
            .unwrap_or_default()
    };
    let mut names = Vec::new();
    for signature in signatures {
        let Some((_, rest)) = signature.split_once("operator") else {
            continue;
        };
        let operator = if rest.starts_with("()") {
            "()"
        } else {
            rest.split('(').next().unwrap_or("").trim()
        };
        let separator = if operator.chars().next().is_some_and(|c| c.is_alphanumeric()) {
            " "
        } else {
            ""
        };
        let symbol = format!("{scope}operator{separator}{operator}");
        if valid_symbol(&symbol) && !names.contains(&symbol) {
            names.push(symbol);
        }
    }
    names
}

fn declaration_kind(signature: &str, name: &str) -> &'static str {
    if signature.starts_with('#') {
        "macro"
    } else if signature.contains(&format!("concept {name}")) {
        "concept"
    } else if ["class", "struct", "union", "enum", "enum class"]
        .iter()
        .any(|word| signature.contains(&format!("{word} {name}")))
    {
        "type"
    } else if signature.contains("typedef ") || signature.contains(&format!("using {name}")) {
        "alias"
    } else if signature.contains('(') {
        "function"
    } else {
        "variable"
    }
}

fn table_entries(declarations: ElementRef<'_>, primary: &Entry) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut heading = String::new();
    for element in declarations.next_siblings().filter_map(ElementRef::wrap) {
        if matches!(element.value().name(), "h2" | "h3") {
            heading = text(element);
            if heading.starts_with("Inherited from") {
                break;
            }
        }
        if element.value().name() != "table" {
            continue;
        }
        let nested_types = matches!(
            heading.as_str(),
            "Member types" | "Nested types" | "Member typedefs" | "Member types and constants"
        );
        let member_objects = matches!(
            heading.as_str(),
            "Member objects"
                | "Data members"
                | "Member constants"
                | "Constants"
                | "Enumerators"
                | "Enumeration constants"
        );
        let enumeration = primary.signature.contains("enum ");
        let scoped_enum =
            primary.signature.contains("enum class ") || primary.signature.contains("enum struct ");
        let aliases = heading.is_empty() && primary.kind == "type" && !enumeration;
        let constants = (heading.is_empty() && (primary.kind == "alias" || enumeration))
            || (enumeration && heading == "Enumeration constants");
        if !aliases && !nested_types && !member_objects && !constants {
            continue;
        }
        for row in element.select(&selector("tr.t-dsc")) {
            let cells: Vec<_> = row
                .children()
                .filter_map(ElementRef::wrap)
                .filter(|child| child.value().name() == "td")
                .collect();
            if cells.len() != 2 {
                continue;
            }
            let label = cells[0]
                .select(&selector("code"))
                .next()
                .map(text)
                .unwrap_or_else(|| text(cells[0]));
            let label = label.split_whitespace().next().unwrap_or("");
            if !valid_symbol(label) {
                continue;
            }
            let definition = text(cells[1]);
            if definition.is_empty() {
                continue;
            }
            let mut entry = primary.clone();
            if aliases {
                if !label.starts_with("std::") || !definition.starts_with("std::") {
                    continue;
                }
                entry.symbol = label.into();
                entry.signature = format!(
                    "using {} = {};",
                    label.rsplit("::").next().unwrap(),
                    definition
                );
                entry.summary = format!("Alias for {definition}.\n\n{}", primary.summary);
                entry.kind = "alias".into();
                entry.overview = false;
            } else {
                let owner = if constants && !scoped_enum {
                    primary
                        .symbol
                        .rsplit_once("::")
                        .map(|(owner, _)| owner)
                        .unwrap_or("")
                } else {
                    &primary.symbol
                };
                entry.symbol = if label.contains("::") || owner.is_empty() {
                    label.into()
                } else {
                    format!("{owner}::{label}")
                };
                entry.signature = entry.symbol.clone();
                entry.summary = format!(
                    "{} of {owner}.\n\n{definition}",
                    if nested_types {
                        "Member type"
                    } else {
                        "Member or constant"
                    }
                );
                entry.kind = if nested_types { "alias" } else { "variable" }.into();
                entry.overview = true;
                entry.example.clear();
            }
            entry.parameters.clear();
            entry.returns.clear();
            entry.template_parameters.clear();
            entry.members.clear();
            entry.editorial = false;
            entry.family = false;
            if let Some(anchor) = cells[0]
                .select(&selector("[id]"))
                .next()
                .and_then(|element| element.value().attr("id"))
            {
                entry.html_path = format!("{}#{anchor}", primary.html_path);
            }
            entries.push(entry);
        }
    }
    entries
}

fn additional_declarations(document: &Html, primary: &Entry) -> Vec<Entry> {
    let mut entries = Vec::new();
    let scope = primary
        .symbol
        .rsplit_once("::")
        .map(|(scope, _)| scope)
        .unwrap_or("");
    for row in document.select(&selector(".t-dcl")) {
        let Some(cell) = row.select(&selector("td")).next() else {
            continue;
        };
        let signature = text(cell);
        let using = signature
            .find("using ")
            .map(|offset| &signature[offset + 6..]);
        let name = if let Some(using) = using {
            using.split('=').next().unwrap_or("").trim().to_string()
        } else if signature.contains("constexpr ") || signature.contains("const ") {
            signature
                .rsplit_once('=')
                .filter(|(left, _)| !left.contains('('))
                .and_then(|(left, _)| left.split_whitespace().next_back())
                .map(docs::canonical)
                .unwrap_or_default()
        } else {
            continue;
        };
        if !valid_symbol(&name) {
            continue;
        }
        let namespace = signature
            .strip_prefix("namespace ")
            .and_then(|rest| rest.split_whitespace().next());
        let owner = namespace
            .map(|namespace| format!("{scope}::{namespace}"))
            .unwrap_or_else(|| scope.to_string());
        let symbol = if owner.is_empty() || name.contains("::") {
            name
        } else {
            format!("{owner}::{name}")
        };
        if symbol == primary.symbol {
            continue;
        }
        let mut entry = primary.clone();
        entry.symbol = symbol;
        entry.signature = shorten(&signature, 700);
        entry.kind = if using.is_some() { "alias" } else { "variable" }.into();
        entry.summary = format!("Declaration provided by the {} reference page. See the local documentation for details.", primary.symbol);
        entry.parameters.clear();
        entry.returns.clear();
        entry.template_parameters.clear();
        entry.members.clear();
        entry.example.clear();
        entry.editorial = false;
        entry.family = false;
        entry.overview = true;
        entries.push(entry);
    }
    entries
}

fn overview_macros(document: &Html, relative: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    for table in document.select(&selector("table.t-dsc-begin")) {
        let headers: Vec<_> = table
            .select(&selector(".t-dsc-header code"))
            .map(text)
            .filter(|header| header.starts_with('<') && header.ends_with('>'))
            .map(|header| header.trim_matches(['<', '>']).to_string())
            .collect();
        if headers.is_empty() {
            continue;
        }
        for row in table.select(&selector("tr.t-dsc")) {
            let cells: Vec<_> = row
                .children()
                .filter_map(ElementRef::wrap)
                .filter(|cell| cell.value().name() == "td")
                .collect();
            if cells.len() != 2 {
                continue;
            }
            let summary = text(cells[1]);
            if !summary.contains("macro") {
                continue;
            }
            let mut labels: Vec<_> = cells[0]
                .select(&selector(".t-lines > span"))
                .map(text)
                .collect();
            if labels.is_empty() {
                labels.push(text(cells[0]));
            }
            for label in labels {
                for symbol in symbol_names(&label) {
                    if symbol.contains("::") {
                        continue;
                    }
                    entries.push(Entry {
                        signature: symbol.clone(),
                        symbol,
                        source: "cppreference (HTML book 20250209)".into(),
                        headers: headers.clone(),
                        summary: shorten(&summary, 650),
                        html_path: relative.into(),
                        kind: "macro".into(),
                        overview: true,
                        source_url: relative
                            .split("/reference/en/")
                            .nth(1)
                            .map(|path| {
                                format!(
                                    "https://en.cppreference.com/w/{}",
                                    path.trim_end_matches(".html")
                                )
                            })
                            .unwrap_or_default(),
                        ..Entry::default()
                    });
                }
            }
        }
    }
    entries
}

fn parse_cppreference(html: &str, relative: &str) -> Vec<Entry> {
    let document = Html::parse_document(html);
    let overview = overview_macros(&document, relative);
    let title = document
        .select(&selector("#firstHeading"))
        .next()
        .map(text)
        .unwrap_or_default();
    let Some(declarations) = document.select(&selector(".t-dcl-begin")).next() else {
        return overview;
    };
    let signatures: Vec<_> = declarations
        .select(&selector(".t-dcl"))
        .filter_map(|row| row.select(&selector("td")).next())
        .map(text)
        .collect();
    if signatures.is_empty() {
        return overview;
    }
    let headers: Vec<_> = declarations
        .select(&selector(".t-dsc-header code"))
        .map(text)
        .map(|value| value.trim_matches(['<', '>']).to_string())
        .filter(|value| !value.is_empty())
        .collect();
    let mut summary = Vec::new();
    for element in declarations.next_siblings().filter_map(ElementRef::wrap) {
        if matches!(element.value().name(), "h2" | "h3") {
            break;
        }
        if element.value().name() == "p" {
            summary.push(text(element));
        }
        if summary.len() == 2 {
            break;
        }
    }
    let parameters = document
        .select(&selector(".t-par"))
        .take(12)
        .map(|row| {
            (
                row.select(&selector("td"))
                    .next()
                    .map(text)
                    .unwrap_or_default(),
                format!("- {}", text(row)),
            )
        })
        .collect::<Vec<_>>();
    let example = document
        .select(&selector(".t-example pre"))
        .next()
        .map(|pre| {
            pre.text()
                .collect::<String>()
                .lines()
                .take(22)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    let returns = section(&document, "Return_value");
    let names = if title.contains("operator") {
        operator_names(&title, &signatures)
    } else {
        symbol_names(&title)
    };
    let source_url = relative
        .split("/reference/en/")
        .nth(1)
        .map(|path| {
            format!(
                "https://en.cppreference.com/w/{}",
                path.trim_end_matches(".html")
            )
        })
        .unwrap_or_default();
    let mut entries: Vec<_> = names
        .iter()
        .map(|symbol| {
            let name = symbol.rsplit("::").next().unwrap_or(symbol);
            let matching: Vec<_> = signatures
                .iter()
                .filter(|signature| mentions(signature, name))
                .filter(|signature| !signature.starts_with("namespace "))
                .take(2)
                .cloned()
                .collect();
            let signature = matching.join("\n");
            let kind = declaration_kind(&signature, name);
            let selected_parameters = parameters
                .iter()
                .filter(|(name, _)| mentions(&signature, name) || name == "...")
                .map(|(_, description)| description.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            Entry {
                symbol: symbol.clone(),
                source: "cppreference (HTML book 20250209)".into(),
                headers: headers.clone(),
                summary: shorten(&summary.join("\n\n"), 650),
                signature: if signature.is_empty() {
                    "See the complete declaration in the local documentation.".into()
                } else {
                    shorten(&signature, 700)
                },
                parameters: if kind == "function" {
                    selected_parameters
                } else {
                    String::new()
                },
                returns: if kind == "function" {
                    returns.clone()
                } else {
                    String::new()
                },
                example: example.clone(),
                fixture: false,
                html_path: relative.into(),
                source_url: source_url.clone(),
                editorial: false,
                user_description: false,
                family: names.len() > 1,
                kind: kind.into(),
                display_kind: String::new(),
                template_parameters: section(&document, "Template_parameters"),
                members: String::new(),
                overview: kind == "type" && title.trim_end().ends_with('>'),
            }
        })
        .collect();
    if let Some(primary) = entries.first() {
        let mut table = table_entries(declarations, primary);
        table.extend(additional_declarations(&document, primary));
        let member_summary = table
            .iter()
            .filter(|entry| entry.overview)
            .take(8)
            .map(|entry| {
                format!(
                    "- `{}`: {}",
                    entry.symbol.rsplit("::").next().unwrap_or(&entry.symbol),
                    shorten(&entry.summary, 100)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        for entry in &mut entries {
            if entry.kind == "type" {
                entry.members = member_summary.clone();
            }
        }
        entries.extend(table);
    }
    entries.extend(overview);
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrong_archive_is_rejected_before_extraction_and_preserves_the_index() {
        let directory =
            std::env::temp_dir().join(format!("cpp-docs-bad-archive-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let archive = directory.join("invalid.tar.xz");
        fs::write(&archive, b"not the pinned cppreference archive").unwrap();
        fs::write(directory.join("index.json"), b"existing index").unwrap();
        let error = import_cppreference(&directory, &archive)
            .unwrap_err()
            .to_string();
        assert!(error.contains("SHA-256 mismatch"));
        assert_eq!(
            fs::read(directory.join("index.json")).unwrap(),
            b"existing index"
        );
        assert!(!directory.join("install.lock").exists());
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 2);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn ensuring_an_installed_index_does_not_download_or_rewrite_it() {
        let directory =
            std::env::temp_dir().join(format!("cpp-docs-ensure-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        publish(&directory, docs::fixtures()).unwrap();
        let before = fs::read(directory.join("index.json")).unwrap();
        ensure(&directory).unwrap();
        assert_eq!(fs::read(directory.join("index.json")).unwrap(), before);
        assert!(!directory.join("downloads").exists());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn a_bad_download_does_not_replace_the_cached_archive() {
        let directory =
            std::env::temp_dir().join(format!("cpp-docs-bad-download-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let cached = directory.join("book.tar.xz");
        let partial = directory.join("book.tar.xz.part");
        fs::write(&cached, b"previous archive").unwrap();
        fs::write(&partial, b"incorrect download").unwrap();
        assert!(finish_download(&partial, &cached)
            .unwrap_err()
            .to_string()
            .contains("SHA-256 mismatch"));
        assert!(!partial.exists());
        assert_eq!(fs::read(&cached).unwrap(), b"previous archive");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn overview_only_macros_and_nonmember_operators_are_indexed() {
        let html = r#"<h1 id="firstHeading">C input/output</h1><table class="t-dsc-begin"><tr class="t-dsc-header"><td><code>&lt;stdio.h&gt;</code></td></tr><tr class="t-dsc"><td><span class="t-lines"><span>EOF</span><span>BUFSIZ</span></span></td><td>Input/output limits (macro constant)</td></tr></table>"#;
        let entries = parse_cppreference(html, "test/reference/en/c/io.html");
        assert_eq!(entries.len(), 2);
        assert!(entries
            .iter()
            .all(|entry| entry.kind == "macro" && entry.headers == ["stdio.h"]));
        let html = r#"<h1 id="firstHeading">operator==, operator!= (std::vector)</h1><table class="t-dcl-begin"><tr class="t-dcl"><td>bool operator==(const vector&amp; a, const vector&amp; b);</td></tr><tr class="t-dcl"><td>bool operator!=(const vector&amp; a, const vector&amp; b);</td></tr></table><p>Compares vectors.</p>"#;
        let entries = parse_cppreference(
            html,
            "test/reference/en/cpp/container/vector/operator_cmp.html",
        );
        assert!(entries
            .iter()
            .any(|entry| entry.symbol == "std::operator=="));
        assert!(entries
            .iter()
            .any(|entry| entry.symbol == "std::operator!="));
        assert!(!valid_symbol("operator==(std::operator="));
    }

    #[test]
    fn indexes_types_aliases_nested_types_and_objects() {
        let html = r#"<h1 id="firstHeading">std::basic_string</h1>
<table class="t-dcl-begin"><tr class="t-dsc-header"><td><code>&lt;string&gt;</code></td></tr>
<tr class="t-dcl"><td>template&lt;class CharT&gt; class basic_string;</td></tr>
<tr class="t-dcl"><td>namespace pmr { template&lt;class CharT&gt; using basic_string = std::basic_string&lt;CharT&gt;; }</td></tr></table>
<p>Stores and manipulates sequences of characters.</p>
<table><tr class="t-dsc"><td><code>std::string</code></td><td>std::basic_string&lt;char&gt;</td></tr></table>
<h3><span id="Template_parameters">Template parameters</span></h3>
<table><tr class="t-par"><td>CharT</td><td>character type</td></tr></table>
<h3>Nested types</h3><table><tr class="t-dsc"><td><code>value_type</code></td><td>CharT</td></tr></table>
<h3>Data members</h3><table><tr class="t-dsc"><td><code>npos</code></td><td>Special value for no match.</td></tr></table>
<h3>Inherited from std::other</h3><h3>Member types</h3><table><tr class="t-dsc"><td>unrelated</td><td>int</td></tr></table>"#;
        let entries = parse_cppreference(html, "test/reference/en/cpp/string/basic_string.html");
        let find = |name| entries.iter().find(|entry| entry.symbol == name).unwrap();
        assert_eq!(find("std::basic_string").kind, "type");
        assert!(find("std::basic_string").members.contains("value_type"));
        assert!(find("std::basic_string")
            .template_parameters
            .contains("character type"));
        assert!(find("std::basic_string").parameters.is_empty());
        assert_eq!(
            find("std::string").signature,
            "using string = std::basic_string<char>;"
        );
        assert_eq!(find("std::string").kind, "alias");
        assert_eq!(find("std::basic_string::value_type").kind, "alias");
        assert_eq!(find("std::basic_string::npos").kind, "variable");
        assert_eq!(find("std::pmr::basic_string").kind, "alias");
        assert!(!entries
            .iter()
            .any(|entry| entry.symbol.ends_with("unrelated")));
    }

    #[test]
    fn indexes_macros_concepts_operators_and_helper_templates() {
        for (title, declaration, kind) in [
            ("EOF", "#define EOF /* implementation-defined */", "macro"),
            (
                "std::size_t",
                "typedef /* implementation-defined */ size_t;",
                "alias",
            ),
            (
                "std::same_as",
                "template&lt;class T, class U&gt; concept same_as = /* see below */;",
                "concept",
            ),
            (
                "std::vector&lt;T&gt;::operator[]",
                "T&amp; operator[](size_type position);",
                "function",
            ),
            (
                "std::vector&lt;T&gt;::operator&lt;=&gt;",
                "auto operator&lt;=&gt;(const vector&amp; other);",
                "function",
            ),
        ] {
            let html = format!("<h1 id=\"firstHeading\">{title}</h1><table class=\"t-dcl-begin\"><tr class=\"t-dcl\"><td>{declaration}</td></tr></table><p>Reference description.</p>");
            let entries = parse_cppreference(&html, "test/reference/en/cpp/test.html");
            assert_eq!(entries[0].kind, kind, "{title}");
            assert!(entries[0].returns.is_empty());
        }
        let html = r#"<h1 id="firstHeading">std::is_const</h1><table class="t-dcl-begin"><tr class="t-dcl"><td>template&lt;class T&gt; struct is_const;</td></tr></table><p>Checks const qualification.</p><h3>Helper variable template</h3><table class="t-dcl-begin"><tr class="t-dcl"><td>template&lt;class T&gt; inline constexpr bool is_const_v = is_const&lt;T&gt;::value;</td></tr></table>"#;
        let entries = parse_cppreference(html, "test/reference/en/cpp/types/is_const.html");
        assert_eq!(
            entries
                .iter()
                .find(|entry| entry.symbol == "std::is_const_v")
                .unwrap()
                .kind,
            "variable"
        );
    }

    #[test]
    fn enum_constants_use_scoped_or_enclosing_namespace() {
        for (declaration, symbol) in [
            ("enum mode { nearest };", "std::nearest"),
            ("enum class mode { nearest };", "std::mode::nearest"),
        ] {
            let html = format!("<h1 id=\"firstHeading\">std::mode</h1><table class=\"t-dcl-begin\"><tr class=\"t-dcl\"><td>{declaration}</td></tr></table><h3>Enumeration constants</h3><table><tr class=\"t-dsc\"><td>nearest</td><td>Round to the nearest value.</td></tr></table>");
            let entries = parse_cppreference(&html, "test/reference/en/cpp/mode.html");
            assert!(
                entries.iter().any(|entry| entry.symbol == symbol),
                "{symbol}"
            );
        }
    }

    #[test]
    fn only_cppreference_is_loaded_and_published() {
        let directory =
            std::env::temp_dir().join(format!("cpp-docs-sources-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let mut entries = docs::fixtures();
        let mut other = entries[0].clone();
        other.source = "Other reference".into();
        entries.push(other);
        fs::write(
            directory.join("index.json"),
            serde_json::to_vec(&Index { schema: 1, entries }).unwrap(),
        )
        .unwrap();
        assert_eq!(load(&directory).unwrap().len(), docs::fixtures().len());
        let entry = docs::fixtures().remove(0);
        assert_eq!(publish(&directory, vec![entry]).unwrap(), 1);
        let index: Index =
            serde_json::from_reader(File::open(directory.join("index.json")).unwrap()).unwrap();
        assert_eq!(index.entries.len(), 1);
        assert!(index.entries[0].source.starts_with("cppreference"));
        fs::remove_file(directory.join("index.json")).unwrap();
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn missing_index_is_distinct_from_unreadable_index_content() {
        let directory = std::env::temp_dir().join(format!("cpp-docs-index-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        assert!(load(&directory).unwrap().is_empty());
        fs::write(directory.join("index.json"), "not json").unwrap();
        let error = load(&directory).err().unwrap().to_string();
        assert!(error.contains("index.json"));
        fs::remove_file(directory.join("index.json")).unwrap();
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn imported_family_pages_preserve_extracted_text_without_compact_notes() {
        let html = r#"<h1 id="firstHeading">printf, fprintf</h1><table class="t-dcl-begin"><tr class="t-dsc-header"><td><code>&lt;stdio.h&gt;</code></td></tr><tr class="t-dcl"><td>int printf(const char *format, ...);</td></tr></table><p>Write formatted output.</p><table><tr class="t-par"><td>stream - output stream</td></tr></table>"#;
        let entries = parse_cppreference(html, "test/reference/en/c/io/fprintf.html");
        let printf = entries
            .iter()
            .find(|entry| entry.symbol == "printf")
            .unwrap();
        assert!(!printf.editorial);
        assert!(!printf.parameters.contains("stream"));
        assert!(printf.example.is_empty());
        assert!(printf.source_url.ends_with("/c/io/fprintf"));
        assert_eq!(entries.len(), 2);
    }
}

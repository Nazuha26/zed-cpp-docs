use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Entry {
    pub symbol: String,
    pub source: String,
    pub headers: Vec<String>,
    pub summary: String,
    pub signature: String,
    pub parameters: String,
    pub returns: String,
    pub example: String,
    #[serde(default = "is_fixture")]
    pub fixture: bool,
    #[serde(default)]
    pub html_path: String,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub editorial: bool,
    #[serde(default)]
    pub user_description: bool,
    #[serde(default)]
    pub family: bool,
    #[serde(default = "function_kind")]
    pub kind: String,
    #[serde(default)]
    pub template_parameters: String,
    #[serde(default)]
    pub members: String,
    #[serde(default)]
    pub overview: bool,
}

fn function_kind() -> String {
    "function".into()
}

fn is_fixture() -> bool {
    true
}

pub fn fixtures() -> Vec<Entry> {
    serde_json::from_str::<Vec<Entry>>(include_str!("../../../fixtures/docs.json"))
        .expect("Bundled fixture data is valid")
}

pub fn index(entries: &[Entry]) -> HashMap<String, Vec<usize>> {
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();
    for (number, entry) in entries.iter().enumerate() {
        index.entry(entry.symbol.clone()).or_default().push(number);
    }
    index
}

pub fn canonical(name: &str) -> String {
    let mut result = String::new();
    let mut depth = 0_u32;
    for character in name.chars() {
        match character {
            '<' | '>'
                if depth == 0
                    && result
                        .rsplit("::")
                        .next()
                        .is_some_and(|part| part.starts_with("operator")) =>
            {
                result.push(character)
            }
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => result.push(character),
            _ => {}
        }
    }
    for abi in [
        "std::__1::",
        "std::__2::",
        "std::__cxx11::",
        "std::__ndk1::",
    ] {
        result = result.replace(abi, "std::");
    }
    // MSVC declares the public ios_base flags in its internal base class.
    result = result.replace("std::_Iosb::", "std::ios_base::");
    result.trim_start_matches("::").to_string()
}

pub fn resolve<'a>(
    entries: &'a [Entry],
    symbols: &Value,
    declaration: &Value,
    roots: &[url::Url],
    index: &HashMap<String, Vec<usize>>,
) -> Option<&'a Entry> {
    let symbols = symbols.get("result")?.as_array()?;
    let declaration_location = declaration.get("result").and_then(|value| {
        if let Some(array) = value.as_array() {
            (array.len() == 1).then(|| &array[0])
        } else {
            value.is_object().then_some(value)
        }
    });
    let info = if symbols.len() == 1 {
        &symbols[0]
    } else {
        let location = declaration_location?;
        let uri = location.get("uri").or_else(|| location.get("targetUri"))?;
        let range = location
            .get("range")
            .or_else(|| location.get("targetSelectionRange"))?;
        let mut matching = symbols.iter().filter(|info| {
            info.pointer("/declarationRange/uri") == Some(uri)
                && info.pointer("/declarationRange/range") == Some(range)
        });
        let info = matching.next()?;
        if matching.next().is_some() {
            return None;
        }
        info
    };
    let name = info.get("name")?.as_str()?;
    let container = info
        .get("containerName")
        .and_then(Value::as_str)
        .unwrap_or("");
    let qualified = canonical(&format!("{container}{name}"));
    // clangd containers normally end in ::; accept older variants without it.
    let qualified = if !container.is_empty() && !container.ends_with("::") {
        canonical(&format!("{container}::{name}"))
    } else {
        qualified
    };
    let location = declaration_location
        .or_else(|| info.get("declarationRange"))
        .or_else(|| info.get("definitionRange"))?;
    let uri = location
        .get("uri")
        .or_else(|| location.get("targetUri"))?
        .as_str()?;
    let uri = url::Url::parse(uri).ok()?;
    if uri.scheme() != "file" {
        return None;
    }
    let path = uri.to_file_path().ok()?;
    // A declaration in the project takes precedence over library lookalikes.
    if roots
        .iter()
        .filter_map(|root| root.to_file_path().ok())
        .any(|root| path.starts_with(root))
    {
        return None;
    }
    let header = path.file_name()?.to_str()?;
    let normalized_path = path.to_string_lossy().replace('\\', "/").to_lowercase();
    let standard_cpp = qualified.starts_with("std::")
        && (normalized_path.contains("/include/c++/")
            // GCC also installs architecture-specific headers under include/<target>/c++/.
            || normalized_path
                .split_once("/include/")
                .and_then(|(_, relative)| relative.split_once('/'))
                .is_some_and(|(_, relative)| relative.starts_with("c++/"))
            || normalized_path.contains("/vc/tools/msvc/"));
    let matches: Vec<_> = index
        .get(&qualified)?
        .iter()
        .map(|number| &entries[*number])
        .filter(|entry| {
            entry.symbol == qualified
                && (standard_cpp
                    || (entry.kind == "macro"
                        && (normalized_path.contains("/vc/tools/msvc/")
                            || (normalized_path.contains("/windows kits/")
                                && normalized_path.contains("/ucrt/"))))
                    || entry.headers.iter().any(|candidate| {
                        Path::new(candidate)
                            .file_name()
                            .and_then(|name| name.to_str())
                            == Some(header)
                    }))
        })
        .collect();
    let overview = matches.iter().map(|entry| entry.overview).min()?;
    let preferred: Vec<_> = matches
        .into_iter()
        .filter(|entry| entry.overview == overview)
        .collect();
    let exact: Vec<_> = preferred
        .iter()
        .copied()
        .filter(|entry| entry.headers.iter().any(|candidate| candidate == header))
        .collect();
    let candidates = if exact.is_empty() { preferred } else { exact };
    if candidates.len() == 1 || candidates.iter().all(|entry| entry.kind == "macro") {
        candidates.first().copied()
    } else {
        None
    }
}

pub fn markdown(entry: &Entry, full_page: Option<&str>) -> String {
    let mut text = format!(
        "### {}\n\n{}\n\n```cpp\n{}\n```",
        entry.symbol, entry.summary, entry.signature
    );
    for (title, content) in [
        ("Template parameters", &entry.template_parameters),
        ("Parameters", &entry.parameters),
        ("Returns", &entry.returns),
        ("Members", &entry.members),
        ("Example", &entry.example),
    ] {
        if content.is_empty() {
            continue;
        }
        text.push_str(&format!("\n\n**{title}**\n\n"));
        if title == "Example" {
            text.push_str(&format!("```cpp\n{content}\n```"));
        } else {
            text.push_str(content);
        }
    }
    if entry.fixture {
        text.push_str(&format!(
            "\n\n_Prototype fixture for {}; full source database is not installed._",
            entry.source
        ));
    } else {
        text.push_str(&format!("\n\n_Source: {}. Extracted locally; full declarations and details are in the linked page._", entry.source));
        if entry.source.starts_with("cppreference") {
            text.push_str(" CC BY-SA 3.0 / GFDL; cppreference contributors.");
        }
        if entry.editorial {
            text.push_str(" Includes compact descriptions maintained by this extension.");
        } else if !entry.example.is_empty() {
            text.push_str(
                " Example shown as an excerpt; see the local page for the complete program.",
            );
        }
        if entry.family && !entry.editorial {
            text.push_str(" Return-value notes refer to the numbered overloads on the shared function-family page.");
        }
    }
    if entry.user_description {
        text.push_str("\n\n_Includes descriptions from your user configuration._");
    }
    if let Some(url) = full_page {
        text.push_str(&format!("\n\n[Open full local documentation]({url})"));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn uri(path: &str) -> String {
        format!("file://{}{path}", if cfg!(windows) { "/C:" } else { "" })
    }

    #[test]
    fn alias_is_selected_by_declaration_range_not_symbol_order() {
        let mut entry = fixtures().remove(0);
        entry.symbol = "std::string".into();
        entry.kind = "alias".into();
        let entries = vec![entry];
        let range = json!({"start":{"line":12,"character":4},"end":{"line":12,"character":10}});
        let location = json!({"uri":uri("/opt/include/string"),"range":range});
        let alias = json!({"name":"string","containerName":"std::","declarationRange":location});
        let underlying = json!({"name":"basic_string","containerName":"std::","declarationRange":{"uri":uri("/opt/include/string"),"range":{"start":{"line":3,"character":4},"end":{"line":3,"character":16}}}});
        let declaration = json!({"result":[location]});
        for symbols in [
            json!({"result":[underlying,alias]}),
            json!({"result":[alias,underlying]}),
        ] {
            assert_eq!(
                resolve(&entries, &symbols, &declaration, &[], &index(&entries))
                    .unwrap()
                    .symbol,
                "std::string"
            );
            assert!(resolve(
                &entries,
                &symbols,
                &json!({"result":null}),
                &[],
                &index(&entries)
            )
            .is_none());
            let roots = [url::Url::parse(&uri("/opt/include/")).unwrap()];
            assert!(resolve(&entries, &symbols, &declaration, &roots, &index(&entries)).is_none());
        }
        assert!(resolve(
            &entries,
            &json!({"result":[alias,alias]}),
            &declaration,
            &[],
            &index(&entries)
        )
        .is_none());
    }

    #[test]
    fn operators_keep_angle_tokens_and_macros_use_definition_locations() {
        assert_eq!(
            canonical("std::__1::vector<int>::operator<=>"),
            "std::vector::operator<=>"
        );
        assert_eq!(
            canonical("std::vector<int>::operator<<"),
            "std::vector::operator<<"
        );
        assert_eq!(
            canonical("std::_Iosb<int>::goodbit"),
            "std::ios_base::goodbit"
        );
        let mut entry = fixtures().remove(0);
        entry.symbol = "EOF".into();
        entry.kind = "macro".into();
        entry.headers = vec!["stdio.h".into()];
        let entries = vec![entry];
        let symbols = json!({"result":[{"name":"EOF","definitionRange":{"uri":uri("/opt/include/stdio.h")}}]});
        assert!(resolve(
            &entries,
            &symbols,
            &json!({"result":null}),
            &[],
            &index(&entries)
        )
        .is_some());
        let symbols = json!({"result":[{"name":"EOF","declarationRange":{"uri":uri("/Program Files (x86)/Windows Kits/10/Include/version/ucrt/stdlib.h")}}]});
        assert!(resolve(
            &entries,
            &symbols,
            &json!({"result":null}),
            &[],
            &index(&entries)
        )
        .is_some());
        let symbols = json!({"result":[{"name":"EOF","declarationRange":{"uri":uri("/custom/sdk/stdlib.h")}}]});
        assert!(resolve(
            &entries,
            &symbols,
            &json!({"result":null}),
            &[],
            &index(&entries)
        )
        .is_none());
    }

    #[test]
    fn resolves_instantiated_standard_member_but_not_project_or_other_container() {
        let entries = fixtures();
        let info = json!({"result":[{"name":"push_back","containerName":"std::__1::vector<int, std::allocator<int>>::"}]});
        let declaration = json!({"result":[{"uri":uri("/opt/sdk/include/stl_vector.h")}]});
        let roots = [url::Url::parse(&uri("/project/")).unwrap()];
        assert_eq!(
            resolve(&entries, &info, &declaration, &roots, &index(&entries))
                .unwrap()
                .symbol,
            "std::vector::push_back"
        );
        assert!(resolve(
            &entries,
            &info,
            &json!({"result":[{"uri":uri("/project/stl_vector.h") }]}),
            &roots,
            &index(&entries)
        )
        .is_none());
        assert!(resolve(
            &entries,
            &json!({"result":[{"name":"push_back","containerName":"UserVector::"}]}),
            &declaration,
            &roots,
            &index(&entries)
        )
        .is_none());
    }

    #[test]
    fn cannot_resolve_by_spelling_or_ambiguous_symbol_alone() {
        let entries = fixtures();
        let info = json!({"result":[{"name":"printf","containerName":""}]});
        assert!(resolve(
            &entries,
            &info,
            &json!({"result":null}),
            &[],
            &index(&entries)
        )
        .is_none());
        assert!(resolve(
            &entries,
            &json!({"result":[{},{}]}),
            &json!({"result":null}),
            &[],
            &index(&entries)
        )
        .is_none());
    }

    #[test]
    fn standard_aliases_resolve_in_multiarch_headers_but_not_project_headers() {
        let entries = vec![Entry {
            symbol: "std::size_t".into(),
            headers: vec!["cstddef".into()],
            kind: "alias".into(),
            ..Entry::default()
        }];
        let info = json!({"result":[{"name":"size_t","containerName":"std::"}]});
        let roots = [url::Url::parse(&uri("/project/")).unwrap()];
        for (path, expected) in [
            ("/usr/include/c++/15/bits/c++config.h", true),
            (
                "/usr/include/x86_64-linux-gnu/c++/15/bits/c++config.h",
                true,
            ),
            (
                "/usr/include/aarch64-linux-gnu/c++/15/bits/c++config.h",
                true,
            ),
            (
                "/project/include/x86_64-linux-gnu/c++/15/bits/c++config.h",
                false,
            ),
            ("/custom/sdk/c++config.h", false),
        ] {
            let declaration = json!({"result":[{"uri":uri(path)}]});
            assert_eq!(
                resolve(&entries, &info, &declaration, &roots, &index(&entries)).is_some(),
                expected,
                "{path}"
            );
        }
    }

    #[test]
    fn overloaded_nonmember_pages_require_an_unambiguous_header_match() {
        let mut vector = fixtures().remove(0);
        vector.symbol = "std::operator==".into();
        vector.headers = vec!["vector".into()];
        let mut array = vector.clone();
        array.headers = vec!["array".into()];
        let entries = vec![array, vector];
        let symbols = json!({"result":[{"name":"operator==","containerName":"std::"}]});
        let declaration =
            json!({"result":[{"uri":uri("/toolchain/VC/Tools/MSVC/version/include/vector")}]});
        assert_eq!(
            resolve(&entries, &symbols, &declaration, &[], &index(&entries))
                .unwrap()
                .headers,
            ["vector"]
        );
        let declaration =
            json!({"result":[{"uri":uri("/toolchain/VC/Tools/MSVC/version/include/xutility")}]});
        assert!(resolve(&entries, &symbols, &declaration, &[], &index(&entries)).is_none());
    }
}

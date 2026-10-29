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

#[cfg(test)]
fn resolve<'a>(
    entries: &'a [Entry],
    symbols: &Value,
    declaration: &Value,
    roots: &[url::Url],
    index: &HashMap<String, Vec<usize>>,
) -> Option<&'a Entry> {
    resolve_with_hover(entries, symbols, declaration, &Value::Null, roots, index)
}

fn msvc_map_owner(hover: &Value) -> Option<&'static str> {
    let text = hover.pointer("/result/contents/value")?.as_str()?;
    let mut owners = text
        .lines()
        .filter_map(|line| line.strip_prefix("// In _Tree<std::_Tmap_traits<"));
    let traits = owners.next()?;
    if owners.next().is_some() {
        return None;
    }
    let mut depth = 0_u32;
    let mut last = 0;
    for (offset, character) in traits.char_indices() {
        match character {
            '<' => depth += 1,
            '>' if depth > 0 => depth -= 1,
            ',' if depth == 0 => last = offset + 1,
            '>' if depth == 0 && traits[offset..].trim() == ">>" => {
                return match traits[last..offset].trim() {
                    "false" => Some("std::map"),
                    "true" => Some("std::multimap"),
                    _ => None,
                };
            }
            '>' => return None,
            _ => {}
        }
    }
    None
}

fn c_runtime_header(entry: &Entry, header: &str, path: &str) -> bool {
    if !entry
        .source_url
        .starts_with("https://en.cppreference.com/w/c/")
        || entry.symbol.contains("::")
    {
        return false;
    }
    let public_header = if path.contains("/vc/tools/msvc/")
        || (path.contains("/windows kits/") && path.contains("/ucrt/"))
    {
        match header {
            "corecrt_malloc.h" | "corecrt_search.h" => "stdlib.h",
            "vcruntime_string.h" | "corecrt_memory.h" => "string.h",
            "corecrt_wstdio.h" => "stdio.h",
            "corecrt_wtime.h" => "time.h",
            _ => return false,
        }
    } else if path.contains("/usr/include/") {
        match header {
            "FILE.h" | "__FILE.h" if path.contains("/bits/types/") => "stdio.h",
            "struct_tm.h" if path.contains("/bits/types/") => "time.h",
            _ => return false,
        }
    } else {
        return false;
    };
    entry
        .headers
        .iter()
        .any(|candidate| candidate == public_header)
}

pub fn resolve_with_hover<'a>(
    entries: &'a [Entry],
    symbols: &Value,
    declaration: &Value,
    hover: &Value,
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
    let mut qualified = if !container.is_empty() && !container.ends_with("::") {
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
    // MSVC shares a tree implementation between map and multimap. The instantiated
    // traits in clangd's hover distinguish their different key-count semantics.
    if standard_cpp && header == "xtree" && canonical(container) == "std::_Tree::" {
        let owner = msvc_map_owner(hover)?;
        qualified = format!("{owner}::{name}");
    }
    let matches: Vec<_> = index
        .get(&qualified)?
        .iter()
        .map(|number| &entries[*number])
        .filter(|entry| {
            entry.symbol == qualified
                && (standard_cpp
                    || c_runtime_header(entry, header, &normalized_path)
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
    } else if candidates.iter().all(|entry| {
        entry.kind == "function"
            && entry
                .source_url
                .starts_with("https://en.cppreference.com/w/c/")
    }) {
        // Some C functions appear on both their own page and a shared family page.
        let mut dedicated = candidates
            .iter()
            .copied()
            .filter(|entry| entry.source_url.rsplit('/').next() == Some(name));
        let entry = dedicated.next()?;
        dedicated.next().is_none().then_some(entry)
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
    fn private_c_runtime_headers_require_a_known_public_header_and_sdk_path() {
        for (symbol, public, private, source) in [
            ("malloc", "stdlib.h", "corecrt_malloc.h", "memory/malloc"),
            (
                "memcpy",
                "string.h",
                "vcruntime_string.h",
                "string/byte/memcpy",
            ),
            ("qsort", "stdlib.h", "corecrt_search.h", "algorithm/qsort"),
            ("FILE", "stdio.h", "corecrt_wstdio.h", "io/FILE"),
            ("tm", "time.h", "corecrt_wtime.h", "chrono/tm"),
        ] {
            let entries = vec![Entry {
                symbol: symbol.into(),
                headers: vec![public.into()],
                source_url: format!("https://en.cppreference.com/w/c/{source}"),
                ..Entry::default()
            }];
            let info = json!({"result":[{"name":symbol}]});
            for prefix in [
                "/tools/VC/Tools/MSVC/version/include/",
                "/Program Files (x86)/Windows Kits/10/Include/version/ucrt/",
            ] {
                let declaration = json!({"result":[{"uri":uri(&format!("{prefix}{private}"))}]});
                assert!(resolve(&entries, &info, &declaration, &[], &index(&entries)).is_some());
                let roots = [url::Url::parse(&uri(prefix)).unwrap()];
                assert!(resolve(&entries, &info, &declaration, &roots, &index(&entries)).is_none());
            }
            let declaration = json!({"result":[{"uri":uri(&format!("/custom/sdk/{private}"))}]});
            assert!(resolve(&entries, &info, &declaration, &[], &index(&entries)).is_none());
        }
        let entry = Entry {
            symbol: "printf".into(),
            headers: vec!["stdio.h".into()],
            source_url: "https://en.cppreference.com/w/c/io/fprintf".into(),
            ..Entry::default()
        };
        assert!(!c_runtime_header(
            &entry,
            "corecrt_malloc.h",
            "/tools/vc/tools/msvc/version/include/corecrt_malloc.h"
        ));
    }

    #[test]
    fn inherited_msvc_tree_members_need_unambiguous_instantiated_map_traits() {
        let entries: Vec<_> = ["std::map::count", "std::multimap::count"]
            .into_iter()
            .map(|symbol| Entry {
                symbol: symbol.into(),
                headers: vec!["map".into()],
                ..Entry::default()
            })
            .collect();
        let info = json!({"result":[{"name":"count","containerName":"std::_Tree::"}]});
        let declaration =
            json!({"result":[{"uri":uri("/tools/VC/Tools/MSVC/version/include/xtree")}]});
        for (flag, expected) in [
            ("false", "std::map::count"),
            ("true", "std::multimap::count"),
        ] {
            let hover = json!({"result":{"contents":{"value":format!("```cpp\n// In _Tree<std::_Tmap_traits<int, int, std::less<int>, std::allocator<std::pair<const int, int>>, {flag}>>\npublic: size_type count(const key_type&) const\n```")}}});
            assert_eq!(
                resolve_with_hover(&entries, &info, &declaration, &hover, &[], &index(&entries))
                    .unwrap()
                    .symbol,
                expected
            );
            let roots = [url::Url::parse(&uri("/tools/")).unwrap()];
            assert!(resolve_with_hover(
                &entries,
                &info,
                &declaration,
                &hover,
                &roots,
                &index(&entries)
            )
            .is_none());
        }
        assert!(resolve(&entries, &info, &declaration, &[], &index(&entries)).is_none());
        for text in [
            "// In _Tree<std::_Tmap_traits<int, int, Less, Alloc, Unknown>>",
            "// In _Tree<std::_Tset_traits<int, Less, Alloc, false>>",
            "// In _Tree<std::_Tmap_traits<int, int, Less, Alloc, false>>\n// In _Tree<std::_Tmap_traits<int, int, Less, Alloc, true>>",
        ] {
            assert!(msvc_map_owner(&json!({"result":{"contents":{"value":text}}})).is_none());
        }
    }

    #[test]
    fn c_function_family_aliases_prefer_the_unique_dedicated_page() {
        let entries: Vec<_> = ["putc", "fputc"]
            .into_iter()
            .map(|page| Entry {
                symbol: "fputc".into(),
                kind: "function".into(),
                headers: vec!["stdio.h".into()],
                source_url: format!("https://en.cppreference.com/w/c/io/{page}"),
                ..Entry::default()
            })
            .collect();
        let info = json!({"result":[{"name":"fputc"}]});
        let declaration = json!({"result":[{"uri":uri("/usr/include/stdio.h")}]});
        assert!(
            resolve(&entries, &info, &declaration, &[], &index(&entries))
                .unwrap()
                .source_url
                .ends_with("/fputc")
        );
        let duplicates = vec![entries[1].clone(), entries[1].clone()];
        assert!(resolve(&duplicates, &info, &declaration, &[], &index(&duplicates)).is_none());
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

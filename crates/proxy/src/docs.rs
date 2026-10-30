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
        "std::_V2::",
    ] {
        result = result.replace(abi, "std::");
    }
    // MSVC declares the public ios_base flags in its internal base class.
    result = result.replace("std::_Iosb::", "std::ios_base::");
    result = result.replace("std::filesystem::__cxx11::", "std::filesystem::");
    result = result.replace("std::chrono::_V2::", "std::chrono::");
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

fn msvc_container_owner(hover: &Value) -> Option<&'static str> {
    let text = hover.pointer("/result/contents/value")?.as_str()?;
    let mut owners = text.lines().filter_map(|line| {
        [
            (
                "// In _Tree<std::_Tmap_traits<",
                "std::map",
                "std::multimap",
            ),
            (
                "// In _Tree<std::_Tset_traits<",
                "std::set",
                "std::multiset",
            ),
            (
                "// In _Hash<std::_Umap_traits<",
                "std::unordered_map",
                "std::unordered_multimap",
            ),
            (
                "// In _Hash<std::_Uset_traits<",
                "std::unordered_set",
                "std::unordered_multiset",
            ),
        ]
        .into_iter()
        .find_map(|(prefix, single, multiple)| {
            line.strip_prefix(prefix)
                .map(|traits| (traits, single, multiple))
        })
    });
    let (traits, single, multiple) = owners.next()?;
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
                    "false" => Some(single),
                    "true" => Some(multiple),
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
    let c_entry = entry
        .source_url
        .starts_with("https://en.cppreference.com/w/c/")
        && !entry.symbol.contains("::");
    let cpp_entry = entry
        .source_url
        .starts_with("https://en.cppreference.com/w/cpp/")
        && entry.symbol.starts_with("std::");
    if !c_entry && !cpp_entry {
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
            "corecrt_math.h" => "math.h",
            "corecrt.h" if entry.symbol.rsplit("::").next() == Some("time_t") => "time.h",
            "vcruntime.h" => "stddef.h",
            _ => return false,
        }
    } else if path.contains("/usr/include/") {
        match header {
            "FILE.h" | "__FILE.h" if path.contains("/bits/types/") => "stdio.h",
            "struct_tm.h" if path.contains("/bits/types/") => "time.h",
            "time_t.h" | "clock_t.h" if path.contains("/bits/types/") => "time.h",
            "mathcalls.h" | "mathcalls-helper-functions.h" if path.contains("/bits/") => "math.h",
            "errno-base.h" | "errno.h" if path.contains("/asm-generic/") => "errno.h",
            _ => return false,
        }
    } else if path.contains("/lib/clang/") && path.contains("/include/") {
        match header {
            "__stddef_null.h" | "__stddef_offsetof.h" | "__stddef_size_t.h" => "stddef.h",
            _ => return false,
        }
    } else {
        return false;
    };
    entry.headers.iter().any(|candidate| {
        candidate == public_header
            || (cpp_entry && crate::notes::cpp_c_header(public_header) == Some(candidate.as_str()))
    })
}

fn provided_header(hover: &Value) -> Option<&str> {
    let text = hover.pointer("/result/contents/value")?.as_str()?;
    let mut headers = text.lines().filter_map(|line| {
        line.trim()
            .strip_prefix("provided by `<")
            .or_else(|| line.trim().strip_prefix("Provided by `<"))?
            .strip_suffix(">`")
    });
    let header = headers.next()?;
    headers.next().is_none().then_some(header)
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
    // GCC's C compatibility headers can expose both a using declaration and
    // the underlying C declaration. Every target must resolve to the same page.
    if let Some(locations) = declaration
        .get("result")
        .and_then(Value::as_array)
        .filter(|items| items.len() > 1)
    {
        let mut resolved: Option<&Entry> = None;
        for location in locations {
            let entry = resolve_with_hover(
                entries,
                &serde_json::json!({"result":symbols}),
                &serde_json::json!({"result":[location]}),
                hover,
                roots,
                index,
            )?;
            if resolved.is_some_and(|previous| {
                previous.symbol != entry.symbol || previous.source_url != entry.source_url
            }) {
                return None;
            }
            resolved = Some(entry);
        }
        return resolved;
    }
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
        let info = matching.next().or_else(|| {
            let text = hover.pointer("/result/contents/value")?.as_str()?;
            let mut macros = symbols.iter().filter(|info| {
                let Some(name) = info.get("name").and_then(Value::as_str) else {
                    return false;
                };
                info.get("usr")
                    .and_then(Value::as_str)
                    .is_some_and(|usr| usr.ends_with(&format!("@macro@{name}")))
                    && text.starts_with(&format!("### macro `{name}`"))
            });
            let info = macros.next()?;
            macros.next().is_none().then_some(info)
        })?;
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
    // MSVC shares tree and hash implementations across public containers.
    // clangd's instantiated traits identify the public owner and duplicate-key rules.
    if standard_cpp
        && ((header == "xtree" && canonical(container) == "std::_Tree::")
            || (header == "xhash" && canonical(container) == "std::_Hash::"))
    {
        let owner = msvc_container_owner(hover)?;
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
                    || (entry.symbol.starts_with("std::")
                        && crate::notes::cpp_c_header(header).is_some_and(|cpp_header| {
                            entry
                                .headers
                                .iter()
                                .any(|candidate| candidate == cpp_header)
                        }))
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
    let mut candidates = if exact.is_empty() { preferred } else { exact };
    if candidates.len() > 1 && qualified.starts_with("std::vector::") {
        if let Some(instance) = hover
            .pointer("/result/contents/value")
            .and_then(Value::as_str)
            .and_then(|text| {
                text.lines()
                    .find_map(|line| line.strip_prefix("// In vector<"))
            })
        {
            let boolean = instance.starts_with("bool>") || instance.starts_with("bool,");
            candidates.retain(|entry| entry.source_url.contains("/vector_bool/") == boolean);
        }
    }
    if candidates.len() > 1 {
        if let Some(public_header) = provided_header(hover) {
            let public: Vec<_> = candidates
                .iter()
                .copied()
                .filter(|entry| {
                    entry
                        .headers
                        .iter()
                        .any(|candidate| candidate == public_header)
                })
                .collect();
            if !public.is_empty() {
                candidates = public;
            }
        }
    }
    if candidates.len() == 1 || candidates.iter().all(|entry| entry.kind == "macro") {
        candidates.first().copied()
    } else if candidates.iter().all(|entry| {
        entry.kind == "function"
            && (entry
                .source_url
                .starts_with("https://en.cppreference.com/w/c/")
                || entry.headers.iter().any(|header| {
                    ["cstdio", "cstdlib", "cstring", "cmath", "cctype", "ctime"]
                        .contains(&header.as_str())
                }))
    }) {
        // C++ abs has separate integer and floating-point reference families.
        // A shared public header alone cannot identify the selected overload.
        if qualified == "std::abs"
            && candidates
                .iter()
                .any(|entry| entry.source_url.ends_with("/fabs"))
        {
            return None;
        }
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
            "// In _Tree<std::_Tset_traits<int, Less, Alloc, Unknown>>",
            "// In _Tree<std::_Tmap_traits<int, int, Less, Alloc, false>>\n// In _Tree<std::_Tmap_traits<int, int, Less, Alloc, true>>",
        ] {
            assert!(msvc_container_owner(&json!({"result":{"contents":{"value":text}}})).is_none());
        }
    }

    #[test]
    fn msvc_traits_keep_ordered_and_hash_container_owners_distinct() {
        for (text, expected) in [
            (
                "// In _Tree<std::_Tset_traits<int, Less, Alloc, false>>",
                "std::set",
            ),
            (
                "// In _Tree<std::_Tset_traits<int, Less, Alloc, true>>",
                "std::multiset",
            ),
            (
                "// In _Hash<std::_Umap_traits<int, int, Hash, Equal, Alloc, false>>",
                "std::unordered_map",
            ),
            (
                "// In _Hash<std::_Uset_traits<int, Hash, Equal, Alloc, true>>",
                "std::unordered_multiset",
            ),
        ] {
            let hover = json!({"result":{"contents":{"value":text}}});
            assert_eq!(msvc_container_owner(&hover), Some(expected));
        }
    }

    #[test]
    fn compatibility_declarations_must_all_resolve_outside_the_project() {
        let entries = vec![Entry {
            symbol: "malloc".into(),
            headers: vec!["stdlib.h".into()],
            ..Entry::default()
        }];
        let range = json!({"start":{"line":1,"character":0},"end":{"line":1,"character":6}});
        let locations = json!([
            {"uri":uri("/usr/include/c++/15/stdlib.h"),"range":range},
            {"uri":uri("/usr/include/stdlib.h"),"range":range}
        ]);
        let symbols = json!({"result":[
            {"name":"malloc","declarationRange":locations[0]},
            {"name":"malloc","declarationRange":locations[1]}
        ]});
        let declaration = json!({"result":locations});
        assert_eq!(
            resolve(&entries, &symbols, &declaration, &[], &index(&entries))
                .unwrap()
                .symbol,
            "malloc"
        );
        let roots = [url::Url::parse(&uri("/usr/include/c++/15/")).unwrap()];
        assert!(resolve(&entries, &symbols, &declaration, &roots, &index(&entries)).is_none());
    }

    #[test]
    fn macro_hover_uses_the_definition_instead_of_the_expanded_function() {
        let entries = vec![Entry {
            symbol: "stdin".into(),
            kind: "macro".into(),
            headers: vec!["stdio.h".into()],
            ..Entry::default()
        }];
        let symbols = json!({"result":[{"name":"fgetc"},{"name":"stdin","usr":"c:@macro@stdin"}]});
        let declaration = json!({"result":[{"uri":uri("/usr/include/stdio.h"),"range":{}}]});
        let hover = json!({"result":{"contents":{"value":"### macro `stdin`  \nprovided by `<stdio.h>`  "}}});
        assert_eq!(provided_header(&hover), Some("stdio.h"));
        assert_eq!(
            resolve_with_hover(
                &entries,
                &symbols,
                &declaration,
                &hover,
                &[],
                &index(&entries)
            )
            .unwrap()
            .symbol,
            "stdin"
        );
        assert!(resolve(&entries, &symbols, &declaration, &[], &index(&entries)).is_none());
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

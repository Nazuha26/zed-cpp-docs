use crate::docs::Entry;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
    symbol: String,
    summary: Option<String>,
    signature: Option<String>,
    parameters: Option<String>,
    returns: Option<String>,
    template_parameters: Option<String>,
    members: Option<String>,
    example: Option<String>,
    #[serde(skip)]
    reference: Option<String>,
}

impl Note {
    fn fields(&self) -> [&Option<String>; 7] {
        [
            &self.summary,
            &self.signature,
            &self.parameters,
            &self.returns,
            &self.template_parameters,
            &self.members,
            &self.example,
        ]
    }
}

fn parse(text: &str) -> io::Result<Vec<Note>> {
    let notes: Vec<Note> = serde_json::from_str(text).map_err(io::Error::other)?;
    let mut symbols = HashSet::new();
    for note in &notes {
        if note.symbol.trim().is_empty() || note.symbol.trim() != note.symbol {
            return Err(io::Error::other(
                "Each description needs a nonempty symbol without surrounding whitespace",
            ));
        }
        if !symbols.insert(&note.symbol) {
            return Err(io::Error::other(format!(
                "Duplicate description for {}",
                note.symbol
            )));
        }
        if note.fields().iter().all(|field| field.is_none()) {
            return Err(io::Error::other(format!(
                "Description for {} has no text fields",
                note.symbol
            )));
        }
    }
    Ok(notes)
}

pub fn bundled() -> Vec<Note> {
    #[derive(Deserialize)]
    struct Reference {
        symbol: String,
        cppreference: String,
    }
    let references: Vec<Reference> =
        serde_json::from_str(include_str!("../../../data/compact-docs-sources.json"))
            .expect("Bundled description references are valid");
    let reference_count = references.len();
    let references: HashMap<_, _> = references
        .into_iter()
        .map(|reference| (reference.symbol, reference.cppreference))
        .collect();
    assert_eq!(
        references.len(),
        reference_count,
        "Duplicate description references"
    );
    let mut notes = parse(include_str!("../../../data/compact-docs.json"))
        .expect("Bundled compact descriptions are valid");
    for note in &mut notes {
        note.reference = Some(
            references
                .get(&note.symbol)
                .expect("Each bundled description has a reference")
                .clone(),
        );
    }
    notes
}

pub fn cpp_c_header(header: &str) -> Option<&'static str> {
    match header {
        "stdio.h" => Some("cstdio"),
        "stdlib.h" => Some("cstdlib"),
        "string.h" => Some("cstring"),
        "math.h" => Some("cmath"),
        "ctype.h" => Some("cctype"),
        "time.h" => Some("ctime"),
        "stddef.h" => Some("cstddef"),
        "stdint.h" => Some("cstdint"),
        "limits.h" => Some("climits"),
        "errno.h" => Some("cerrno"),
        _ => None,
    }
}

pub fn default_path() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from)
    };
    base.unwrap_or_else(|| {
        PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| ".".into())).join(".config")
    })
    .join("zed-cpp-docs/descriptions.json")
}

pub fn load(path: &Path) -> io::Result<Vec<Note>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    const LIMIT: u64 = 2 * 1024 * 1024;
    let mut text = String::new();
    file.take(LIMIT + 1).read_to_string(&mut text)?;
    if text.len() as u64 > LIMIT {
        return Err(io::Error::other("User descriptions exceed the 2 MiB limit"));
    }
    parse(&text)
}

pub fn apply(entries: &mut [Entry], notes: &[Note], user: bool) {
    let index = crate::docs::index(entries);
    for note in notes {
        let Some(original_matches) = index.get(&note.symbol) else {
            continue;
        };
        let mut matches = original_matches.clone();
        if !user
            && note
                .reference
                .as_deref()
                .is_some_and(|url| url.starts_with("https://en.cppreference.com/w/c/"))
        {
            let headers: HashSet<_> = original_matches
                .iter()
                .map(|number| &entries[*number])
                .filter(|entry| Some(&entry.source_url) == note.reference.as_ref())
                .flat_map(|entry| {
                    entry
                        .headers
                        .iter()
                        .filter_map(|header| cpp_c_header(header))
                })
                .collect();
            if let Some(aliases) = index.get(&format!("std::{}", note.symbol)) {
                matches.extend(aliases.iter().copied().filter(|number| {
                    let entry = &entries[*number];
                    let page = entry.source_url.rsplit('/').next();
                    let family = note
                        .reference
                        .as_deref()
                        .and_then(|url| url.rsplit('/').next());
                    entry
                        .source_url
                        .starts_with("https://en.cppreference.com/w/cpp/")
                        && (!note
                            .reference
                            .as_deref()
                            .is_some_and(|url| url.contains("/numeric/math/"))
                            || page == family
                            || page == Some(note.symbol.as_str()))
                        && entry
                            .headers
                            .iter()
                            .any(|header| headers.contains(header.as_str()))
                }));
            }
        }
        for number in matches {
            let entry = &mut entries[number];
            if !user
                && entry.symbol == note.symbol
                && !entry.source_url.is_empty()
                && note
                    .reference
                    .as_ref()
                    .is_some_and(|reference| reference != &entry.source_url)
            {
                continue;
            }
            for (field, replacement) in [
                (&mut entry.summary, &note.summary),
                (&mut entry.signature, &note.signature),
                (&mut entry.parameters, &note.parameters),
                (&mut entry.returns, &note.returns),
                (&mut entry.template_parameters, &note.template_parameters),
                (&mut entry.members, &note.members),
                (&mut entry.example, &note.example),
            ] {
                if let Some(value) = replacement {
                    field.clone_from(value);
                }
            }
            if user {
                entry.user_description = true;
            } else {
                entry.editorial = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_notes_follow_the_public_header_without_replacing_cpp_overloads() {
        let mut entries: Vec<_> = [
            (
                "remove",
                "stdio.h",
                "https://en.cppreference.com/w/c/io/remove",
            ),
            (
                "std::remove",
                "cstdio",
                "https://en.cppreference.com/w/cpp/io/c/remove",
            ),
            (
                "std::remove",
                "algorithm",
                "https://en.cppreference.com/w/cpp/algorithm/remove",
            ),
        ]
        .into_iter()
        .map(|(symbol, header, url)| Entry {
            symbol: symbol.into(),
            headers: vec![header.into()],
            source_url: url.into(),
            summary: "Imported text".into(),
            ..Entry::default()
        })
        .collect();
        let mut notes = parse(r#"[{"symbol":"remove","summary":"Delete one file"}]"#).unwrap();
        notes[0].reference = Some(entries[0].source_url.clone());
        apply(&mut entries, &notes, false);
        assert_eq!(entries[0].summary, "Delete one file");
        assert_eq!(entries[1].summary, "Delete one file");
        assert_eq!(entries[2].summary, "Imported text");
        assert!(!entries[2].editorial);
    }

    #[test]
    fn user_fields_override_bundled_fields_and_preserve_source_metadata() {
        let mut entries = vec![Entry {
            symbol: "std::getline".into(),
            source: "cppreference".into(),
            headers: vec!["string".into()],
            summary: "Extracted summary".into(),
            template_parameters: "Extracted template parameters".into(),
            html_path: "reference/en/cpp/string/basic_string/getline.html".into(),
            ..Entry::default()
        }];
        apply(&mut entries, &bundled(), false);
        assert!(entries[0].summary.starts_with("Read characters"));
        let signature = entries[0].signature.clone();
        let user = parse(r#"[{"symbol":"std::getline","summary":"My description","example":""}]"#)
            .unwrap();
        apply(&mut entries, &user, true);
        let entry = &entries[0];
        assert_eq!(entry.summary, "My description");
        assert_eq!(entry.signature, signature);
        assert_eq!(entry.template_parameters, "Extracted template parameters");
        assert!(entry.example.is_empty());
        assert_eq!(entry.headers, ["string"]);
        assert!(entry.html_path.ends_with("getline.html"));
        assert!(entry.editorial && entry.user_description);
    }

    #[test]
    fn classes_and_new_bundled_symbols_need_no_code_allowlist() {
        let mut entries = vec![Entry {
            symbol: "std::vector".into(),
            kind: "type".into(),
            summary: "Extracted class".into(),
            ..Entry::default()
        }];
        let notes = parse(r#"[{"symbol":"std::vector","members":"Compact member list"}]"#).unwrap();
        apply(&mut entries, &notes, false);
        assert_eq!(entries[0].summary, "Extracted class");
        assert_eq!(entries[0].members, "Compact member list");
        assert_eq!(entries[0].kind, "type");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn invalid_descriptions_are_rejected_before_any_fields_are_applied() {
        for text in [
            r#"[{"symbol":"printf","summary":"One"},{"symbol":"printf","summary":"Two"}]"#,
            r#"[{"symbol":"printf","summery":"Typo"}]"#,
            r#"[{"symbol":"printf"}]"#,
            r#"[{"symbol":" ","summary":"Empty name"}]"#,
            r#"[{"symbol":"printf","html_path":"outside.html"}]"#,
        ] {
            assert!(parse(text).is_err(), "{text}");
        }
        assert!(parse("[]").unwrap().is_empty());
    }
}

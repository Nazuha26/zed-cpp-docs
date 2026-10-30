# Compact documentation

`compact-docs.json` contains 341 original, concise descriptions: 215 C++ symbols
and 126 C library symbols. Coverage follows common tasks rather than a fixed
entry limit: containers and adaptors, strings, algorithms, iterators, streams,
C++17 filesystem operations, formatted and file I/O, memory, byte strings,
conversions, mathematics, time, types, constants, public fields and macros.

Browse the [complete symbol list](SYMBOLS.md) with a short description and
reference link for each entry.

Each description has a complete runnable example. C++ examples target C++17.
All C library examples also compile and run as C11. Signatures show representative
overloads; version differences and important preconditions are stated in the
text. Template parameter names in signatures are placeholders, not additional
types that a user must declare.

## Sources

[`compact-docs-sources.json`](compact-docs-sources.json) records, for every symbol:

- `cppreference`: the matching page in the downloaded cppreference HTML book
  dated 2025-02-09. These URLs are checked against the imported index.
- `standard`: a C++ working-draft section or a section of the C11 committee draft
  N1570, used to check behavior, return values and preconditions.
- `language`: `cpp` for C++ library symbols, or `c` for the C library subset.

The [C++ working draft](https://eel.is/c++draft/) includes newer facilities and
overloads. The compact examples deliberately stay within C++17; the tooltip
calls out relevant version differences, such as the return type of
`vector::emplace_back` and mutable `basic_string::data`.
The [C11 draft text](https://port70.net/~nsz/c/c11/n1570.html) supplies the C
baseline. C++ const-preserving overloads of C string search functions are
identified separately in their descriptions.

The source manifest also scopes bundled descriptions to their matching reference
pages. C descriptions can be shared with the corresponding C++ C-library wrapper
headers without changing unrelated overloads (for example, the range algorithm
`std::remove`). Runtime descriptions retain the imported page, headers and attribution, so the full documentation
still opens locally and works offline. Content licenses are listed in
[`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md).

## Verification

```sh
python tests/examples.py clang++ --c-compiler clang --run
python tests/catalog.py target/release/zed-cpp-docs-proxy clangd /path/to/local/docs
```

The first command compiles and runs every example as C++17 and the C library
examples again as C11. The second checks every description against the imported
index and requests each tooltip through a real clangd process. It verifies the
curated text, attribution and local documentation link. User overrides and
unmodified cppreference fallbacks are tested separately in `tests/descriptions.py`.

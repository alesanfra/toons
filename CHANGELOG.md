# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [0.8.0] - 2026-09-17

### Added

- **TOON specification v4.1 support**, validated against the conformance
  fixtures of [spec tag v4.1.1](https://github.com/toon-format/spec/tree/v4.1.1/tests):
  - `#` comment lines, removed in a lexical pre-pass before every other
    rule, so a comment never ends a scope or counts as a row.
  - Nested field groups in tabular headers
    (`orders[2]{id,customer{name,country},total}:`), encoded and decoded at
    any depth.
  - Keyed tabular form for objects of uniform objects
    (`servers[2:]{host,port}:`), including the keyless root form and the
    header-on-hyphen-line form inside list items.
  - The canonical empty-array forms `key: []` and `[]`; the legacy
    `key[0]:` and `[0]:` are still decoded.
  - The normative decoder number grammar: `.5`, `1.`, `+5`, `05`,
    `Infinity`, `NaN`, `0x10`, and `1_000` decode as strings.
  - `\uXXXX` escapes in quoted strings and keys, emitted for control
    characters and rejected for lone surrogates.
  - Byte-order-mark removal, CRLF input, and trailing-space stripping.
- Strict-mode checks from Sections 12 and 14: duplicate sibling keys,
  indentation depth jumps, over-indented lines, blank lines inside a header
  span, trailing content after a root array, malformed bracket segments and
  field lists, header delimiter mismatches, keyless headers outside the
  document root, and scope width and count mismatches.
- Non-strict mode applies last-write-wins for duplicate keys and never lets
  a declared `[N]` truncate a scope.
- Tuples are encoded as TOON arrays, matching the `json` module.
- Integers outside the 64-bit range keep their exact value in both
  directions; previously they were silently converted to floats.
- `toons.pyi` declares `__version__`.
- `toons.__toon_spec__` reports the implemented TOON specification version
  (`"4.1"`), as recommended by spec Section 13.

### Changed

- The `indent` option of `loads`, `load`, `dumps`, and `dump` is now spelled
  `indent_size`, matching the spec's `indentSize`. `indent` stays accepted as
  a deprecated alias; passing both with different values raises `ValueError`.
  `to_json` keeps `indent` for the JSON output, as before, and gains
  `indent_size` for the TOON input.
- **Breaking:** the decoder no longer detects the indentation from the
  input; it defaults to 2 spaces, as spec Section 13 requires.
- **Breaking:** the encoder no longer chooses a form by preference. Tabular
  form is mandatory wherever detection succeeds, keyed tabular form applies
  in object-field and root positions, a list-item object's first field sits
  on the hyphen line, and empty arrays emit `key: []`.
- **Breaking:** Python 3.8 is now the minimum. PyO3 0.29 no longer offers the
  `abi3-py37` target, so wheels are tagged `cp38-abi3`. Python 3.7 users stay
  on 0.7.0.
- Updated PyO3 from 0.28 to 0.29, refreshed the development and
  documentation dependencies, and bumped the GitHub Actions used by CI.
- **Breaking:** values with no TOON representation (sets, generators, custom
  objects, functions) now raise `TypeError` instead of being encoded as
  `null`, and non-string object keys raise `TypeError` instead of crashing
  the interpreter.
- Invalid option values raise `ValueError` instead of being ignored:
  `delimiter` must be `","`, `"\t"`, or `"|"`; the encoder `indent_size`
  must be at least 2 and the decoder `indent_size` at least 1.
- Out-of-domain numbers have a documented policy: integers stay exact, a
  decimal or exponent token beyond double precision is rejected in strict
  mode and decodes as an infinity when `strict=False`.
- Strings are quoted when they start with `#`, when they are numeric-like
  with a leading `+`, and no longer merely because they start or end with
  non-ASCII whitespace.
- Release builds enable link-time optimization and strip symbols.
- Read the Docs installs the documentation dependencies with `uv sync`
  against `pyproject.toml` and `uv.lock`, so the exported
  `docs/requirements.txt` is gone.
- Publishing to PyPI runs in its own `pypi` environment, so a tagged release
  shows up under the repository's Deployments and can be gated there.

### Removed

- **Breaking:** the `key_folding` and `flatten_depth` encoder options and the
  `expand_paths` decoder option. Key folding and path expansion were removed
  from the specification in v4.0; dotted keys are single literal keys. Data
  encoded with `key_folding="safe"` must be re-hydrated with a v3 decoder
  using `expand_paths="safe"` before re-encoding.

### Fixed

- Encoding a structure that contains a reference cycle raised a stack
  overflow that killed the interpreter; it now raises `ValueError`. Nesting
  deeper than 1000 containers raises `ValueError` for the same reason.
- `dumps(..., delimiter="")` panicked; a panic also escaped for non-string
  object keys.
- An array nested inside an array under an object key was written with a
  stray trailing space and a misplaced header, producing output that could
  not be decoded again.
- Decoding a document nested more than a thousand levels deep overflowed the
  stack and killed the interpreter; it now raises `ToonDecodeError`. The same
  bound applies to the encoder's form detection, which crashed on a deeply
  nested uniform column.
- Encoding a `str` that holds an unpaired surrogate raised a misleading
  `TypeError` about the type; it now raises `ValueError` naming the cause.
- The Read the Docs build failed because the crate used `let` chains, which
  need a newer compiler than the Rust toolchain pinned in
  `.readthedocs.yaml`. The pin now tracks `latest`, and `rust-version` in
  `Cargo.toml` states the 1.88 the crate actually needs.

## [0.7.0] - 2026-05-21

### Added

- `to_json()` convenience API.

### Changed

- Binary wheels are no longer built for Linux x86, s390x, ppc64le, armv7l,
  and 32-bit Windows.

## [0.6.0] - 2026-05-15

### Added

- `ToonDecodeError` with `.line` and `.source` attributes.

### Fixed

- Quoted strings containing colons are parsed as primitives in expanded
  arrays.

## Earlier releases

See the [commit history](https://github.com/alesanfra/toons/commits/main) for
releases before 0.6.0.

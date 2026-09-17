# AGENTS.md

Working notes for coding agents (and humans) touching this repository.

## What this project is

`toons` is a Python extension module written in Rust with [PyO3](https://pyo3.rs/)
and built by [maturin](https://www.maturin.rs/). It encodes and decodes
[TOON](https://github.com/toon-format/spec) (Token Oriented Object Notation)
and exposes a `json`-like API: `loads`, `load`, `dumps`, `dump`, `to_json`,
plus the `ToonDecodeError` exception.

**Target specification: TOON v3.0**, exposed at runtime as
`toons.__toon_spec__` and pinned to the conformance fixtures of spec tag
v3.0.1. Changing that constant means re-vendoring the fixtures and updating
`README.md`, `docs/index.md`, and this file; `tests/integration/test_smoke.py`
asserts its value so the change cannot pass unnoticed. Upstream is at v4.1; comment lines, nested field groups,
keyed tabular form, `key: []` empty arrays, and the `indentSize` rename are
v4 features and are deliberately absent. Do not implement one of them
piecemeal: a v4 upgrade means re-vendoring the fixtures from a v4 tag and
working through the whole diff, since v4 also changes decoding of some
conforming v3 documents.

There is no Python source: everything importable is defined in Rust and
described for type checkers in `toons.pyi`.

`toons.pyi` is written by hand on purpose. `maturin generate-stubs` (PyO3's
`experimental-inspect`) does produce a stub, but it drops every docstring,
omits `ToonDecodeError`, and types file objects as `Any`, so it is a
downgrade. `tests/integration/test_stubs.py` keeps the hand-written stub in
sync with the compiled signatures.

## Layout

| Path | Contents |
| --- | --- |
| `src/lib.rs` | PyO3 module: public functions, argument validation, docstrings |
| `src/serialization.rs` | Encoder (Python object to TOON text) |
| `src/deserialization.rs` | Decoder (TOON text to Python object) |
| `toons.pyi` | Type stubs, shipped in the wheel |
| `tests/integration/` | pytest suite |
| `tests/integration/fixtures/` | Official spec fixtures, vendored as JSON |
| `docs/` | MkDocs site published on Read the Docs |
| `examples/` | Runnable scripts |

## Environment

Requires [uv](https://docs.astral.sh/uv/) and a Rust toolchain.

```bash
uv venv -p 3.14                # once
uv sync --frozen               # dev dependencies
uv sync --frozen --all-groups  # add the docs dependencies too
uv run maturin develop --uv    # compile the extension into the venv
```

**`uv run` re-syncs the project by default and overwrites the module that
`maturin develop` just built.** Always run tests and scripts as:

```bash
uv run --no-sync pytest
uv run --no-sync python examples/string_example.py
```

Re-run `uv run maturin develop --uv` after every change to a `.rs` file; the
Rust code is not rebuilt automatically by pytest.

## Checks to run before proposing a change

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
uv run maturin develop --uv
uv run --no-sync pytest
```

CI runs the same four, plus `ruff check .` and `ruff format --check .` for
the Python files (tests, examples, stubs). `pre-commit run -a` covers the
formatters and linters locally.

## Adding a feature

1. Look for the behavior in the [TOON spec](https://github.com/toon-format/spec/blob/main/SPEC.md).
   The code comments reference spec section numbers; keep that habit.
2. Add or change the option in `src/lib.rs`, including validation. Unknown
   option values must raise `ValueError`, never be silently ignored.
3. Implement encoding in `src/serialization.rs` (`Encoder` methods) or
   decoding in `src/deserialization.rs` (`Parser` methods).
4. Mirror the signature and docstring in `toons.pyi`.
5. Add tests under `tests/integration/`.
6. Document user-visible behavior in `docs/` and add a `CHANGELOG.md` entry.

### Encoder invariants

- `depth` is the indentation level of the line currently being written.
  Children go one level deeper.
- A container writes its own newline and indentation only when it is not the
  root and not already positioned by a key.
- Every container is registered with `Encoder::enter` and released with
  `Encoder::leave`. This detects reference cycles and caps nesting at
  `MAX_DEPTH`; skipping it risks a stack overflow, which crashes the
  interpreter rather than raising.
- Values that cannot be represented raise `TypeError`. Do not fall back to
  `null`: silent data loss was a bug, not a feature.

### Decoder invariants

- Errors go through `Parser::err_here` / `Parser::err_at` so that
  `ToonDecodeError.line` and `.source` stay populated.
- `strict=False` only relaxes documented leniencies (blank lines inside
  arrays, indentation detection). It never changes the shape of valid data.

## Tests

The suite is pytest only, with `@pytest.mark.parametrize` instead of loops,
and assertions on complete output rather than substrings.

- `test_spec_fixtures.py` runs the vendored spec fixtures for encode and
  decode through all four entry points. Do not edit fixture JSON by hand:
  it is copied from the spec repository.
- `test_smoke.py`, `test_to_json.py`, `test_decode_errors.py`,
  `test_encode_errors.py`, `test_roundtrip.py`, `test_non_serializable.py`,
  `test_complex_regression.py` cover this implementation's own contract.
- `test_stubs.py` compares `toons.pyi` against the runtime signatures, so a
  new or changed argument fails the suite until the stub is updated.

Run a subset with `uv run --no-sync pytest -k tabular`.

## Docs

MkDocs with Material theme; `mkdocs.yml` holds the navigation. The docs
dependencies live in the `docs` group, so install them with
`uv sync --frozen --all-groups` first.

```bash
uv run --no-sync mkdocs serve
```

Every code block in `docs/` is expected to run as written and to produce the
output shown in its comments. Verify examples against a freshly built module
instead of copying them from memory.

Read the Docs installs `docs/requirements.txt`; regenerate it after changing
the `docs` dependency group:

```bash
uv export --only-group docs --no-hashes --no-emit-project -o docs/requirements.txt
```

## Conventions

- Commits follow [Conventional Commits](https://www.conventionalcommits.org/):
  `feat(parser): ...`, `fix(serialization): ...`, `docs: ...`, `ci: ...`.
- Rust: `cargo fmt` defaults, no `unwrap()` on anything reachable from Python
  input (a panic surfaces as `PanicException` and is never a correct API).
- Python: ruff with a 79-column limit.
- Comments explain why, not what. Keep them short and drop them when the code
  already says it.

## Release

Version lives in `Cargo.toml` and is re-exported as `toons.__version__`;
`pyproject.toml` takes it from there. To release: bump the version, update
`CHANGELOG.md`, tag, and let the `release` job in `.github/workflows/CI.yml`
build the wheels and publish to PyPI.

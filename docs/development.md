# Development

How to build TOONS from source and work on it.

## Prerequisites

- [uv](https://docs.astral.sh/uv/) for the Python side
- A Rust toolchain (install with [rustup](https://rustup.rs/))
- Git

## Setup

```bash
git clone https://github.com/alesanfra/toons.git
cd toons

uv venv -p 3.14
uv sync --frozen
uv run maturin develop --uv
```

`maturin develop` compiles the Rust extension and installs it into the
virtual environment.

!!! warning "Use `--no-sync` after building"
    `uv run` re-syncs the project by default, which reinstalls `toons` from
    the package index and replaces the module you just built. Run tests and
    scripts as `uv run --no-sync pytest`.

Rebuild after every change to a `.rs` file:

```bash
uv run maturin develop --uv
```

Add `--release` for an optimized build. Debug builds compile faster but
encode and decode noticeably slower.

## Project layout

```
toons/
├── src/
│   ├── lib.rs               # PyO3 module: public API and argument validation
│   ├── serialization.rs     # Encoder: Python object → TOON
│   └── deserialization.rs   # Decoder: TOON → Python object
├── toons.pyi                # Type stubs shipped in the wheel
├── tests/
│   ├── conftest.py
│   └── integration/
│       ├── fixtures/        # Official spec fixtures (JSON)
│       └── test_*.py
├── examples/                # Runnable scripts
├── docs/                    # This documentation
├── Cargo.toml               # Rust package and version number
└── pyproject.toml           # Python package, dependency groups, tooling
```

All public functions live in `src/lib.rs`. There is no Python source code.

## Tests

```bash
uv run --no-sync pytest
uv run --no-sync pytest -k tabular      # subset by name
uv run --no-sync pytest --cov=toons     # coverage
```

See the [Testing Guide](testing-guide.md) for conventions.

## Code quality

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
uvx ruff check .
uvx ruff format .
```

Or run everything configured for the repository at once:

```bash
uv run --no-sync pre-commit run -a
```

Install the hooks so they run on each commit:

```bash
uv run --no-sync pre-commit install
```

## Building wheels

```bash
uv run maturin build --release
ls target/wheels/
```

CI builds wheels for Linux (glibc and musl), macOS, and Windows on both
x86_64 and aarch64, including free-threaded Python 3.14 builds.

## Documentation

```bash
uv sync --frozen --all-groups   # installs the docs dependency group
uv run --no-sync mkdocs serve   # http://127.0.0.1:8000
uv run --no-sync mkdocs build   # static site in site/
```

Read the Docs runs `uv sync` with the `docs` group against `pyproject.toml`
and `uv.lock`, so adding a docs dependency needs nothing more than a lock
update. The build compiles the extension as well, which is why
`.readthedocs.yaml` also asks for a Rust toolchain.

## Debugging

Print from Rust with `eprintln!` and rebuild:

```rust
eprintln!("value: {:?}", obj);
```

Print from Python tests with `pytest -s` to keep stdout visible.

## Release

1. Bump the version in `Cargo.toml` (`pyproject.toml` reads it from there).
2. Add a `CHANGELOG.md` entry.
3. Tag the commit with the version number and push the tag.

The `release` job in `.github/workflows/CI.yml` builds every wheel, attests
the artifacts, and publishes to PyPI.

## Resources

- [PyO3 guide](https://pyo3.rs/)
- [maturin documentation](https://www.maturin.rs/)
- [TOON specification](https://github.com/toon-format/spec/blob/main/SPEC.md)

## See also

- [Contributing](contributing.md)
- [Testing Guide](testing-guide.md)
- [API Reference](api-reference.md)

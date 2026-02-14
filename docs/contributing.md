# Contributing

Thanks for helping improve TOONS.

## Quick steps

1. Fork the repo and create a branch.
2. Set up the dev environment.
3. Make changes with tests.
4. Run checks and open a PR.

## Prerequisites

- Python 3.7+
- Rust (latest stable)
- Git
- maturin

## Setup

```bash
pip install -r requirements-dev.txt
maturin develop
pre-commit install
```

## Build

```bash
# Debug build (fast)
maturin develop

# Release build (optimized)
maturin develop --release
```

## Tests

```bash
pytest
pytest -v
```

## Code quality

```bash
pre-commit run -a
ruff check .
ruff format .
cargo fmt
cargo clippy
```

## PR notes

- Keep changes focused and documented.
- Add tests for new behavior or bug fixes.
- Update docs for user-facing changes.

## Questions?

If you have questions about contributing, feel free to:

- Open a GitHub issue
- Start a GitHub discussion
- Review existing issues and PRs for examples

Thank you for contributing to TOONS! 🎉

## See Also
- [Code of Conduct](https://www.contributor-covenant.org/) - Community guidelines

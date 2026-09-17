# Contributing

Contributions are welcome: bug reports, fixes, documentation, and features.

## Before you start

- For a bug, open an issue with a minimal reproduction.
- For a feature, open an issue first so the design can be discussed. Behavior
  that the [TOON specification](https://github.com/toon-format/spec/blob/main/SPEC.md)
  already defines is the easiest to accept.

## Workflow

1. Fork the repository and create a branch.
2. Set up the environment as described in [Development](development.md).
3. Make the change, with tests.
4. Run the checks below.
5. Open a pull request describing what changed and why.

## Checks

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
uv run maturin develop --uv
uv run --no-sync pytest
uv run --no-sync pre-commit run -a
```

## Expectations for a pull request

- New behavior or a fixed bug comes with a test.
- User-visible changes update `docs/` and `CHANGELOG.md`.
- Public functions keep `src/lib.rs` docstrings and `toons.pyi` stubs in sync.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/),
  for example `fix(deserialization): accept blank lines in non-strict mode`.

Agents working in this repository should read
[AGENTS.md](https://github.com/alesanfra/toons/blob/main/AGENTS.md), which
records the build commands and the encoder and decoder invariants.

## Questions

Open a GitHub issue or start a discussion. Existing issues and pull requests
are a good reference for the level of detail expected.

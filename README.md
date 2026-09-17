# TOONS - Token Oriented Object Notation Serializer

[![PyPI version](https://badge.fury.io/py/toons.svg)](https://badge.fury.io/py/toons)
[![Python](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)
[![Documentation Status](https://readthedocs.org/projects/toons/badge/?version=latest)](https://toons.readthedocs.io/en/latest/?badge=latest)
[![CI](https://github.com/alesanfra/toons/workflows/CI/badge.svg)](https://github.com/alesanfra/toons/actions)
[![PyPI Downloads](https://static.pepy.tech/personalized-badge/toons?period=total&units=INTERNATIONAL_SYSTEM&left_color=BLACK&right_color=GREEN&left_text=downloads)](https://pepy.tech/projects/toons)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**A high-performance TOON (Token Oriented Object Notation) parser and serializer for Python.**

TOONS is a Rust implementation with a Python interface that mirrors the `json`
module, for the TOON format: a token-efficient serialization format designed
for Large Language Model contexts.

TOONS is listed among the [community implementations of the TOON format](https://toonformat.dev/ecosystem/implementations.html#community-implementations).

## Why TOON?

TOON uses 30-60% fewer tokens than the equivalent JSON, which matters when
data goes into a prompt. This example saves 40%:

**JSON (26 tokens):**

```json
{"users": [{"name": "Alice", "age": 25}, {"name": "Bob", "age": 30}]}
```

**TOON (16 tokens):**

```
users[2]{name,age}:
  Alice,25
  Bob,30
```

> Token counts measured with the Anthropic Claude tokenizer. Try other
> tokenizers in the [tokenizer playground](https://huggingface.co/spaces/Xenova/the-tokenizer-playground).

## Features

- **Fast**: Rust implementation with PyO3 bindings
- **Token-efficient**: 30-60% fewer tokens than JSON
- **Familiar API**: same shape as the `json` module
- **Spec compliant**: TOON v4.1, validated against the official fixtures (see [TOON version](#toon-version))
- **Typed**: type stubs ship with the wheel

## TOON version

**TOONS implements TOON specification v4.1.**

Every release is validated against the official conformance fixtures from
[spec tag v4.1.1](https://github.com/toon-format/spec/tree/v4.1.1/tests),
vendored under `tests/integration/fixtures/` and run on every build.

The v4 features are all supported:

- `#` comment lines, stripped before any other rule
- nested field groups in tabular headers
  (`orders[2]{id,customer{name,country},total}:`)
- keyed tabular form for objects of uniform objects (`users[2:]{age,city}:`)
- `key: []` and `[]` for empty arrays
- the normative decoder number grammar (`.5`, `+5`, `0x10`, `NaN` are strings)

The implemented specification version is readable at runtime, as spec
Section 13 recommends:

```python
import toons

print(toons.__toon_spec__)   # 4.1   specification version
print(toons.__version__)     # 0.8.0 library version
```

Key folding and path expansion were removed from the specification in v4.0,
so the `key_folding`, `flatten_depth`, and `expand_paths` options are gone.
The `indent` option is now spelled `indent_size`, matching the spec's
`indentSize`; `indent` keeps working as an alias.

## Install

```bash
pip install toons
```

## Usage

```python
import toons

# Parse a TOON string
data = toons.loads("""
name: Alice
age: 30
tags[3]: python,rust,toon
""")
print(data)
# {'name': 'Alice', 'age': 30, 'tags': ['python', 'rust', 'toon']}

# Serialize to TOON
print(toons.dumps({"name": "Bob", "age": 25, "active": True}))
# name: Bob
# age: 25
# active: true

# Convert TOON to JSON
print(toons.to_json("name: Alice\nage: 30", indent=2))
# {
#   "name": "Alice",
#   "age": 30
# }
```

Files work the same way as in the `json` module:

```python
import toons

with open("data.toon", "w") as f:
    toons.dump({"message": "Hello, TOON!"}, f)

with open("data.toon", "r") as f:
    data = toons.load(f)
```

## Documentation

Full documentation: **[toons.readthedocs.io](https://toons.readthedocs.io/en/stable/)**

- [Quick Start](https://toons.readthedocs.io/en/stable/) - installation and first steps
- [Data Types](https://toons.readthedocs.io/en/stable/data-types/) - how Python values map to TOON
- [Complex Examples](https://toons.readthedocs.io/en/stable/examples/) - delimiters, tabular forms, strict mode
- [API Reference](https://toons.readthedocs.io/en/stable/api-reference/) - full signatures

## Development

```bash
git clone https://github.com/alesanfra/toons.git
cd toons

uv venv -p 3.14
uv sync --frozen

uv run maturin develop --uv     # build the extension
uv run --no-sync pytest         # run the tests
```

`uv run` without `--no-sync` reinstalls the published wheel over the module
you just built. See the [Development Guide](https://toons.readthedocs.io/en/stable/development/)
and [AGENTS.md](AGENTS.md) for details.

## Wheels

Since v0.7.0, binary wheels are no longer built for the following targets,
which had close to no downloads between 2026-04-20 and 2026-05-20:

- Linux x86, s390x, ppc64le (0 downloads)
- Linux armv7l (6 downloads)
- Windows x86, 32-bit (0 downloads)

Source distributions remain available, so `toons` still installs on those
platforms when a Rust compiler is present.

## Contributing

Contributions are welcome. Please read the
[Contributing Guide](https://toons.readthedocs.io/en/stable/contributing/),
follow [Conventional Commits](https://www.conventionalcommits.org/), and run
the tests before opening a pull request.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## License

Apache License 2.0. See [LICENSE](LICENSE).

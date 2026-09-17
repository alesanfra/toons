# Testing guide

Testing conventions for TOONS.

## Running the suite

```bash
uv run maturin develop --uv     # rebuild after Rust changes
uv run --no-sync pytest
```

`uv run` without `--no-sync` reinstalls `toons` from the package index and
discards the locally built module, so the suite would test the published
wheel instead of your working copy.

Useful variations:

```bash
uv run --no-sync pytest -v                  # verbose
uv run --no-sync pytest -k tabular          # select by name
uv run --no-sync pytest -s                  # show print output
uv run --no-sync pytest --cov=toons         # coverage
```

## Conventions

- pytest only; no `unittest`.
- Group related cases in a `Test*` class with a short docstring.
- Use `@pytest.mark.parametrize` instead of looping inside a test.
- Assert on the complete output, not on a substring of it.
- Name tests `test_<function>_<scenario>`.

```python
import pytest

import toons


class TestDumps:
    """Serialization of simple values."""

    @pytest.mark.parametrize(
        "data,expected",
        [
            ({"name": "Alice"}, "name: Alice"),
            ({"age": 30}, "age: 30"),
            ({"active": True}, "active: true"),
        ],
    )
    def test_dumps_primitive_values(self, data, expected):
        """Primitives encode as bare key-value lines."""
        assert toons.dumps(data) == expected
```

## Test files

| File | Scope |
| --- | --- |
| `test_spec_fixtures.py` | Official specification fixtures |
| `test_smoke.py` | Core API surface: loads, dumps, load, dump |
| `test_to_json.py` | `to_json()` conversion |
| `test_decode_errors.py` | `ToonDecodeError` messages, `.line`, `.source` |
| `test_encode_errors.py` | Unsupported types, cycles, option validation |
| `test_roundtrip.py` | Nested arrays, large integers, tuples |
| `test_v4_forms.py` | Comments, keyed tabular, field groups, numbers |
| `test_non_serializable.py` | `datetime`, `date`, `time`, `Decimal` |
| `test_stubs.py` | `toons.pyi` matches the compiled signatures |
| `test_complex_regression.py` | Large mixed document |

## Specification compliance

TOONS is validated against the
[official TOON specification fixtures](https://github.com/toon-format/spec/tree/main/tests),
a set of language-agnostic JSON files covering encoding and decoding:
primitives, objects (nested and keyed tabular), arrays (inline, tabular,
nested, mixed), delimiters, whitespace, comment lines, root forms, and
error handling.

`tests/integration/test_spec_fixtures.py` loads every fixture file and runs
each case through `dumps`, `dump`, `loads`, and `load`, asserting the exact
expected output or the expected error when `shouldError` is set.

The fixtures under `tests/integration/fixtures/` are copied verbatim from
[spec tag v4.1.1](https://github.com/toon-format/spec/tree/v4.1.1/tests) and
are byte-identical to it. Do not edit them by hand: re-vendor them from an
upstream tag, and add implementation-specific cases in a regular test file.

## Continuous integration

Every push and pull request runs the linters and then the test suite on
Linux, macOS, and Windows against the freshly built wheels. Keep tests fast
and deterministic, and avoid depending on external services or the
filesystem outside `tmp_path`.

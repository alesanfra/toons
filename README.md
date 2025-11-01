# TOONS - a TOON Serializer

This is a fast Rust-based parser and serializer for the TOON format (Token Oriented Object Notation), a token-efficient data serialization format designed specifically for Large Language Models.

This library provides a Python interface that mirrors the API of Python's standard `json` module, making it easy to work with TOON-formatted strings and files.

## Architecture

The library is implemented in Rust using PyO3 for Python bindings, providing four core functions that parallel the standard library's `json` module:

- `load(file)` - Parse TOON data from a file object
- `loads(string)` - Parse TOON data from a string
- `dump(obj, file)` - Serialize Python object to TOON format and write to file
- `dumps(obj)` - Serialize Python object to TOON format string

The implementation follows the [TOON Specification v1.3](https://github.com/johannschopplich/toon), ensuring proper support for:
- Indentation-based structure
- Array notation with element count (e.g., `tags[3]: admin,ops,dev`)
- Tabular format for uniform object arrays
- Configurable delimiters (comma, pipe, or tab)
- Unquoted keys and string values

## Features

- **Fast serialization/deserialization**: Implemented in Rust with PyO3 bindings
- **TOON Spec v1.3 compliant**: Full support for the official TOON specification
- **Token-efficient**: 30-60% fewer tokens than equivalent JSON, ideal for LLM contexts
- **Familiar API**: Mirrors Python's `json` module interface (`load`, `loads`, `dump`, `dumps`)
- **Python native types**: Returns/accepts Python dict, list, str, int, float, bool, None
- **File and string operations**: Complete support for both file I/O and string operations

## Installation

### Production Installation

```bash
pip install toons
```

### Development Installation

```bash
# Install development dependencies
pip install -r requirements-dev.txt
pip insta

# Build the Rust extension
maturin develop
```

## Usage

The API mirrors Python's `json` module for easy adoption:

### String Operations

```python
import toons

# Parse TOON string (loads)
toon_string = """
name: John Doe
age: 30
tags[3]: admin,developer,ops
"""
data = toons.loads(toon_string)
print(data)  # {'name': 'John Doe', 'age': 30, 'tags': ['admin', 'developer', 'ops']}

# Serialize to TOON string (dumps)
data = {
    "name": "John Doe",
    "age": 30,
    "tags": ["admin", "developer", "ops"]
}
toon_output = toons.dumps(data)
print(toon_output)
# Output:
# name: John Doe
# age: 30
# tags[3]: admin,developer,ops
```

### File Operations

```python
import toons

# Parse TOON file (load)
with open('data.toon', 'r') as f:
    data = toons.load(f)
    print(data)

# Serialize to TOON file (dump)
data = {"users": [{"name": "Alice", "age": 25}, {"name": "Bob", "age": 30}]}
with open('output.toon', 'w') as f:
    toons.dump(data, f)
```

### TOON Format Examples

**Simple object:**
```
name: John
age: 30
active: true
```

**Array notation:**
```
tags[3]: admin,ops,dev
```

**Nested structure:**
```
user:
  name: John
  contacts:
    email: john@example.com
    phone: 555-1234
```

**Tabular format (uniform objects):**
```
users[2]{name,age}:
  Alice,25
  Bob,30
```

## Supported Data Types

| Python Type | TOON Format | Example |
|-------------|-------------|---------|
| `dict` | Indented key-value pairs | `name: John\nage: 30` |
| `list` | Array notation with count | `tags[3]: a,b,c` |
| `str` | Unquoted (when safe) | `name: John` |
| `int` | Number literal | `age: 30` |
| `float` | Decimal literal | `price: 19.99` |
| `bool` | `true`/`false` | `active: true` |
| `None` | `null` | `value: null` |

**Note**: The TOON format is significantly more token-efficient than JSON, especially for arrays and nested structures, making it ideal for LLM applications.

## API Reference

### `loads(s: str) -> Any`
Parse a TOON-formatted string and return the corresponding Python object.

**Arguments:**
- `s` (str): TOON-formatted string to parse

**Returns:**
- Python object (dict, list, str, int, float, bool, or None)

**Raises:**
- `ValueError`: If the string is not valid TOON format

### `load(fp: IO[str]) -> Any`
Parse TOON data from a file object and return the corresponding Python object.

**Arguments:**
- `fp`: File-like object supporting `.read()`

**Returns:**
- Python object (dict, list, str, int, float, bool, or None)

**Raises:**
- `ValueError`: If the file content is not valid TOON format

### `dumps(obj: Any) -> str`
Serialize a Python object to a TOON-formatted string.

**Arguments:**
- `obj`: Python object to serialize

**Returns:**
- TOON-formatted string

**Raises:**
- `ValueError`: If the object cannot be serialized to TOON format

### `dump(obj: Any, fp: IO[str]) -> None`
Serialize a Python object to TOON format and write to a file object.

**Arguments:**
- `obj`: Python object to serialize
- `fp`: File-like object supporting `.write()`

**Raises:**
- `ValueError`: If the object cannot be serialized to TOON format

## Error Handling

The library raises `ValueError` with descriptive error messages for invalid TOON syntax:

```python
try:
    result = toons.loads('invalid toon syntax')
except ValueError as e:
    print(f"Parse error: {e}")
```

## Examples

See the `examples/` directory for simple usage examples:

```bash
# String operations (loads/dumps)
python examples/string_example.py

# File operations (load/dump)
python examples/file_example.py
```

## Testing

Run the test suite with pytest:

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=toons

# Run specific test file
pytest tests/unit/test_loads.py

# Run specific test
pytest tests/unit/test_loads.py -k test_loads_simple_object
```

## Development

### Prerequisites

- Python 3.7+
- Rust (latest stable)
- maturin

### Building

```bash
# Development build
maturin develop

# Release build
maturin build --release
```



```
toons/
├── src/
│   └── lib.rs              # Rust implementation
├── tests/
│   └── unit/               # Unit tests
│       ├── test_loads.py   # Tests for loads()
│       ├── test_dumps.py   # Tests for dumps()
│       ├── test_load.py    # Tests for load()
│       ├── test_dump.py    # Tests for dump()
│       └── test_roundtrip.py
├── examples/               # Usage examples
├── TOON_SPEC_1.3.md       # TOON specification reference
├── Cargo.toml             # Rust dependencies
├── pyproject.toml         # Python project config
└── README.md
```

## TOON Format Specification

This library implements **TOON Specification v1.3** (2025-10-31) using the [`rtoon`](https://crates.io/crates/rtoon) Rust crate (v0.1.3).

**Note**: `rtoon` implements TOON Spec v1.2, which is fully compatible with v1.3. The specification is included in the repository as `TOON_SPEC_1.3.md` for reference.

### Compliance Verification

The test suite in `tests/unit/test_spec_compliance.py` contains 40 tests verifying compliance with TOON Spec v1.3:
- ✅ All primitive types (null, bool, int, float, string)
- ✅ Object encoding with key-value pairs and 2-space indentation
- ✅ Array notation with element count `[N]:`
- ✅ Tabular format for uniform object arrays `[N]{fields}:`
- ✅ Nested structures and complex data
- ✅ Round-trip fidelity for all data types
- ✅ File I/O operations

Run tests: `pytest tests/unit/test_spec_compliance.py -v`

### Why rtoon?

The `rtoon` crate was chosen as the implementation backend because it provides:
- **Full round-trip support**: Both encoding (serialization) and decoding (parsing) with complete fidelity
- **TOON Spec v1.2 compliance**: Follows the official TOON specification
- **Robust parsing**: Strict mode validation ensures data integrity
- **Active maintenance**: Updated regularly with the latest specification changes
- **Well-tested**: Comprehensive test suite ensures reliability

For more details on `rtoon`, see: https://github.com/shreyasbhat0/rtoon

## License

This project is open source. See LICENSE file for details.

# API Reference

Complete reference for the TOONS Python API.

## Module: `toons`

The `toons` module provides a JSON-like API for working with TOON format data.

---

## Functions

### `loads(s, *, strict=True)`

Parse a TOON-formatted string and return the corresponding Python object.

**Signature:**
```python
def loads(s: str, *, strict: bool = True) -> Any
```

**Parameters:**

- `s` (`str`): TOON-formatted string to parse
- `strict` (`bool`, optional): If `True` (default), enforce strict TOON v3.0 compliance. If `False`, allow some leniency (e.g., blank lines in arrays, indentation mismatches).

**Returns:**

- Python object: `dict`, `list`, `str`, `int`, `float`, `bool`, or `None`

**Raises:**

- `ValueError`: If the string is not valid TOON format

**Example:**

```python
import toons

# Simple object
data = toons.loads("name: Alice\nage: 30")
print(data)  # {'name': 'Alice', 'age': 30}

# Strict mode (default) - raises error on blank lines in arrays
try:
    toons.loads("items[2]:\n  - 1\n\n  - 2")
except ValueError:
    print("Strict mode error")

# Non-strict mode - allows blank lines
data = toons.loads("items[2]:\n  - 1\n\n  - 2", strict=False)

# Array with count notation
data = toons.loads("tags[3]: python,rust,toon")
print(data)  # {'tags': ['python', 'rust', 'toon']}

# Tabular format

toon_str = """
users[2]{name,age}:
  Alice,30
  Bob,25
"""
data = toons.loads(toon_str)
print(data)
# {'users': [{'name': 'Alice', 'age': 30}, {'name': 'Bob', 'age': 25}]}
```

**Error Handling:**

```python
try:
    result = toons.loads("invalid toon syntax")
except ValueError as e:
    print(f"Parse error: {e}")
```

---

### `load(fp, *, strict=True)`

Parse TOON data from a file object and return the corresponding Python object.

**Signature:**
```python
def load(fp: IO[str], *, strict: bool = True) -> Any
```

**Parameters:**

- `fp`: File-like object supporting `.read()` method
- `strict` (`bool`, optional): If `True` (default), enforce strict TOON v3.0 compliance. If `False`, allow some leniency.

**Returns:**

- Python object: `dict`, `list`, `str`, `int`, `float`, `bool`, or `None`

**Raises:**

- `ValueError`: If the file content is not valid TOON format
- `IOError`: If there are file reading errors

**Example:**

```python
import toons

# Read from file
with open("data.toon", "r") as f:
    data = toons.load(f)
    print(data)

# With error handling
try:
    with open("data.toon", "r") as f:
        data = toons.load(f)
except FileNotFoundError:
    print("File not found")
except ValueError as e:
    print(f"Invalid TOON format: {e}")
```

---

### `dumps(obj)`

Serialize a Python object to a TOON-formatted string.

**Signature:**
```python
def dumps(obj: Any) -> str
```

**Parameters:**

- `obj`: Python object to serialize (must be JSON-serializable types)

**Returns:**

- `str`: TOON-formatted string

**Raises:**

- `ValueError`: If the object cannot be serialized to TOON format

**Supported Types:**

| Python Type | TOON Format | Example Input | Example Output |
|-------------|-------------|---------------|----------------|
| `dict` | Key-value pairs | `{"name": "Alice"}` | `name: Alice` |
| `list` | Array notation | `[1, 2, 3]` | Root: `[3]: 1,2,3` |
| `str` | Unquoted/quoted | `"hello"` | `hello` |
| `int` | Number literal | `42` | `42` |
| `float` | Decimal | `3.14` | `3.14` |
| `bool` | `true`/`false` | `True` | `true` |
| `None` | `null` | `None` | `null` |

**Example:**

```python
import toons

# Simple object
data = {"name": "Alice", "age": 30}
print(toons.dumps(data))
# name: Alice
# age: 30

# List of primitives
data = {"tags": ["python", "rust", "toon"]}
print(toons.dumps(data))
# tags[3]: python,rust,toon

# Uniform objects (tabular format)
data = {
    "users": [
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25}
    ]
}
print(toons.dumps(data))
# users[2]{name,age}:
#   Alice,30
#   Bob,25

# Nested objects
data = {
    "user": {
        "name": "Alice",
        "contact": {"email": "alice@example.com"}
    }
}
print(toons.dumps(data))
# user:
#   name: Alice
#   contact:
#     email: alice@example.com
```

---

### `dump(obj, fp)`

Serialize a Python object to TOON format and write to a file object.

**Signature:**
```python
def dump(obj: Any, fp: IO[str]) -> None
```

**Parameters:**

- `obj`: Python object to serialize
- `fp`: File-like object supporting `.write()` method

**Returns:**

- `None`

**Raises:**

- `ValueError`: If the object cannot be serialized to TOON format
- `IOError`: If there are file writing errors

**Example:**

```python
import toons

# Write to file
data = {
    "users": [
        {"name": "Alice", "role": "admin"},
        {"name": "Bob", "role": "user"}
    ]
}

with open("users.toon", "w") as f:
    toons.dump(data, f)

# With error handling
try:
    with open("output.toon", "w") as f:
        toons.dump(data, f)
except ValueError as e:
    print(f"Serialization error: {e}")
except IOError as e:
    print(f"File write error: {e}")
```

---

## Data Type Mapping

### Python to TOON (Encoding)

| Python Type | TOON Representation | Notes |
|-------------|---------------------|-------|
| `dict` | Object with indented key-value pairs | 2-space indentation for nesting |
| `list` of primitives | Inline array: `key[N]: v1,v2,...` | Comma-separated by default |
| `list` of uniform objects | Tabular: `key[N]{fields}: rows...` | Auto-detected for efficiency |
| `list` of mixed types | List items: `key[N]:\n  - item1\n  - item2` | Expanded format |
| `str` | Unquoted when safe, quoted otherwise | See [quoting rules](specification.md#quoting-rules) |
| `int`, `float` | Number literal | No scientific notation |
| `bool` | `true` or `false` | Lowercase |
| `None` | `null` | |

### TOON to Python (Decoding)

| TOON Format | Python Type | Notes |
|-------------|-------------|-------|
| `key: value` | `dict` with `str` key | Unquoted strings parsed as needed |
| `key[N]: v1,v2,...` | `dict` with `list` value | Inline array |
| `key[N]{f1,f2}:\n  row1\n  row2` | `dict` with `list` of `dict` | Tabular format |
| `key[N]:\n  - item` | `dict` with `list` value | Expanded list items |
| `true`, `false` | `bool` | |
| `null` | `None` | |
| Numeric string | `int` or `float` | Parsed from unquoted tokens |
| Quoted string | `str` | Escapes unescaped |

---

## Error Handling

All functions raise `ValueError` with descriptive messages for errors:

```python
import toons

# Parse error
try:
    toons.loads("invalid: [unclosed")
except ValueError as e:
    print(f"Error: {e}")

# Serialization error
try:
    toons.dumps(lambda x: x)  # Functions can't be serialized
except ValueError as e:
    print(f"Error: {e}")
```

Common error scenarios:

- **Invalid TOON syntax**: Malformed strings, missing colons, incorrect indentation
- **Array count mismatches**: Declared count doesn't match actual items (in strict mode)
- **Unsupported types**: Attempting to serialize non-JSON-compatible types
- **Encoding errors**: File I/O errors during `load()` or `dump()`

---

## Type Hints

TOONS provides basic type hints:

```python
from typing import Any, IO

def loads(s: str, *, strict: bool = True) -> Any: ...
def load(fp: IO[str], *, strict: bool = True) -> Any: ...
def dumps(obj: Any) -> str: ...
def dump(obj: Any, fp: IO[str]) -> None: ...
```

For more precise typing in your code:

```python
from typing import Dict, List, Union

# Example with type hints
def process_user_data(toon_str: str) -> Dict[str, Union[str, int, List[str]]]:
    data = toons.loads(toon_str)
    return data
```

---

## Performance Notes

- **Rust Backend**: TOONS uses Rust for high-performance parsing and serialization
- **Memory Efficiency**: Streaming is used where possible to minimize memory usage
- **Token Efficiency**: TOON format typically uses 30-60% fewer tokens than JSON
- **Round-trip Fidelity**: Data types are preserved through serialization cycles

---

## Comparison with `json` Module

TOONS mirrors the `json` module API for easy migration:

| json | toons | Notes |
|------|-------|-------|
| `json.loads()` | `toons.loads()` | Same API, different format |
| `json.dumps()` | `toons.dumps()` | More compact output |
| `json.load()` | `toons.load()` | File operations |
| `json.dump()` | `toons.dump()` | File operations |

**Migration example:**

```python
# Before (JSON)
import json
data = json.loads('{"name": "Alice"}')
output = json.dumps(data)

# After (TOON)
import toons
data = toons.loads('name: Alice')
output = toons.dumps(data)
```

---

## See Also

- [Examples](examples.md) - Practical usage examples
- [TOON Specification](specification.md) - Format specification
- [Data Types](data-types.md) - Detailed type information
- [Development](development.md) - Contributing to TOONS

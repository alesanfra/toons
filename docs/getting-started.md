# Getting Started

TOONS is a Python library for working with TOON format, a compact, indentation-based data format similar to JSON.

## Installation

Install from PyPI (requires Python 3.7+):

```bash
pip install toons
```

Or build from source:

```bash
git clone https://github.com/alesanfra/toons.git
cd toons
pip install -r requirements-dev.txt
maturin develop
```

Quick verification:

```python
import toons
data = toons.loads("name: Alice")
print(data)  # {'name': 'Alice'}
```

## Quick Start: Parse and Serialize

```python
import toons

# Parse TOON string
data = toons.loads("name: Alice\nage: 30\ntags[2]: python,rust")
print(data)  # {'name': 'Alice', 'age': 30, 'tags': ['python', 'rust']}

# Serialize to TOON
output = toons.dumps(data)
print(output)
# name: Alice
# age: 30
# tags[2]: python,rust
```

## File Operations

```python
import toons

# Save to file
data = {"users": [{"name": "Alice", "role": "admin"}, {"name": "Bob", "role": "user"}]}
with open("users.toon", "w") as f:
    toons.dump(data, f)

# Load from file
with open("users.toon", "r") as f:
    loaded = toons.load(f)
```

## API Functions

| Function | Description |
|----------|-------------|
| `loads(s, *, strict, expand_paths, indent)` | Parse TOON string → Python object |
| `dumps(obj, *, indent, delimiter, key_folding, flatten_depth)` | Python object → TOON string |
| `load(fp, *, strict, expand_paths, indent)` | Load TOON from file |
| `dump(obj, fp, *, indent, delimiter, key_folding, flatten_depth)` | Save TOON to file |

**Key Parameters:**

- `strict`: Enforce TOON v3.0 compliance (default: `True`)
- `indent`: Spaces per indentation level (default: `2`)
- `delimiter`: Array separator: `","`, `"\t"`, or `"|"` (default: `","`)
- `expand_paths`: Path expansion mode: `"safe"`, `"always"` (default: `None`)
- `key_folding`: Flatten nested keys: `"safe"`, `"on"`, `"always"` (default: `None`)
- `flatten_depth`: Max depth for key folding

See [API Reference](api-reference.md) for complete parameter documentation.

## Error Handling

```python
import toons

try:
    toons.loads("invalid: [syntax")
except ValueError as e:
    print(f"Parse error: {e}")

try:
    toons.dumps(lambda x: x)  # Functions can't be serialized
except ValueError as e:
    print(f"Serialization error: {e}")
```

## Learn More

- [API Reference](api-reference.md) — Complete parameter documentation
- [Examples](examples.md) — Practical usage examples
- [TOON Format](specification.md) — Format specification
- [Data Types](data-types.md) — Type mapping reference

# Getting Started

This guide will help you install and start using TOONS in your Python projects.

## Installation

### From PyPI (Recommended)

Install the latest stable version from PyPI:

```bash
pip install toons
```

TOONS requires Python 3.7 or higher and works on:

- **Linux**: x86_64, x86 (i686), aarch64, armv7, s390x, ppc64le (glibc and musl)
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x64, x86

### From Source

For development or if you want to build from source:

```bash
# Clone the repository
git clone https://github.com/alesanfra/toons.git
cd toons

# Install development dependencies
pip install -r requirements-dev.txt

# Build the Rust extension with maturin
maturin develop
```

#### Prerequisites for Building from Source

- **Python 3.7+**
- **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
- **maturin** - Python build tool for Rust extensions

## Verifying Installation

Check that TOONS is installed correctly:

```python
import toons

# Test basic functionality
data = toons.loads("name: Alice")
print(data)  # {'name': 'Alice'}

# Check version (if available)
print(toons.__version__)  # e.g., '0.1.2'
```

## Your First TOON Program

Create a simple Python script to serialize and parse TOON data:

```python
import toons

# Create some data
user_data = {
    "name": "Alice",
    "age": 30,
    "roles": ["admin", "developer"],
    "active": True
}

# Serialize to TOON format
toon_string = toons.dumps(user_data)
print("TOON format:")
print(toon_string)
# Output:
# name: Alice
# age: 30
# roles[2]: admin,developer
# active: true

# Parse back to Python
parsed_data = toons.loads(toon_string)
print("\nParsed data:")
print(parsed_data)

# Verify round-trip
assert parsed_data == user_data
print("\n✓ Round-trip successful!")
```

## Working with Files

TOONS provides `load()` and `dump()` functions for file operations:

```python
import toons

# Save data to a TOON file
data = {
    "users": [
        {"name": "Alice", "role": "admin"},
        {"name": "Bob", "role": "user"}
    ]
}

with open("users.toon", "w") as f:
    toons.dump(data, f)

# Load data from a TOON file
with open("users.toon", "r") as f:
    loaded_data = toons.load(f)

print(loaded_data)
```

The resulting `users.toon` file will contain:

```
users[2]{name,role}:
  Alice,admin
  Bob,user
```

## API Overview

TOONS mirrors Python's `json` module with four core functions:

| Function | Description | Equivalent to |
|----------|-------------|---------------|
| `loads(s)` | Parse TOON string to Python object | `json.loads()` |
| `dumps(obj)` | Serialize Python object to TOON string | `json.dumps()` |
| `load(fp)` | Parse TOON from file object | `json.load()` |
| `dump(obj, fp)` | Serialize Python object to TOON file | `json.dump()` |

All functions return or accept standard Python types: `dict`, `list`, `str`, `int`, `float`, `bool`, `None`.

## Error Handling

TOONS raises `ValueError` for invalid TOON syntax:

```python
import toons

try:
    result = toons.loads("invalid: [syntax")
except ValueError as e:
    print(f"Parse error: {e}")
```

For serialization, `ValueError` is raised if the object cannot be serialized:

```python
import toons

try:
    # TOONS doesn't support arbitrary Python objects
    toons.dumps(lambda x: x)
except ValueError as e:
    print(f"Serialization error: {e}")
```

## Next Steps

- Learn more about [Examples](examples.md) for practical use cases
- Read the [API Reference](api-reference.md) for detailed documentation
- Explore the [TOON Specification](specification.md) to understand the format
- Check out the [Development Guide](development.md) to contribute

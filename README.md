# TOONS

**A high-performance TOON (Token Oriented Object Notation) parser and serializer for Python.**

TOONS is a fast Rust-based library that provides a Python interface mirroring the `json` module API, making it easy to work with the TOON format—a token-efficient data serialization format designed specifically for Large Language Models.

## Why TOON?

The TOON format achieves **30-60% fewer tokens** than equivalent JSON, making it ideal for LLM contexts where token count impacts costs and context capacity.

**JSON (37 tokens):**
```json
{"users": [{"name": "Alice", "age": 25}, {"name": "Bob", "age": 30}]}
```

**TOON (17 tokens):**
```
users[2]{name,age}:
  Alice,25
  Bob,30
```

## Features

- 🚀 **Fast**: Rust implementation with PyO3 bindings
- 📊 **Token-Efficient**: 30-60% fewer tokens than JSON
- 🔄 **Familiar API**: Drop-in replacement for `json` module
- ✅ **Spec Compliant**: Full TOON Specification v1.3 support
- 🐍 **Python Native**: Works with standard Python types

## Quick Start

### Installation

```bash
pip install toons
```

### Basic Usage

```python
import toons

# Parse TOON string
data = toons.loads("""
name: Alice
age: 30
tags[3]: python,rust,toon
""")
print(data)
# {'name': 'Alice', 'age': 30, 'tags': ['python', 'rust', 'toon']}

# Serialize to TOON
user = {"name": "Bob", "age": 25, "active": True}
print(toons.dumps(user))
# name: Bob
# age: 25
# active: true
```

### File Operations

```python
import toons

# Write to file
with open("data.toon", "w") as f:
    toons.dump({"message": "Hello, TOON!"}, f)

# Read from file
with open("data.toon", "r") as f:
    data = toons.load(f)
```

## Documentation

- 📖 **[Full Documentation](docs/)** - Complete guides and API reference
- 🚀 **[Getting Started](docs/getting-started.md)** - Installation and first steps
- 💡 **[Examples](docs/examples.md)** - Practical usage examples
- 📚 **[API Reference](docs/api-reference.md)** - Complete API documentation
- 📋 **[TOON Specification](docs/specification.md)** - Format specification v1.3

## Development

```bash
# Clone repository
git clone https://github.com/alesanfra/toons.git
cd toons

# Install dependencies
pip install -r requirements-dev.txt

# Build extension
maturin develop

# Run tests
pytest
```

See the [Development Guide](docs/development.md) for more details.

## Contributing

Contributions are welcome! Please follow [Conventional Commits](https://www.conventionalcommits.org/) and run tests before submitting.

See [Contributing Guide](docs/contributing.md) for details.

## License

This project is licensed under the Apache License 2.0. See LICENSE file for details.

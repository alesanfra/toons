# TOONS - Token Oriented Object Notation Serializer

[![PyPI version](https://badge.fury.io/py/toons.svg)](https://badge.fury.io/py/toons)
[![Python](https://img.shields.io/badge/python-3.7+-blue.svg)](https://www.python.org/downloads/)
[![Documentation Status](https://readthedocs.org/projects/toons/badge/?version=latest)](https://toons.readthedocs.io/en/latest/?badge=latest)
[![CI](https://github.com/alesanfra/toons/workflows/CI/badge.svg)](https://github.com/alesanfra/toons/actions)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**A high-performance TOON (Token Oriented Object Notation) parser and serializer for Python.**

TOONS - Token Oriented Object Notation Serializer - is a fast Rust-based library that provides a Python interface mirroring the `json` module API, making it easy to work with the TOON format—a token-efficient data serialization format designed specifically for Large Language Models.


## Documentation

📖 Read the full documentation at **[toons.readthedocs.io](https://toons.readthedocs.io/en/stable/)**.

Quick start pages:
- 🚀 **[Getting Started](https://toons.readthedocs.io/en/stable/getting-started/)** - Installation and first steps
- 💡 **[Examples](https://toons.readthedocs.io/en/stable/examples/)** - Practical usage examples
- 📚 **[API Reference](https://toons.readthedocs.io/en/stable/api-reference/)** - Complete API documentation


## Why TOON?

The TOON format achieves **30-60% fewer tokens** than equivalent JSON, making it ideal for LLM contexts where token count impacts costs and context capacity.

In this simple example we can achive -40% with respect to JSON:

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

> **Note**: Calculations were done using Anthropic Claude tokenizer, you can experiment with different tokenizer [here](https://huggingface.co/spaces/Xenova/the-tokenizer-playground)


## Features

- 🚀 **Fast**: Rust implementation with PyO3 bindings
- 📊 **Token-Efficient**: 30-60% fewer tokens than JSON
- 🔄 **Familiar API**: Drop-in replacement for `json` module
- ✅ **Spec Compliant**: Full TOON Specification v3.0 support
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

See the [Development Guide](https://toons.readthedocs.io/en/stable/development/) for more details.

## Contributing

Contributions are welcome! Please follow [Conventional Commits](https://www.conventionalcommits.org/) and run tests before submitting.

See [Contributing Guide](https://toons.readthedocs.io/en/stable/contributing/) for details.

## License

This project is licensed under the Apache License 2.0. See LICENSE file for details.

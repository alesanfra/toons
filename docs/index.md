# TOONS - Token Oriented Object Notation Serializer

**A high-performance TOON (Token Oriented Object Notation) parser and serializer for Python, implemented in Rust.**

TOONS provides a Python interface that mirrors the API of Python's standard `json` module, making it easy to work with TOON-formatted strings and files. The TOON format is designed specifically for Large Language Models, achieving 30-60% fewer tokens than equivalent JSON.

## Key Features

- **🚀 Fast**: Implemented in Rust with PyO3 bindings for maximum performance
- **📊 Token-Efficient**: 30-60% fewer tokens than JSON, ideal for LLM contexts
- **🔄 Familiar API**: Mirrors Python's `json` module (`load`, `loads`, `dump`, `dumps`)
- **✅ Spec Compliant**: Full support for TOON Specification v1.3
- **🐍 Python Native**: Returns/accepts Python dict, list, str, int, float, bool, None
- **📁 File & String**: Complete support for both file I/O and string operations

## Quick Example

```python
import toons

# Parse TOON string
data = toons.loads("""
name: John Doe
age: 30
tags[3]: admin,developer,ops
""")
print(data)  # {'name': 'John Doe', 'age': 30, 'tags': ['admin', 'developer', 'ops']}

# Serialize to TOON
data = {"name": "Alice", "age": 25}
print(toons.dumps(data))
# Output:
# name: Alice
# age: 25
```

## Why TOON?

The TOON format is significantly more compact than JSON, especially for arrays and nested structures:

**JSON (146 characters, ~37 tokens):**
```json
{"users": [{"name": "Alice", "age": 25}, {"name": "Bob", "age": 30}]}
```

**TOON (47 characters, ~17 tokens):**
```
users[2]{name,age}:
  Alice,25
  Bob,30
```

This makes TOON ideal for:

- LLM prompt contexts where tokens are limited
- API responses in AI applications
- Data serialization for machine learning pipelines
- Any scenario where minimizing token count is important

## Architecture

The library is fully implemented in Rust with a custom parser and serializer, using PyO3 to provide Python bindings. This architecture ensures:

- High-performance parsing and serialization
- Memory efficiency
- Type safety
- Full TOON Specification v1.3 compliance
- Complete control over implementation details

## Next Steps

- [Installation & Quick Start](getting-started.md) - Get started with TOONS
- [Examples](examples.md) - See practical usage examples
- [API Reference](api-reference.md) - Complete API documentation
- [TOON Specification](specification.md) - Learn about the TOON format

## License

This project is licensed under the Apache License 2.0.

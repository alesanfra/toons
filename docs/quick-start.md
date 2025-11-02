# Quick Start

Get up and running with TOONS in minutes.

## Installation

```bash
pip install toons
```

## Basic Usage

### String Operations

```python
import toons

# Parse TOON string
toon_str = """
name: Alice
age: 30
active: true
"""
data = toons.loads(toon_str)
print(data)
# {'name': 'Alice', 'age': 30, 'active': True}

# Serialize to TOON
user = {"name": "Bob", "age": 25}
print(toons.dumps(user))
# name: Bob
# age: 25
```

### File Operations

```python
import toons

# Write to file
data = {"message": "Hello, TOON!"}
with open("data.toon", "w") as f:
    toons.dump(data, f)

# Read from file
with open("data.toon", "r") as f:
    loaded = toons.load(f)
print(loaded)
```

## Common Patterns

### Working with Lists

```python
import toons

# Simple list
tags = {"tags": ["python", "rust", "toon"]}
print(toons.dumps(tags))
# tags[3]: python,rust,toon

# List of objects (tabular format)
users = {
    "users": [
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25}
    ]
}
print(toons.dumps(users))
# users[2]{name,age}:
#   Alice,30
#   Bob,25
```

### Nested Structures

```python
import toons

data = {
    "user": {
        "name": "Alice",
        "contact": {
            "email": "alice@example.com",
            "phone": "555-1234"
        }
    }
}

print(toons.dumps(data))
# user:
#   name: Alice
#   contact:
#     email: alice@example.com
#     phone: 555-1234
```

## Next Steps

- Explore [detailed examples](examples.md)
- Read the [API reference](api-reference.md)
- Learn about the [TOON format](specification.md)

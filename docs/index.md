# TOONS

Fast TOON (Token Oriented Object Notation) parsing and serialization for Python.

## Quick start

### Install

```bash
pip install toons
```

### Parse and serialize

```python
import toons

data = toons.loads("""
name: Alice
age: 30
tags[3]: admin,developer,ops
""")
print(data)
# {'name': 'Alice', 'age': 30, 'tags': ['admin', 'developer', 'ops']}

print(toons.dumps({"name": "Bob", "active": True}))
# name: Bob
# active: true
```

### Files

```python
import toons

payload = {"users": [{"id": 1, "name": "A"}, {"id": 2, "name": "B"}]}

with open("users.toon", "w") as f:
    toons.dump(payload, f)

with open("users.toon", "r") as f:
    loaded = toons.load(f)
```

## Official specification

Refer to the [official TOON specification](https://github.com/toon-format/spec/blob/main/SPEC.md) for the formal grammar and rules:

## Learn next

- [Data Types](data-types.md)
- [Complex Examples](examples.md)

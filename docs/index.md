# TOONS

Fast TOON (Token Oriented Object Notation) parsing and serialization for
Python, implemented in Rust.

TOONS mirrors the `json` module API: `loads`, `load`, `dumps`, `dump`, plus
`to_json` for converting TOON to JSON.

## Install

```bash
pip install toons
```

## Parse and serialize

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

## Read and write files

```python
import toons

payload = {"users": [{"id": 1, "name": "A"}, {"id": 2, "name": "B"}]}

with open("users.toon", "w") as f:
    toons.dump(payload, f)

with open("users.toon", "r") as f:
    loaded = toons.load(f)

print(loaded == payload)
# True
```

## Convert to JSON

```python
import toons

print(toons.to_json("users[2]{id,name}:\n  1,A\n  2,B", indent=2))
# {
#   "users": [
#     {
#       "id": 1,
#       "name": "A"
#     },
#     {
#       "id": 2,
#       "name": "B"
#     }
#   ]
# }
```

## Specification

TOONS implements **TOON specification v3.0**, validated on every build against
the official conformance fixtures from
[spec tag v3.0.1](https://github.com/toon-format/spec/tree/v3.0.1/tests).

The upstream specification is now at v4.1. The v4 additions (comment lines,
nested field groups, keyed tabular form, `key: []` for empty arrays, the
`indentSize` rename) are not implemented yet.

```python
import toons

print(toons.__toon_spec__)   # 3.0   specification version
print(toons.__version__)     # 0.8.0 library version
```

## Learn next

- [Data Types](data-types.md) - how Python values map to TOON
- [Complex Examples](examples.md) - delimiters, key folding, path expansion
- [API Reference](api-reference.md) - full signatures

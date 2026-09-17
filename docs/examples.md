# Complex examples

Practical uses of the optional encoder and decoder arguments.

## Custom delimiters

The delimiter is declared in the array header, so it round-trips. Allowed
values are `","` (default), `"\t"`, and `"|"`.

```python
import toons

data = {"users": [{"id": 1, "name": "A"}, {"id": 2, "name": "B"}]}

print(toons.dumps(data, delimiter="|"))
# users[2|]{id|name}:
#   1|A
#   2|B
```

A non-comma delimiter pays off when the values themselves contain commas,
since those values no longer need quoting.

## Key folding

Key folding rewrites a chain of single-key objects as one dotted key. It
applies only when every object in the chain has exactly one key and every
segment can be written unquoted.

```python
import toons

print(toons.dumps({"user": {"profile": {"name": "Alice"}}}, key_folding="safe"))
# user.profile.name: Alice

print(toons.dumps({"a": {"b": {"c": [1, 2, 3]}}}, key_folding="safe"))
# a.b.c[3]: 1,2,3
```

An object with more than one key stops the chain, so nothing is folded:

```python
import toons

payload = {"user": {"profile": {"name": "Alice", "role": "admin"}}}

print(toons.dumps(payload, key_folding="safe"))
# user:
#   profile:
#     name: Alice
#     role: admin
```

`flatten_depth` caps how many segments a folded key may contain:

```python
import toons

print(toons.dumps({"a": {"b": {"c": {"d": 1}}}}, key_folding="safe", flatten_depth=2))
# a.b:
#   c:
#     d: 1
```

Folding is skipped when the folded key would collide with a literal sibling
key, so decoding stays unambiguous.

## Path expansion

Path expansion is the decoding counterpart of key folding: it turns dotted
keys back into nested objects.

```python
import toons

toon_str = """
user.name: Alice
user.age: 30
"""

print(toons.loads(toon_str, expand_paths="safe"))
# {'user': {'name': 'Alice', 'age': 30}}

print(toons.loads(toon_str))
# {'user.name': 'Alice', 'user.age': 30}
```

`"safe"` expands unquoted keys only, leaving a quoted `"user.name"` as a
literal key. `"always"` expands quoted keys too. In strict mode, a conflict
between an expanded path and an existing value raises `ToonDecodeError`.

## Relaxed parsing

`strict=False` tolerates input that the specification rejects, such as blank
lines inside an array:

```python
import toons

toon_str = """
items[2]:
  - 1

  - 2
"""

print(toons.loads(toon_str, strict=False))
# {'items': [1, 2]}
```

In strict mode, the same input raises:

```python
import toons

try:
    toons.loads("items[2]:\n  - 1\n\n  - 2")
except toons.ToonDecodeError as exc:
    print(exc.line, exc)
# 3 TOON parse error at line 3: Blank line inside array
```

## Custom indentation

```python
import toons

print(toons.dumps({"config": {"host": "localhost", "port": 5432}}, indent=4))
# config:
#     host: localhost
#     port: 5432
```

The minimum is 2 spaces; smaller values raise `ValueError`. When decoding,
`indent` states the expected indentation instead of detecting it from the
input, which makes indentation errors detectable:

```python
import toons

try:
    toons.loads("a:\n   b: 1", indent=2)
except toons.ToonDecodeError as exc:
    print(exc)
# TOON parse error at line 2: Indentation 3 is not a multiple of indent size 2
```

## Converting to JSON

`to_json()` decodes TOON and re-encodes it with the standard library, which
is handy when a tool downstream expects JSON.

```python
import toons

print(toons.to_json("name: Alice\nage: 30", indent=2))
# {
#   "name": "Alice",
#   "age": 30
# }
```

## Error handling

Decoding failures raise `ToonDecodeError`, a subclass of `ValueError` that
carries the position of the problem.

```python
import toons

try:
    toons.loads("items[3]: a,b")
except toons.ToonDecodeError as exc:
    print(exc.line, repr(exc.source), str(exc))
# 1 'items[3]: a,b' TOON parse error at line 1: Array declared length 3 but found 2 elements
```

Encoding failures raise `TypeError` for unsupported values and `ValueError`
for invalid options or reference cycles:

```python
import toons

data = {}
data["self"] = data

try:
    toons.dumps(data)
except ValueError as exc:
    print(exc)
# Circular reference detected
```

## Preparing LLM context

The tabular form is what makes TOON compact for prompts:

```python
import json

import toons

transactions = [
    {"date": "2025-01-01", "amount": 99.99, "status": "completed"},
    {"date": "2025-01-05", "amount": 149.5, "status": "completed"},
    {"date": "2025-01-10", "amount": 75.0, "status": "pending"},
]
context = {"user": {"id": 123, "tier": "premium"}, "transactions": transactions}

toon_text = toons.dumps(context)
print(toon_text)
# user:
#   id: 123
#   tier: premium
# transactions[3]{date,amount,status}:
#   2025-01-01,99.99,completed
#   2025-01-05,149.5,completed
#   2025-01-10,75,pending

print(len(toon_text), len(json.dumps(context)))
# 150 247
```

## See also

- [Data Types](data-types.md)
- [API Reference](api-reference.md)
- [Development](development.md)

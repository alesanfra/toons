# Complex examples

Practical uses of the optional encoder and decoder arguments, and of the
forms TOON picks for you.

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

## Keyed tabular form

An object whose values are two or more uniform objects collapses into one
keyed header, with the entry key in front of each row:

```python
import toons

servers = {
    "servers": {
        "alpha": {"host": "a", "port": 8080},
        "beta": {"host": "b", "port": 9090},
    }
}

print(toons.dumps(servers))
# servers[2:]{host,port}:
#   alpha: a,8080
#   beta: b,9090

print(toons.loads(toons.dumps(servers)) == servers)
# True
```

The form is chosen from the data, not by an option. An object with one
entry, with entry values of differing shapes, or with a non-object value
stays in nested form.

## Nested field groups

A tabular column whose values are uniform objects becomes a nested field
group, so the rows stay flat:

```python
import toons

orders = {
    "orders": [
        {"id": 1, "customer": {"name": "Ada", "country": "DK"}, "total": 99},
        {"id": 2, "customer": {"name": "Bob", "country": "UK"}, "total": 149},
    ]
}

print(toons.dumps(orders))
# orders[2]{id,customer{name,country},total}:
#   1,Ada,DK,99
#   2,Bob,UK,149
```

Cells map to the header's leaf fields in depth-first order, and nesting is
not capped.

## Comment lines

A line whose first non-space character is `#` is a comment. The decoder
removes comment lines before every other rule, so a comment never ends a
scope or counts as a row. There are no inline or trailing comments, and the
encoder never writes one.

```python
import toons

toon_str = """
# inventory snapshot
items[2]{sku,qty}:
  A1,2
  # restocked
  B2,7
"""

print(toons.loads(toon_str))
# {'items': [{'sku': 'A1', 'qty': 2}, {'sku': 'B2', 'qty': 7}]}
```

Because `#` only starts a comment at the beginning of a line, a string value
that starts with `#` is quoted on encoding:

```python
import toons

print(toons.dumps({"note": "#x"}))
# note: "#x"
```

## Relaxed parsing

`strict=False` tolerates input that the specification rejects, such as blank
lines inside an array, a count that does not match the declared length, and
duplicate keys (last write wins):

```python
import toons

toon_str = """
items[2]:
  - 1

  - 2
"""

print(toons.loads(toon_str, strict=False))
# {'items': [1, 2]}

print(toons.loads("name: Ada\nname: Bob", strict=False))
# {'name': 'Bob'}
```

In strict mode, the same inputs raise:

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

print(toons.dumps({"config": {"host": "localhost", "port": 5432}}, indent_size=4))
# config:
#     host: localhost
#     port: 5432
```

`indent` is accepted as an alias of `indent_size`, so older code keeps
working; passing both with different values raises `ValueError`.

The minimum is 2 spaces; smaller values raise `ValueError`. When decoding,
`indent_size` states the expected indentation, which is what makes
indentation errors detectable:

```python
import toons

try:
    toons.loads("a:\n   b: 1", indent_size=2)
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
# 1 'items[3]: a,b' TOON parse error at line 1: Array declared length 3 but found 2 values
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

# Data Types

Complete guide to data type mapping between Python and TOON format.

## Overview

TOONS supports all JSON-compatible Python types with efficient serialization and precise round-trip fidelity.

## Supported Types

### Primitives

#### Strings

**Python → TOON:**

```python
import toons

# Unquoted (safe)
print(toons.dumps({"name": "Alice"}))
# name: Alice

# Quoted (contains special characters)
print(toons.dumps({"text": "Hello: World"}))
# text: "Hello: World"

# Quoted (starts with hyphen)
print(toons.dumps({"value": "-123"}))
# value: "-123"

# Unicode and emoji (safe unquoted)
print(toons.dumps({"emoji": "🎉", "text": "Hello мир"}))
# emoji: 🎉
# text: Hello мир
```

**TOON → Python:**

```python
import toons

# Unquoted strings
data = toons.loads("name: Alice")
print(data)  # {'name': 'Alice'}

# Quoted strings with escapes
data = toons.loads('text: "Line 1\\nLine 2"')
print(data)  # {'text': 'Line 1\nLine 2'}
```

**Quoting Rules:**

A string MUST be quoted if it:

1. Is empty (`""`)
2. Has leading/trailing whitespace
3. Equals `true`, `false`, or `null`
4. Is numeric-like (e.g., `"42"`, `"3.14"`, `"1e6"`)
5. Contains: `:`, `"`, `\`, `[`, `]`, `{`, `}`
6. Contains newline, tab, or carriage return
7. Contains the active delimiter
8. Equals `-` or starts with `-`

**Valid Escape Sequences:**

| Escape | Result |
|--------|--------|
| `\\` | `\` |
| `\"` | `"` |
| `\n` | newline |
| `\r` | carriage return |
| `\t` | tab |

#### Numbers

**Integers:**

```python
import toons

# Positive
print(toons.dumps({"count": 42}))
# count: 42

# Negative
print(toons.dumps({"balance": -100}))
# balance: -100

# Zero
print(toons.dumps({"value": 0}))
# value: 0

# Large numbers
print(toons.dumps({"big": 9007199254740991}))
# big: 9007199254740991
```

**Floats:**

```python
import toons

# Decimal
print(toons.dumps({"price": 19.99}))
# price: 19.99

# Negative decimal
print(toons.dumps({"temp": -3.14}))
# temp: -3.14

# Very small (no scientific notation)
print(toons.dumps({"epsilon": 0.000001}))
# epsilon: 0.000001

# Very large (no scientific notation)
print(toons.dumps({"big": 1000000.0}))
# big: 1000000.0
```

**Special Cases:**

- `-0` is normalized to `0`
- Scientific notation input (e.g., `1e6`) is expanded to `1000000`
- Sufficient precision maintained for round-trip fidelity

#### Booleans

```python
import toons

data = {"active": True, "verified": False}
print(toons.dumps(data))
# active: true
# verified: false

# Parsing
data = toons.loads("active: true\nverified: false")
print(data)  # {'active': True, 'verified': False}
```

**Note:** TOON uses lowercase `true`/`false`, not `True`/`False`.

#### Null

```python
import toons

data = {"value": None, "optional": None}
print(toons.dumps(data))
# value: null
# optional: null

# Parsing
data = toons.loads("value: null")
print(data)  # {'value': None}
```

### Collections

#### Dictionaries (Objects)

**Simple Objects:**

```python
import toons

user = {
    "id": 123,
    "name": "Alice",
    "email": "alice@example.com"
}

print(toons.dumps(user))
# id: 123
# name: Alice
# email: alice@example.com
```

**Nested Objects:**

```python
import toons

data = {
    "user": {
        "profile": {
            "name": "Alice",
            "age": 30
        }
    }
}

print(toons.dumps(data))
# user:
#   profile:
#     name: Alice
#     age: 30
```

**Key Requirements:**

- Keys must be strings
- Unquoted keys must match `^[A-Za-z_][\w.]*$`
- Other keys must be quoted
- Key order is preserved

```python
import toons

# Valid unquoted keys
data = {"name": "Alice", "user_id": 123, "api.version": "1.0"}
print(toons.dumps(data))
# name: Alice
# user_id: 123
# api.version: 1.0

# Keys requiring quotes
data = {"full name": "Alice", "user-id": 123}
print(toons.dumps(data))
# "full name": Alice
# "user-id": 123
```

#### Lists (Arrays)

**Primitive Arrays (Inline):**

```python
import toons

# Numbers
data = {"numbers": [1, 2, 3, 4, 5]}
print(toons.dumps(data))
# numbers[5]: 1,2,3,4,5

# Strings
data = {"tags": ["python", "rust", "toon"]}
print(toons.dumps(data))
# tags[3]: python,rust,toon

# Mixed primitives
data = {"mixed": [42, "hello", True, None]}
print(toons.dumps(data))
# mixed[4]: 42,hello,true,null

# Empty array
data = {"empty": []}
print(toons.dumps(data))
# empty[0]:
```

**Uniform Object Arrays (Tabular):**

```python
import toons

users = {
    "users": [
        {"name": "Alice", "age": 30, "role": "admin"},
        {"name": "Bob", "age": 25, "role": "user"},
        {"name": "Charlie", "age": 35, "role": "moderator"}
    ]
}

print(toons.dumps(users))
# users[3]{name,age,role}:
#   Alice,30,admin
#   Bob,25,user
#   Charlie,35,moderator
```

**Requirements for Tabular Format:**

1. All elements must be objects (dicts)
2. All objects must have exactly the same keys
3. All values must be primitives (no nested objects/arrays)

**Non-Uniform Arrays (Expanded):**

```python
import toons

# Mixed types
data = {
    "items": [
        42,
        "text",
        {"name": "Alice"},
        [1, 2, 3],
        None
    ]
}

print(toons.dumps(data))
# items[5]:
#   - 42
#   - text
#   - name: Alice
#   - [3]: 1,2,3
#   - null

# Non-uniform objects
data = {
    "users": [
        {"name": "Alice", "age": 30},
        {"name": "Bob", "role": "admin"}  # Different keys
    ]
}

print(toons.dumps(data))
# users[2]:
#   - name: Alice
#     age: 30
#   - name: Bob
#     role: admin
```

**Root Arrays:**

```python
import toons

# Root primitive array
data = [1, 2, 3, 4, 5]
print(toons.dumps(data))
# [5]: 1,2,3,4,5

# Root object array (tabular)
data = [
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25}
]
print(toons.dumps(data))
# [2]{name,age}:
#   Alice,30
#   Bob,25
```

## Type Conversion Table

### Python → TOON

| Python Type | TOON Format | Example |
|-------------|-------------|---------|
| `str` | Unquoted or quoted | `name: Alice` or `"name": "Alice"` |
| `int` | Number literal | `age: 30` |
| `float` | Decimal literal | `price: 19.99` |
| `bool` | `true` / `false` | `active: true` |
| `None` | `null` | `value: null` |
| `dict` | Indented key-value | `user:\n  name: Alice` |
| `list` (primitives) | Inline array | `tags[3]: a,b,c` |
| `list` (uniform objects) | Tabular | `users[2]{name,age}:\n  Alice,30\n  Bob,25` |
| `list` (mixed) | Expanded | `items[2]:\n  - 1\n  - text` |

### TOON → Python

| TOON Format | Python Type | Example |
|-------------|-------------|---------|
| `true` / `false` | `bool` | `True` / `False` |
| `null` | `NoneType` | `None` |
| Numeric token | `int` or `float` | `42` or `3.14` |
| Unquoted token | `str` | `"Alice"` |
| Quoted string | `str` | `"Hello\nWorld"` |
| `key: value` | `dict` | `{"key": "value"}` |
| `key[N]: v1,v2` | `dict` with `list` | `{"key": ["v1", "v2"]}` |
| Tabular format | `dict` with `list[dict]` | `{"users": [{"name": "Alice"}]}` |

## Edge Cases

### Empty Values

```python
import toons

# Empty object
data = {}
print(toons.dumps(data))
# (empty string)

# Empty array
data = {"items": []}
print(toons.dumps(data))
# items[0]:

# Empty string
data = {"text": ""}
print(toons.dumps(data))
# text: ""

# Parsing empty
data = toons.loads("")
print(data)  # None
```

### Special Characters

```python
import toons

# Strings with delimiters
data = {"csv": "a,b,c"}
print(toons.dumps(data))
# csv: "a,b,c"  # Quoted because contains comma

# Strings with colons
data = {"time": "12:30:45"}
print(toons.dumps(data))
# time: "12:30:45"  # Quoted because contains colon

# Strings with quotes
data = {"text": 'He said "hello"'}
print(toons.dumps(data))
# text: "He said \"hello\""  # Escaped quotes
```

### Numeric Strings

```python
import toons

# Numeric strings must be quoted
data = {"zip": "12345"}
print(toons.dumps(data))
# zip: "12345"  # Quoted to preserve as string

# Parsing
data = toons.loads("zip: 12345")  # Without quotes
print(data)  # {'zip': 12345}  # Parsed as number!

data = toons.loads('zip: "12345"')  # With quotes
print(data)  # {'zip': '12345'}  # Preserved as string
```

### Very Large Numbers

```python
import toons

# Safe integer range (JavaScript Number.MAX_SAFE_INTEGER)
safe_max = 9007199254740991
data = {"value": safe_max}
print(toons.dumps(data))
# value: 9007199254740991

# Beyond safe range - may lose precision
big = 9007199254740992
data = {"value": big}
result = toons.dumps(data)
parsed = toons.loads(result)
print(parsed["value"] == big)  # May be False due to float precision

# Solution: use string for very large numbers
data = {"value": "9007199254740992"}
print(toons.dumps(data))
# value: "9007199254740992"  # Preserved exactly
```

## Unsupported Types

These Python types cannot be directly serialized:

- **Functions** / **lambdas**
- **Classes** / **instances** (unless dict-like)
- **Modules**
- **File objects**
- **Generators**
- **Custom objects** (without conversion)

**Workaround:** Convert to supported types before serialization:

```python
import toons
from datetime import datetime

# Date objects
date = datetime.now()
data = {"timestamp": date.isoformat()}  # Convert to string
print(toons.dumps(data))
# timestamp: 2025-01-01T00:00:00

# Custom objects
class User:
    def __init__(self, name, age):
        self.name = name
        self.age = age

    def to_dict(self):
        return {"name": self.name, "age": self.age}

user = User("Alice", 30)
data = {"user": user.to_dict()}  # Convert to dict
print(toons.dumps(data))
# user:
#   name: Alice
#   age: 30
```

## Type Preservation

TOONS preserves types through round-trip serialization:

```python
import toons

original = {
    "string": "hello",
    "int": 42,
    "float": 3.14,
    "bool_true": True,
    "bool_false": False,
    "null": None,
    "array": [1, 2, 3],
    "object": {"nested": "value"}
}

# Round-trip
toon_str = toons.dumps(original)
parsed = toons.loads(toon_str)

# Verify types preserved
assert isinstance(parsed["string"], str)
assert isinstance(parsed["int"], int)
assert isinstance(parsed["float"], float)
assert isinstance(parsed["bool_true"], bool)
assert isinstance(parsed["bool_false"], bool)
assert parsed["null"] is None
assert isinstance(parsed["array"], list)
assert isinstance(parsed["object"], dict)

print("✓ All types preserved!")
```

## See Also

- [API Reference](api-reference.md) - Function signatures
- [Examples](examples.md) - Practical examples
- [Specification](specification.md) - TOON format specification

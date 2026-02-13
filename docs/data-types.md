# Data types

How TOONS maps Python values to TOON and back.

## Mapping summary

| Python | TOON | Notes |
|---|---|---|
| `dict` | object | Keys are strings; order preserved |
| `list` | array | Inline or multiline |
| `str` | string | Quoted only when needed |
| `int` | integer | No scientific notation |
| `float` | float | Normalized decimal |
| `bool` | `true`/`false` | Lowercase |
| `None` | `null` | |

## Strings

Strings are unquoted when safe, quoted when required.

Quote a string if it is empty, has leading/trailing whitespace, is numeric-like, equals `true`/`false`/`null`, starts with `-`, or contains reserved characters (`:`, `"`, `\`, `[`, `]`, `{`, `}`) or the active delimiter.

```python
import toons

print(toons.dumps({"name": "Alice"}))
# name: Alice

print(toons.dumps({"text": "Hello: World"}))
# text: "Hello: World"

print(toons.loads('text: "Line 1\\nLine 2"'))
# {'text': 'Line 1\nLine 2'}
```

## Numbers

TOON uses plain decimal notation. Scientific notation input is expanded.

```python
import toons

print(toons.dumps({"count": 42, "pi": 3.14}))
# count: 42
# pi: 3.14
```

## Booleans and null

```python
import toons

print(toons.dumps({"active": True, "value": None}))
# active: true
# value: null
```

## Objects (dict)

Unquoted keys must match $^[A-Za-z_][\w.]*$$. Other keys are quoted.

```python
import toons

print(toons.dumps({"user_id": 1, "full name": "Alice"}))
# user_id: 1
# "full name": Alice
```

## Arrays (list)

Primitive arrays are inline; mixed or nested arrays are multiline.

```python
import toons

print(toons.dumps({"tags": ["python", "rust", "toon"]}))
# tags[3]: python,rust,toon

print(toons.dumps({"items": [1, {"a": 1}, True]}))
# items[3]:
#   - 1
#   - a: 1
#   - true
```

## Tabular arrays

Uniform arrays of objects can serialize in a compact tabular form.

```python
import toons

data = {"users": [{"id": 1, "name": "A"}, {"id": 2, "name": "B"}]}
print(toons.dumps(data))
# users[2]{id,name}:
#   1,A
#   2,B
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
print(data)  # {}
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

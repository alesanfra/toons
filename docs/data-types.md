# Data types

How TOONS maps Python values to TOON and back.

## Mapping summary

| Python | TOON | Notes |
| --- | --- | --- |
| `dict` | object | Keys must be strings; insertion order is preserved |
| `list`, `tuple` | array | Inline, tabular, or expanded |
| `str` | string | Quoted only when needed |
| `int` | integer | Plain decimal, any magnitude |
| `float` | number | Plain decimal; `NaN` and infinities become `null` |
| `bool` | `true` / `false` | Lowercase |
| `None` | `null` | |
| `datetime`, `date`, `time` | string | ISO 8601, via `isoformat()` |
| `Decimal` | number | Converted through `float`, so trailing zeros are lost |

Decoding maps TOON back to `dict`, `list`, `str`, `int`, `float`, `bool`, and
`None`. A tuple therefore decodes as a list.

## Strings

Strings are unquoted when that is unambiguous, quoted when it is not.

A string is quoted when it is empty, has leading or trailing whitespace,
looks like a number, equals `true`, `false`, or `null`, starts with `-`, or
contains `:`, `"`, `\`, `[`, `]`, `{`, `}`, a newline, a tab, or the active
delimiter.

```python
import toons

print(toons.dumps({"name": "Alice"}))
# name: Alice

print(toons.dumps({"text": "Hello: World"}))
# text: "Hello: World"

print(toons.dumps({"csv": "a,b,c"}))
# csv: "a,b,c"

print(toons.loads('text: "Line 1\\nLine 2"'))
# {'text': 'Line 1\nLine 2'}
```

Quoting is what preserves the type of a string that looks numeric:

```python
import toons

print(toons.dumps({"zip": "12345"}))
# zip: "12345"

print(toons.loads("zip: 12345"))
# {'zip': 12345}

print(toons.loads('zip: "12345"'))
# {'zip': '12345'}
```

## Numbers

TOON writes plain decimal notation, never exponents. Integers keep their
exact value regardless of magnitude.

```python
import toons

print(toons.dumps({"count": 42, "pi": 3.14, "small": 2.5e-4, "big": 1.5e10}))
# count: 42
# pi: 3.14
# small: 0.00025
# big: 15000000000

print(toons.loads("value: 1180591620717411303424"))
# {'value': 1180591620717411303424}
```

Floats that are not finite have no TOON representation and encode as `null`:

```python
import toons

print(toons.dumps({"value": float("nan")}))
# value: null
```

## Booleans and null

```python
import toons

print(toons.dumps({"active": True, "value": None}))
# active: true
# value: null
```

## Objects

Keys are written bare when they match `[A-Za-z_][A-Za-z0-9_.]*`, and quoted
otherwise.

```python
import toons

print(toons.dumps({"user_id": 1, "full name": "Alice"}))
# user_id: 1
# "full name": Alice
```

An empty object encodes as nothing at the root, and an empty document
decodes as an empty dict:

```python
import toons

print(repr(toons.dumps({})))
# ''

print(toons.loads(""))
# {}
```

## Arrays

Arrays always declare their length. The layout depends on the contents.

**Primitive arrays are inline:**

```python
import toons

print(toons.dumps({"tags": ["python", "rust", "toon"]}))
# tags[3]: python,rust,toon

print(toons.dumps({"items": []}))
# items[0]:
```

**Uniform object arrays use the tabular form**, which is where TOON saves the
most tokens. It applies when every element is an object with the same keys
and only primitive values:

```python
import toons

users = {
    "users": [
        {"name": "Alice", "age": 30, "role": "admin"},
        {"name": "Bob", "age": 25, "role": "user"},
    ]
}

print(toons.dumps(users))
# users[2]{name,age,role}:
#   Alice,30,admin
#   Bob,25,user
```

**Everything else uses the expanded form**, one `- ` item per element:

```python
import toons

print(toons.dumps({"items": [42, "text", {"name": "Alice"}, [1, 2, 3], None]}))
# items[5]:
#   - 42
#   - text
#   - name: Alice
#   - [3]: 1,2,3
#   - null

print(toons.dumps({"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "role": "admin"}]}))
# users[2]:
#   - name: Alice
#     age: 30
#   - name: Bob
#     role: admin
```

**Arrays can be the root value:**

```python
import toons

print(toons.dumps([1, 2, 3, 4, 5]))
# [5]: 1,2,3,4,5

print(toons.dumps([{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]))
# [2]{name,age}:
#   Alice,30
#   Bob,25
```

## Dates and times

`datetime`, `date`, and `time` objects encode as their ISO 8601 string. They
decode back as strings, not as date objects.

```python
import toons
from datetime import date, datetime

print(toons.dumps({"day": date(2025, 2, 7)}))
# day: 2025-02-07

print(toons.dumps({"when": datetime(2025, 2, 7, 14, 30, 45)}))
# when: "2025-02-07T14:30:45"
```

## Unsupported values

Values with no TOON representation raise `TypeError`, as they do in the
`json` module: sets, generators, functions, modules, file objects, and
arbitrary class instances. Object keys that are not strings also raise
`TypeError`.

```python
import toons

try:
    toons.dumps({"tags": {"a", "b"}})
except TypeError as exc:
    print(exc)
# Object of type set is not TOON serializable
```

Convert such values before encoding:

```python
import toons


class User:
    def __init__(self, name, age):
        self.name = name
        self.age = age

    def to_dict(self):
        return {"name": self.name, "age": self.age}


print(toons.dumps({"user": User("Alice", 30).to_dict()}))
# user:
#   name: Alice
#   age: 30
```

A structure that references itself, directly or indirectly, raises
`ValueError`, and so does nesting deeper than 1000 containers.

## Round trips

Types survive a round trip, except that tuples come back as lists and
date-like objects come back as strings.

```python
import toons

original = {
    "string": "hello",
    "int": 42,
    "float": 3.14,
    "bool": True,
    "null": None,
    "array": [1, 2, 3],
    "object": {"nested": "value"},
}

assert toons.loads(toons.dumps(original)) == original
```

## See also

- [API Reference](api-reference.md)
- [Complex Examples](examples.md)

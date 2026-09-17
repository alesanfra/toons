# Data types

How TOONS maps Python values to TOON and back.

## Mapping summary

| Python | TOON | Notes |
| --- | --- | --- |
| `dict` | object | Keys must be strings; insertion order is preserved |
| `list`, `tuple` | array | Inline, tabular, or list form |
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

A string is quoted when it is empty, has a leading or trailing space or tab,
looks like a number, equals `true`, `false`, or `null`, starts with `-` or
`#`, or contains `:`, `"`, `\`, `[`, `]`, `{`, `}`, a control character, or
the active delimiter.

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
# items: []
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

A column whose values are uniform objects becomes a nested field group, so
the rows stay flat:

```python
import toons

orders = {
    "orders": [
        {"id": 1, "customer": {"name": "Ada", "country": "DK"}},
        {"id": 2, "customer": {"name": "Bob", "country": "UK"}},
    ]
}

print(toons.dumps(orders))
# orders[2]{id,customer{name,country}}:
#   1,Ada,DK
#   2,Bob,UK
```

**Everything else uses the list form**, one `- ` item per element:

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

## Objects of uniform objects

An object with two or more entries whose values share one uniform object
shape uses the keyed tabular form, which declares the fields once and puts
each entry key in front of its row:

```python
import toons

print(toons.dumps({"a": {"x": 1, "y": 2}, "b": {"x": 3, "y": 4}}))
# [2:]{x,y}:
#   a: 1,2
#   b: 3,4

print(toons.dumps({"m": {"a": {"x": 1}, "b": {"x": 2}}}))
# m[2:]{x}:
#   a: 1
#   b: 2
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

## Documented behavior

The specification requires implementations to document a few choices
(Sections 2, 4, 12, and 15). TOONS makes them as follows.

**Numbers out of the numeric domain.** Integers of any magnitude decode as
exact Python ints. A decimal or exponent token whose magnitude exceeds the
double-precision range is rejected in strict mode and decodes as
`float("inf")` when `strict=False`; a magnitude below it decodes as `0.0`.

```python
import toons

try:
    toons.loads("a: 1e400")
except toons.ToonDecodeError as exc:
    print(exc)
# TOON parse error at line 1: Number 1e400 is out of range
```

**Unpaired surrogates.** A `str` holding an unpaired surrogate has no TOON
representation and raises `ValueError` instead of being replaced with
U+FFFD.

**Tabs in indentation.** Tabs are an error in strict mode. With
`strict=False` a leading tab counts as one indentation level, whatever
`indent_size` is.

**Key order and reserved keys.** Object key order is preserved, except that
tabular and keyed tabular rows decode in the header's field order. No key
is special: `__proto__`, `constructor`, and `prototype` are ordinary dict
keys.

**Nesting.** Encoding or decoding more than 1000 nested containers raises
an error rather than exhausting the stack.

## See also

- [API Reference](api-reference.md)
- [Complex Examples](examples.md)

# TOON Format Specification

**Authoritative Reference:** [TOON Format Specification v3.0](https://github.com/toon-format/spec/blob/main/SPEC.md)

For complete format documentation, including formal grammar, conformance rules, edge cases, security considerations, and normative requirements, see the official specification above.

## Overview

**TOON** (Token-Oriented Object Notation) is a compact, indentation-based text format that encodes the JSON data model with explicit structure and minimal quoting.

**Key features:**

- **Line-oriented**: Each logical element on its own line or line group
- **Indentation-based**: Nesting via indentation levels (default 2 spaces)
- **Minimal quoting**: Strings quoted only when necessary
- **Explicit array length**: Arrays declare element count and delimiter upfront
- **Uniform object arrays**: Efficiently encodes arrays of objects with identical fields as tables
- **Deterministic**: Canonical number formatting, fixed delimiter scoping, consistent quoting rules

## Format Basics

### Objects

Objects use indentation instead of braces:

```
name: Alice
age: 30
active: true
```

Nested objects:

```
user:
  name: Alice
  address:
    city: Seattle
    zip: 98109
```

### Arrays

Primitive arrays inline with explicit count:

```
tags[3]: python,rust,javascript
```

Objects as table (uniform field names):

```
users[2]{name,age}:
  Alice,30
  Bob,25
```

Mixed/complex items expanded:

```
items[2]:
  - name: Item A
    value: 100
  - name: Item B
    value: 200
```

### Numbers and Primitives

- **Numbers**: `42`, `3.14`, `-0.5`, `1e-6` (exponent forms accepted on input; canonical decimal form on output)
- **Booleans**: `true`, `false`
- **Null**: `null`
- **Strings**: Unquoted when safe; quoted with `"` when containing special characters

### String Escaping

Quoted strings support minimal escaping:
- `\\` → backslash
- `\"` → double quote
- `\n` → newline
- `\r` → carriage return
- `\t` → tab

## Delimiters

Arrays support three delimiter characters:

- **Comma** (`,`) — default
- **Tab** (`\t`) — for tab-separated values
- **Pipe** (`|`) — for pipe-separated values

Delimiter declared in array header:

```
# Tab-delimited
tags[\t]: python\truby\tjavascript

# Pipe-delimited
row[|]: Alice|30|admin
```

## Indentation

- Default indentation: **2 spaces** per level
- Indentation is mandatory for nested structures
- **Tabs are not allowed** for indentation; only spaces
- All lines at the same depth must use consistent indentation

## Key Features

### Uniform Object Arrays (Tabular Format)

When all objects in an array have identical fields, TOON uses a compact tabular format:

```
employees[3]{id,name,department,salary}:
  1,Alice,Engineering,120000
  2,Bob,Sales,80000
  3,Carol,Engineering,125000
```

This is more compact than expanded format with repeated keys.

### Explicit Array Length

Array headers declare the number of elements:

```
tags[3]: python,rust,javascript
users[2]{name,age}:
  Alice,30
  Bob,25
```

This allows:
- Validation (exact count checking in strict mode)
- Early detection of truncation or malformed data
- Pre-allocation in decoders

### Strict Mode

**Strict mode (default):** Enforces TOON v3.0 compliance
- Exact array count validation
- Consistent indentation and delimiter usage
- No blank lines within arrays
- No invalid escape sequences

**Non-strict mode:** Allows leniency
- Missing or mismatched array counts tolerated
- Blank lines within arrays permitted
- Indentation mismatches forgiven
- Some recovery from parsing errors

## Comparison with JSON

| Aspect | JSON | TOON |
|--------|------|------|
| **Syntax** | Braces, quotes required | Indentation, minimal quoting |
| **Array declaration** | Inline `[...]` | Explicit count: `key[N]:` |
| **Uniform tables** | Repeated field names | Single header: `key[N]{fields}:` |
| **Quoting** | All strings quoted | Only when necessary |
| **Size** | Baseline | 30-60% smaller for uniform arrays |
| **Human readability** | Good | Excellent (less clutter) |

## When to Use TOON

Use TOON when:

- Data contains **arrays of uniform objects** (tables)
- You want **minimal quoting** and **compact output**
- **Human readability** matters
- You need **explicit array lengths** for validation

Don't use TOON for:

- General nested JSON conversion (use JSON)
- Flat tables without objects (use CSV/TSV)
- Public APIs requiring JSON support

## Examples

### Configuration File

```
app:
  name: MyApp
  version: 1.0.0
  debug: false

database:
  host: localhost
  port: 5432
  credentials:
    user: admin
    password: "secret@123"

features[2]: logging,telemetry
```

### LLM Prompt Context

```
conversation[3]{role,content}:
  system,"You are a helpful assistant"
  user,"What is TOON?"
  assistant,"TOON is a compact data format..."
```

### Data Export

```
products[2]{id,name,price,stock}:
  1,Widget,"9.99",150
  2,Gadget,"24.95",75
```

## External Resources

- **Official Specification:** https://github.com/toon-format/spec/blob/main/SPEC.md
- **Python Implementation:** https://github.com/alesanfra/toons
- **Specification Repository:** https://github.com/toon-format/spec

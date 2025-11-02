# TOON Specification v1.3

This page contains the complete TOON (Token-Oriented Object Notation) Specification v1.3.

!!! info "Specification Details"
    - **Version**: 1.3
    - **Date**: 2025-10-31
    - **Status**: Working Draft
    - **Author**: Johann Schopplich ([@johannschopplich](https://github.com/johannschopplich))
    - **Original Spec License**: MIT
    - **Canonical Repository**: [https://github.com/johannschopplich/toon](https://github.com/johannschopplich/toon)

!!! note "TOONS Library License"
    This TOONS library implementation is licensed under **Apache License 2.0**.

## Abstract

Token-Oriented Object Notation (TOON) is a compact, human-readable serialization format optimized for Large Language Model (LLM) contexts, achieving 30-60% token reduction versus JSON for uniform tabular data. This specification defines TOON's data model, syntax, encoding/decoding semantics, and conformance requirements.

## Status of This Document

This document is a Working Draft v1.3 and may be updated, replaced, or obsoleted. Implementers should monitor the canonical repository at https://github.com/johannschopplich/toon for changes.

This specification is stable for implementation but not yet finalized. Breaking changes are unlikely but possible before v2.0.

---

## Quick Reference

### Key Features

- **Indentation-based structure** (2 spaces by default)
- **Array notation with element count**: `tags[3]: a,b,c`
- **Tabular format** for uniform object arrays: `users[2]{name,age}:\n  Alice,30\n  Bob,25`
- **Unquoted keys and values** when safe
- **Token-efficient**: 30-60% fewer tokens than JSON

### Basic Syntax Examples

**Simple Object:**
```
name: Alice
age: 30
active: true
```

**Nested Object:**
```
user:
  name: Alice
  contact:
    email: alice@example.com
```

**Primitive Array:**
```
tags[3]: python,rust,toon
```

**Tabular Array:**
```
users[2]{name,age}:
  Alice,30
  Bob,25
```

---

## Data Model

TOON models data as:

- **JsonPrimitive**: `string | number | boolean | null`
- **JsonObject**: `{ [string]: JsonValue }`
- **JsonArray**: `JsonValue[]`

### Type Mappings

| TOON Type | Example | Python Type |
|-----------|---------|-------------|
| String | `name: Alice` | `str` |
| Number | `age: 30` | `int` or `float` |
| Boolean | `active: true` | `bool` |
| Null | `value: null` | `None` |
| Object | `user:\n  name: Alice` | `dict` |
| Array | `tags[2]: a,b` | `list` |

### Ordering Requirements

- **Array order MUST be preserved**
- **Object key order MUST be preserved** as encountered by the encoder

### Number Handling

**Encoding:**
- `-0` MUST be normalized to `0`
- Finite numbers MUST be rendered without scientific notation
  - `1e6` → `1000000`
  - `1e-6` → `0.000001`

**Precision:**
- Implementations MUST ensure round-trip fidelity
- Sufficient precision to decode back to original value
- Trailing zeros MAY be omitted (e.g., `1000000` not `1000000.0`)

---

## Syntax Rules

### Objects

**Encoding:**
```
key: value          # Primitive field
key:                # Nested or empty object
  nested: value     # Nested fields at depth +1
```

- Single space after colon for primitive values
- Key order preserved from source object
- 2-space indentation per nesting level (configurable)

### Arrays

#### Primitive Arrays (Inline)

**Syntax:** `key[N]: v1,v2,v3`

```
tags[3]: python,rust,toon
```

- `N` is the element count
- Values separated by delimiter (comma default)
- Empty arrays: `key[0]:`

#### Arrays of Uniform Objects (Tabular)

**Syntax:** `key[N]{field1,field2}:`

```
users[2]{name,age}:
  Alice,30
  Bob,25
```

**Requirements for tabular format:**
- All elements must be objects
- All objects must have the same keys
- All values must be primitives (no nested arrays/objects)

**Benefits:**
- Extremely token-efficient
- Human-readable tabular presentation
- Automatic detection by encoder

#### Mixed Arrays (Expanded)

**Syntax:** `key[N]:` followed by list items

```
items[3]:
  - 42
  - name: Alice
  - tags[2]: a,b
```

- Each item starts with `- ` at depth +1
- Supports primitives, objects, and arrays

### Strings and Quoting

#### When Strings MUST Be Quoted

A string must be quoted if it:

1. Is empty (`""`)
2. Has leading or trailing whitespace
3. Equals `true`, `false`, or `null` (case-sensitive)
4. Is numeric-like (matches `/^-?\d+(?:\.\d+)?(?:e[+-]?\d+)?$/i`)
5. Contains: `:`, `"`, `\`, `[`, `]`, `{`, `}`
6. Contains control characters (newline, tab, CR)
7. Contains the active delimiter (comma, tab, or pipe)
8. Equals `-` or starts with `-`

Otherwise, strings MAY be unquoted.

#### Escape Sequences

**Only these escapes are valid:**

| Escape | Character |
|--------|-----------|
| `\\` | Backslash |
| `\"` | Double quote |
| `\n` | Newline (U+000A) |
| `\r` | Carriage return (U+000D) |
| `\t` | Tab (U+0009) |

Any other escape sequence MUST cause a parse error.

### Keys

- Keys MAY be unquoted if they match: `^[A-Za-z_][\w.]*$`
- Otherwise, keys MUST be quoted
- All keys MUST be followed by `:`

### Delimiters

Three delimiters are supported:

| Delimiter | Symbol | Header Example |
|-----------|--------|----------------|
| Comma (default) | `,` | `[3]:` or `[3,]:` |
| Tab | U+0009 | `[3	]:` |
| Pipe | `\|` | `[3\|]:` |

**Document vs Active Delimiter:**

- **Document delimiter**: Used outside array scope for quoting decisions
- **Active delimiter**: Declared by array header, used for splitting that array's values

### Indentation

**Encoding Requirements:**
- MUST use consistent spaces per level (default 2)
- Tabs MUST NOT be used for indentation
- Exactly one space after `:` in key-value lines
- No trailing spaces on lines
- No trailing newline at document end

**Decoding (Strict Mode):**
- Leading spaces MUST be exact multiple of `indentSize`
- Tabs used as indentation MUST error
- Tabs allowed in quoted strings and as HTAB delimiter

---

## Conformance

### Encoder Conformance

An encoder MUST:

- Produce output adhering to all syntax rules
- Be deterministic in:
  - Object field order (encounter order)
  - Tabular vs expanded array detection
  - Quoting decisions based on delimiter context

### Decoder Conformance

A decoder MUST:

- Implement tokenization, escaping, and type interpretation correctly
- Parse array headers and apply declared active delimiter
- Support strict mode requirements
- Enforce indentation rules in strict mode

### Strict Mode

In strict mode (default), decoders MUST error on:

- **Array count mismatches**: declared `N` ≠ actual count
- **Tabular width mismatches**: row value count ≠ field count
- **Syntax errors**: missing colons, invalid escapes, unterminated strings
- **Indentation errors**: not a multiple of `indentSize`, tabs in indentation
- **Blank lines inside arrays/tabular rows**
- **Empty input** (no non-empty lines)

---

## Examples

### Complete Examples

**Simple Object:**
```
id: 123
name: Ada
active: true
```

**Nested Object:**
```
user:
  id: 123
  profile:
    name: Ada
    email: ada@example.com
```

**Array of Primitives:**
```
tags[4]: python,rust,javascript,go
```

**Tabular Array:**
```
products[3]{sku,name,price}:
  A001,Widget,9.99
  B002,Gadget,14.50
  C003,Tool,7.25
```

**Mixed Array:**
```
items[4]:
  - 42
  - name: Alice
    age: 30
  - tags[2]: admin,user
  - null
```

**Tab Delimiter:**
```
users[2	]{name	role}:
  Alice	admin
  Bob	user
```

**Pipe Delimiter:**
```
tags[3|]: reading|gaming|coding
```

### Invalid Examples

```
# Missing colon
key value

# Invalid escape
name: "bad\xescape"

# Wrong indentation (strict mode)
user:
   name: Alice    # 3 spaces, should be 2

# Count mismatch (strict mode)
tags[3]: a,b      # Only 2 values

# Width mismatch (strict mode)
users[2]{name,age}:
  Alice,30
  Bob             # Missing age
```

---

## Security Considerations

- Quoting rules mitigate injection and ambiguity
- Strings with colons, delimiters, hyphens, control chars, or brackets MUST be quoted
- Strict mode detects malformed strings, truncation, or injected data via count/width checks
- Encoders SHOULD avoid excessive memory on large inputs
- Unicode: encoders SHOULD NOT alter Unicode beyond required escaping

---

## Internationalization

- Full Unicode support in keys and values
- Encoders MUST NOT use locale-dependent formatting
- ISO 8601 SHOULD be used for date strings

---

## Implementation Notes

### TOONS Implementation

TOONS (this library) implements TOON Spec v1.3 using:

- **Backend**: Custom Rust parser and serializer, fully implemented from scratch
- **Language**: Rust with PyO3 Python bindings
- **Compliance**: 40+ tests verifying spec compliance
- **Implementation**: 100% native code with complete control over parsing and serialization logic

### Parsing Overview

1. Split input into lines
2. Compute depth from leading spaces / `indentSize`
3. Skip ignorable blank lines outside arrays
4. Determine root form (object, array, or primitive)
5. Parse objects at depth d, arrays at depth d+1

### Array Header Parsing

1. Locate `[...]` segment
2. Parse optional `#` marker (ignored)
3. Parse length N
4. Parse optional delimiter symbol (comma if absent)
5. Parse optional `{...}` fields segment
6. Require `:` after header
7. Return header info and inline values

---

## References

### Normative References

- **[RFC2119]**: Key words for RFCs (MUST, SHOULD, etc.)
- **[RFC8174]**: Uppercase vs lowercase in RFC 2119
- **[RFC5234]**: ABNF syntax specification

### Informative References

- **[RFC8259]**: JSON specification
- **[RFC4180]**: CSV format
- **[YAML]**: YAML specification v1.2
- **[UNICODE]**: Unicode Standard v15.1
- **[ISO8601]**: Date and time representations

---

## Full Specification Document

For the complete, authoritative specification with formal grammar and all details, see:

[TOON_SPEC_1.3.md](https://github.com/johannschopplich/toon/blob/main/SPEC.md)

---

## See Also

- [API Reference](api-reference.md) - TOONS Python API
- [Examples](examples.md) - Practical examples
- [Data Types](data-types.md) - Type mapping details

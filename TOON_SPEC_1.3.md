# TOON Specification

## Token-Oriented Object Notation

**Version:**
1.3
**Date:** 2025-10-31
**Status:** Working Draft
**Author:** Johann Schopplich
([@johannschopplich](https://github.com/johannschopplich))
**License:** MIT

---

## Abstract

Token-Oriented Object Notation (TOON) is a compact, human-readable
serialization format optimized for Large Language Model (LLM) contexts, achieving
30-60% token reduction versus JSON for uniform tabular data. This specification
defines TOON's data model, syntax, encoding/decoding semantics, and conformance
requirements.

## Status of This Document

This document is a Working Draft v1.3 and
may be updated, replaced, or obsoleted. Implementers should monitor the
canonical repository at https://github.com/johannschopplich/toon for changes.

This
specification is stable for implementation but not yet finalized. Breaking changes
are unlikely but possible before v2.0.

## Normative References

**[RFC2119]**
Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14,
RFC 2119, March 1997.
https://www.rfc-editor.org/rfc/rfc2119

**[RFC8174]**
Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC
8174, May 2017.
https://www.rfc-editor.org/rfc/rfc8174

## Informative References

**[RFC8259]** Bray, T., Ed., "The JavaScript Object Notation (JSON) Data
Interchange Format", STD 90, RFC 8259, December
2017.
https://www.rfc-editor.org/rfc/rfc8259

**[RFC4180]** Shafranovich, Y., "Common Format and MIME Type for
Comma-Separated Values (CSV) Files", RFC 4180, October
2005.
https://www.rfc-editor.org/rfc/rfc4180

**[RFC5234]** Crocker, D., Ed., and P. Overell, "Augmented BNF
for Syntax Specifications: ABNF", STD 68, RFC 5234, January
2008.
https://www.rfc-editor.org/rfc/rfc5234

**[RFC6838]** Freed, N., Klensin, J., and T. Hansen,
"Media Type Specifications and Registration Procedures", BCP 13, RFC 6838, January
2013.
https://www.rfc-editor.org/rfc/rfc6838

**[YAML]** Ben-Kiki, O., Evans,
C., and I. döt Net, "YAML Ain't Markup Language (YAML™) Version 1.2", 3rd Edition,
October 2021.
https://yaml.org/spec/1.2.2/

**[UNICODE]** The Unicode
Consortium, "The Unicode Standard", Version 15.1, September
2023.
https://www.unicode.org/versions/Unicode15.1.0/

**[ISO8601]** ISO 8601:2019, "Date and time —
Representations for information
interchange".
https://www.iso.org/standard/70907.html

## Conventions and Terminology

The key words "MUST", "MUST NOT", "REQUIRED",
"SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL"
in this document are to be interpreted as described in [RFC2119] and [RFC8174]
when, and only when, they appear in all capitals, as shown here.

Audience:
implementers of encoders/decoders/validators; tool authors; practitioners embedding
TOON in LLM prompts.

All normative text in this specification is contained in
Sections 1-16 and Section 19. All appendices are informative except where
explicitly marked normative. Examples throughout this document are informative unless
explicitly stated otherwise.

Implementations that fail to conform to any MUST or
REQUIRED level requirement are non-conformant. Implementations that conform to
all MUST and REQUIRED level requirements but fail to conform to SHOULD or
RECOMMENDED level requirements are said to be "not fully conformant" but are still
considered conformant.

## Introduction

TOON (Token-Oriented Object Notation) is a serialization
format optimized for Large Language Model contexts where token count directly
impacts costs, context capacity, and latency. While JSON and similar formats
serve general purposes, TOON achieves 30-60% token reduction for tabular data
through compact syntax, particularly for arrays of uniform objects. The format
maintains human readability, deterministic encoding, and strict validation while
modeling JSON-compatible data types.

### Specification Scope

This specification defines:

- The abstract data model (Section 2)
- Type normalization rules for encoders (Section 3)
- Concrete syntax and formatting rules (Sections 5-12)
- Parsing and decoding semantics (Section 4)
- Conformance requirements for encoders, decoders, and validators (Section 13)
- Security and internationalization considerations (Sections 15-16)

## 1. Terminology and Conventions

### Core Concepts

- TOON document: A sequence of UTF-8 text lines formatted according to this spec.
- Line: A sequence of non-newline characters terminated by LF (U+000A) in serialized form. Encoders MUST use LF.

### Structural Terms

- Indentation level (depth): Leading indentation measured in fixed-size space units (indentSize). Depth 0 has no indentation.
- Indentation unit (indentSize): A fixed number of spaces per level (default 2). Tabs MUST NOT be used for indentation.

### Array Terms

- Header: The bracketed declaration for arrays, optionally followed by a field list, and terminating with a colon; e.g., key[3]: or items[2]{a,b}:.
- Field list: Brace-enclosed, delimiter-separated list of field names for tabular arrays: {f1<delim>f2}.
- List item: A line beginning with "- " at a given depth representing an element in an expanded array.
- Length marker: Optional "#" prefix for array lengths in headers, e.g., [#3]. Decoders MUST accept and ignore it semantically.

### Delimiter Terms

- Delimiter: The character used to separate array/tabular values: comma (default), tab (HTAB, U+0009), or pipe ("|").
- Document delimiter: The encoder-selected delimiter used for quoting decisions outside any array scope (default comma).
- Active delimiter: The delimiter declared by the closest array header in scope, used to split inline primitive arrays and tabular rows under that header; it also governs quoting decisions for values within that array's scope.

### Type Terms

- Primitive: string, number, boolean, or null.
- Object: Mapping from string keys to JsonValue.
- Array: Ordered sequence of JsonValue.
- JsonValue: Primitive | Object | Array.

### Conformance Terms

- Strict mode: Decoder mode that enforces counts, indentation, and delimiter consistency; also rejects invalid escapes and missing colons (default: true).

### Notation

- Regular expressions appear in slash-delimited form.
- ABNF snippets follow RFC 5234; HTAB means the U+0009 character.

## 2. Data Model

TOON models data as:
- JsonPrimitive: string | number | boolean | null
- JsonObject: { [string]: JsonValue }
- JsonArray: JsonValue[]

### Ordering

- Array order MUST be preserved.
- Object key order MUST be preserved as encountered by the encoder.

### Numbers (encoding)

- -0 MUST be normalized to 0.
- Finite numbers MUST be rendered without scientific notation (e.g., 1e6 → 1000000; 1e-6 → 0.000001).
- Implementations MUST ensure decimal rendering does not use exponent notation.

### Numbers (precision)

- JavaScript implementations SHOULD use the language's default Number.toString() conversion, which provides sufficient precision (typically 15-17 significant digits) for round-trip fidelity with IEEE 754 double-precision values.
- Implementations MUST preserve sufficient precision to ensure round-trip fidelity: decoding an encoded number MUST yield a value equal to the original.
- Trailing zeros MAY be omitted for whole numbers (e.g., 1000000 is preferred over 1000000.0).
- Very large numbers (e.g., greater than 10^20) that may lose precision in floating-point representation SHOULD be converted to quoted decimal strings if exact precision is required.

### Null

Represented as the literal null.

## 3. Encoding Normalization (Reference Encoder)

The reference encoder normalizes non-JSON values to the data model:

- Number:
  - Finite → number (non-exponential). -0 → 0.
  - NaN, +Infinity, -Infinity → null.
- BigInt (JavaScript):
  - If within Number.MIN_SAFE_INTEGER..Number.MAX_SAFE_INTEGER → converted to number.
  - Otherwise → converted to a decimal string (e.g., "9007199254740993") and encoded as a string (quoted because it is numeric-like).
- Date → ISO string (e.g., "2025-01-01T00:00:00.000Z").
- Set → array by iterating entries and normalizing each element.
- Map → object using String(key) for keys and normalizing values.
- Plain object → own enumerable string keys in encounter order; values normalized recursively.
- Function, symbol, undefined, or unrecognized types → null.

Note: Other language ports SHOULD apply analogous normalization consistent with this spec's data model and encoding rules.

## 4. Decoding Interpretation (Reference Decoder)

Decoders map text tokens to host values:

### Quoted tokens (strings and keys)

- MUST be unescaped per Section 7.1 (only \\, \", \n, \r, \t are valid). Any other escape or an unterminated string MUST error.
- Quoted primitives remain strings even if they look like numbers/booleans/null.

### Unquoted value tokens

- true, false, null → booleans/null.
- Numeric parsing:
  - MUST accept standard decimal and exponent forms (e.g., 42, -3.14, 1e-6, -1E+9).
  - MUST treat tokens with forbidden leading zeros (e.g., "05", "0001") as strings (not numbers).
  - Only finite numbers are expected from conforming encoders.
- Otherwise → string.

### Keys

- Decoded as strings (quoted keys MUST be unescaped per Section 7.1).
- A colon MUST follow a key; missing colon MUST error.

## 5. Concrete Syntax and Root Form

TOON is a deterministic, line-oriented, indentation-based notation.

### Objects

- key: value for primitives.
- key: alone for nested or empty objects; nested fields appear at depth +1.

### Arrays

- Primitive arrays are inline: key[N<delim?>]: v1<delim>v2….
- Arrays of arrays (primitives): expanded list items under a header: key[N<delim?>]: then "- [M<delim?>]: …".
- Arrays of objects:
  - Tabular form when uniform and primitive-only: key[N<delim?>]{f1<delim>f2}: then one row per line.
  - Otherwise: expanded list items: key[N<delim?>]: with "- …" items (see Sections 9.4 and 10).

### Root form discovery

- If the first non-empty depth-0 line is a valid root array header per Section 6 (must include a colon), decode a root array.
- Else if the document has exactly one non-empty line and it is neither a valid array header nor a key-value line (quoted or unquoted key), decode a single primitive.
- Otherwise, decode an object.
- In strict mode, multiple non-key/value non-header lines at depth 0 is invalid.

## 6. Header Syntax (Normative)

Array headers declare length and active delimiter, and optionally field names.

### General forms

- Root header (no key): [<marker?>N<delim?>]:
- With key: key[<marker?>N<delim?>]:
- Tabular fields: key[<marker?>N<delim?>]{field1<delim>field2<delim>…}:

### Where

- N is the non-negative integer length.
- <marker?> is optional "#"; decoders MUST accept and ignore it semantically.
- <delim?> is:
  - absent for comma (","),
  - HTAB (U+0009) for tab,
  - "|" for pipe.
- Field names in braces are separated by the same active delimiter and encoded as keys (Section 7.3).

### Spacing and delimiters

- Every header line MUST end with a colon.
- When inline values follow a header on the same line (non-empty primitive arrays), there MUST be exactly one space after the colon before the first value.
- The active delimiter declared by the bracket segment applies to:
  - splitting inline primitive arrays on that header line,
  - splitting tabular field names in "{…}",
  - splitting all rows/items within the header's scope,
  - unless a nested header changes it.
- The same delimiter symbol declared in the bracket MUST be used in the fields segment and in all row/value splits in that scope.
- Absence of a delimiter symbol in a bracket segment ALWAYS means comma, regardless of any parent header.

### Normative header grammar (ABNF)

```
; Core rules from RFC 5234
ALPHA  = %x41-5A / %x61-7A   ; A-Z / a-z
DIGIT  = %x30-39             ; 0-9
DQUOTE = %x22                ; "
HTAB   = %x09                ; horizontal tab
LF     = %x0A                ; line feed
SP     = %x20                ; space

; Header syntax
bracket-seg   = "[" [ "#" ] 1*DIGIT [ delimsym ] "]"
delimsym      = HTAB / "|"

; Field names are keys (quoted/unquoted) separated by the active delimiter
fields-seg    = "{" fieldname *( delim fieldname ) "}"
delim         = delimsym / ","
fieldname     = key

header        = [ key ] bracket-seg [ fields-seg ] ":"
key           = unquoted-key / quoted-key

; Unquoted keys must match identifier pattern
unquoted-key  = ( ALPHA / "_" ) *( ALPHA / DIGIT / "_" / "." )

; Quoted keys use only escapes from Section 7.1
; (Exact escaped-char repertoire is defined in Section 7.1)
; quoted-key   = DQUOTE *(escaped-char / safe-char) DQUOTE
```

Note: The grammar above specifies header syntax. TOON's grammar is deliberately designed to prioritize human readability and token efficiency over strict LR(1) parseability. This requires some context-sensitive parsing (particularly for tabular row disambiguation in Section 9.3), which is a deliberate design tradeoff. Reference implementations demonstrate that deterministic parsing is achievable with modest lookahead.

### Decoding requirements

- The bracket segment MUST parse as a non-negative integer length N.
- If a trailing tab or pipe appears inside the brackets, it selects the active delimiter; otherwise comma is active.
- If a fields segment occurs between the bracket and the colon, parse field names using the active delimiter; quoted names MUST be unescaped per Section 7.1.
- A colon MUST follow the bracket and optional fields; missing colon MUST error.

## 7. Strings and Keys

### 7.1 Escaping (Encoding and Decoding)

In quoted strings and keys, the following characters MUST be escaped:
- "\\" → "\\\\"
- "\"" → "\\\""
- U+000A newline → "\\n"
- U+000D carriage return → "\\r"
- U+0009 tab → "\\t"

Decoders MUST reject any other escape sequence and unterminated strings.

Tabs are allowed inside quoted strings and as a declared delimiter; they MUST NOT be used for indentation (Section 12).

### 7.2 Quoting Rules for String Values (Encoding)

A string value MUST be quoted if any of the following is true:
- It is empty ("").
- It has leading or trailing whitespace.
- It equals true, false, or null (case-sensitive).
- It is numeric-like:
  - Matches /^-?\d+(?:\.\d+)?(?:e[+-]?\d+)?$/i (e.g., "42", "-3.14", "1e-6").
  - Or matches /^0\d+$/ (leading-zero decimals such as "05").
- It contains a colon (:), double quote ("), or backslash (\).
- It contains brackets or braces ([, ], {, }).
- It contains control characters: newline, carriage return, or tab.
- It contains the relevant delimiter:
  - Inside array scope: the active delimiter (Section 1).
  - Outside array scope: the document delimiter (Section 1).
- It equals "-" or starts with "-" (any hyphen at position 0).

Otherwise, the string MAY be emitted without quotes. Unicode, emoji, and strings with internal (non-leading/trailing) spaces are safe unquoted provided they do not violate the conditions.

### 7.3 Key Encoding (Encoding)

Object keys and tabular field names:
- MAY be unquoted only if they match: ^[A-Za-z_][\w.]*$.
- Otherwise, they MUST be quoted and escaped per Section 7.1.

### 7.4 Decoding Rules for Strings and Keys (Decoding)

- Quoted strings and keys MUST be unescaped per Section 7.1; any other escape MUST error. Quoted primitives remain strings.
- Unquoted values:
  - true/false/null → boolean/null
  - Numeric tokens → numbers (with the leading-zero rule in Section 4)
  - Otherwise → strings
- Keys (quoted or unquoted) MUST be followed by ":"; missing colon MUST error.

## 8. Objects

### Encoding

- Primitive fields: key: value (single space after colon).
- Nested or empty objects: key: on its own line. If non-empty, nested fields appear at depth +1.
- Key order: Implementations MUST preserve encounter order when emitting fields.
- An empty object at the root yields an empty document (no lines).

### Decoding

- A line "key:" with nothing after the colon at depth d opens an object; subsequent lines at depth > d belong to that object until the depth decreases to ≤ d.
- Lines "key: value" at the same depth are sibling fields.
- Missing colon after a key MUST error.

## 9. Arrays

### 9.1 Primitive Arrays (Inline)

#### Encoding

- Non-empty arrays: key[N<delim?>]: v1<delim>v2<delim>… where each vi is encoded as a primitive (Section 7) with delimiter-aware quoting.
- Empty arrays: key[0<delim?>]: (no values following).
- Root arrays: [N<delim?>]: v1<delim>…

#### Decoding

- Split using the active delimiter declared by the header; non-active delimiters MUST NOT split values.
- In strict mode, the number of decoded values MUST equal N; otherwise MUST error.

### 9.2 Arrays of Arrays (Primitives Only) — Expanded List

#### Encoding

- Parent header: key[N<delim?>]: on its own line.
- Each inner primitive array is a list item:
  - - [M<delim?>]: v1<delim>v2<delim>…
  - Empty inner arrays: - [0<delim?>]:

#### Decoding

- Items appear at depth +1, each starting with "- " and an inner array header "[M<delim?>]: …".
- Inner arrays are split using their own active delimiter; in strict mode, counts MUST match M.
- In strict mode, the number of list items MUST equal outer N.

### 9.3 Arrays of Objects — Tabular Form

#### Tabular detection (encoding; MUST hold for all elements)

- Every element is an object.
- All objects have the same set of keys (order per object MAY vary).
- All values across these keys are primitives (no nested arrays/objects).

#### When satisfied (encoding)

- Header: key[N<delim?>]{f1<delim>f2<delim>…}: where field order is the first object's key encounter order.
- Field names encoded per Section 7.3.
- Rows: one line per object at depth +1 under the header; values are encoded primitives (Section 7) and joined by the active delimiter.
- Root tabular arrays omit the key: [N<delim?>]{…}: followed by rows.

#### Decoding

- A tabular header declares the active delimiter and ordered field list.
- Rows appear at depth +1 as delimiter-separated value lines.
- Strict mode MUST enforce:
  - Each row's value count equals the field count.
  - The number of rows equals N.
- Disambiguation at row depth (unquoted tokens):
  - Compute the first unquoted occurrence of the active delimiter and the first unquoted colon.
  - If a same-depth line has no unquoted colon → row.
  - If both appear, compare first-unquoted positions:
    - Delimiter before colon → row.
    - Colon before delimiter → key-value line (end of rows).
  - If a line has an unquoted colon but no unquoted active delimiter → key-value line (end of rows).

### 9.4 Mixed / Non-Uniform Arrays — Expanded List

#### When tabular requirements are not met (encoding)

- Header: key[N<delim?>]:
- Each element is rendered as a list item at depth +1 under the header:
  - Primitive: - <primitive>
  - Primitive array: - [M<delim?>]: v1<delim>…
  - Object: formatted per Section 10 (objects as list items).
  - Complex arrays: - key'[M<delim?>]: followed by nested items as appropriate.

#### Decoding

- Header declares list length N and the active delimiter for any nested inline arrays.
- Each list item starts with "- " at depth +1 and is parsed as:
  - Primitive (no colon and no array header),
  - Inline primitive array (- [M<delim?>]: …),
  - Object with first field on the hyphen line (- key: … or - key[N…]{…}: …),
  - Or nested arrays via nested headers.
- In strict mode, the number of list items MUST equal N.

## 10. Objects as List Items

For an object appearing as a list item:

### Empty object list item

A single "-" at the list-item indentation level.

### First field on the hyphen line

- Primitive: - key: value
- Primitive array: - key[M<delim?>]: v1<delim>…
- Tabular array: - key[N<delim?>]{fields}:
  - Followed by tabular rows at depth +1 (relative to the hyphen line).
- Non-uniform array: - key[N<delim?>]:
  - Followed by list items at depth +1.
- Object: - key:
  - Nested object fields appear at depth +2 (i.e., one deeper than subsequent sibling fields of the same list item).
- Remaining fields of the same object appear at depth +1 under the hyphen line in encounter order, using normal object field rules.

### Decoding

- The first field is parsed from the hyphen line. If it is a nested object (- key:), nested fields are at +2 relative to the hyphen line; subsequent fields of the same list item are at +1.
- If the first field is a tabular header on the hyphen line, its rows are at +1; subsequent sibling fields continue at +1 after the rows.

## 11. Delimiters

### Supported delimiters

- Comma (default): header omits the delimiter symbol.
- Tab: header includes HTAB inside brackets and braces (e.g., [N<TAB>], {a<TAB>b}); rows/inline arrays use tabs.
- Pipe: header includes "|" inside brackets and braces; rows/inline arrays use "|".

### Document vs Active delimiter

- Encoders select a document delimiter (option) that influences quoting in contexts not governed by an array header (e.g., object values).
- Inside an array header's scope, the active delimiter governs splitting and quoting of inline arrays and tabular rows for that array.
- Absence of a delimiter symbol in a header ALWAYS means comma for that array's scope; it does not inherit from any parent.

### Delimiter-aware quoting (encoding)

- Within an array's scope, strings containing the active delimiter MUST be quoted to avoid splitting.
- Outside any array scope, encoders SHOULD use the document delimiter to decide delimiter-aware quoting for values.
- Strings containing non-active delimiters do not require quoting unless another quoting condition applies (Section 7.2).

### Delimiter-aware parsing (decoding)

- Inline arrays and tabular rows MUST be split only on the active delimiter declared by the nearest array header.
- Strings containing the active delimiter MUST be quoted to avoid splitting; non-active delimiters MUST NOT cause splits.
- Nested headers may change the active delimiter; decoding MUST use the delimiter declared by the nearest header.
- If the bracket declares tab or pipe, the same symbol MUST be used in the fields segment and for splitting all rows/values in that scope.

## 12. Indentation and Whitespace

### Encoding

- Encoders MUST use a consistent number of spaces per level (default 2; configurable).
- Tabs MUST NOT be used for indentation.
- Exactly one space after ": " in key: value lines.
- Exactly one space after array headers when followed by inline values.
- No trailing spaces at the end of any line.
- No trailing newline at the end of the document.

### Decoding

#### Strict mode

- The number of leading spaces on a line MUST be an exact multiple of indentSize; otherwise MUST error.
- Tabs used as indentation MUST error. Tabs are allowed in quoted strings and as the HTAB delimiter.

#### Non-strict mode

- Depth MAY be computed as floor(indentSpaces / indentSize).
- Tabs in indentation are non-conforming and MAY be accepted or rejected.

### Whitespace handling

- Surrounding whitespace around tokens SHOULD be tolerated; internal semantics follow quoting rules.
- Blank lines:
  - Outside arrays/tabular rows: decoders SHOULD ignore completely blank lines (do not create/close structures).
  - Inside arrays/tabular rows: in strict mode, MUST error; in non-strict mode, MAY be ignored and not counted as a row/item.
- Trailing newline at end-of-file: decoders SHOULD accept; validators MAY warn.

## 13. Conformance

### Conformance classes

- Encoder:
  - MUST produce output adhering to all normative rules in Sections 2–12 and 15.
  - MUST be deterministic regarding:
    - Object field order (encounter order).
    - Tabular detection (uniform vs non-uniform).
    - Quoting decisions given values and delimiter context (document delimiter or active delimiter in array scope).

- Decoder:
  - MUST implement tokenization, escaping, and type interpretation per Sections 4 and 7.4.
  - MUST parse array headers per Section 6 and apply the declared active delimiter to inline arrays and tabular rows.
  - MUST implement structure and depth rules per Sections 8–11, including objects-as-list-items placement.
  - MUST enforce strict-mode rules in Section 14 when strict = true.

- Validator:
  - SHOULD verify structural conformance (headers, indentation, list markers).
  - SHOULD verify whitespace invariants.
  - SHOULD verify delimiter consistency between headers and rows.
  - SHOULD verify length counts vs declared [N].

### Options

- Encoder options:
  - indent (default: 2 spaces)
  - delimiter (document delimiter; default: comma; alternatives: tab, pipe)
  - lengthMarker (default: disabled)

- Decoder options:
  - indent (default: 2 spaces)
  - strict (default: true)

## 14. Strict Mode Errors and Diagnostics (Authoritative Checklist)

When strict mode is enabled (default), decoders MUST error on the following conditions.

### 14.1 Array Count and Width Mismatches

- Inline primitive arrays: decoded value count ≠ declared N.
- List arrays: number of list items ≠ declared N.
- Tabular arrays: number of rows ≠ declared N.
- Tabular row width mismatches: any row's value count ≠ field count.

### 14.2 Syntax Errors

- Missing colon in key context.
- Invalid escape sequences or unterminated strings in quoted tokens.
- Delimiter mismatch (detected via width/count checks and header scope).

### 14.3 Indentation Errors

- Leading spaces not a multiple of indentSize.
- Any tab used in indentation (tabs allowed in quoted strings and as HTAB delimiter).

### 14.4 Structural Errors

- Blank lines inside arrays/tabular rows.
- Empty input (document with no non-empty lines after ignoring trailing newline(s) and ignorable blank lines outside arrays/tabular rows).

## 15. Security Considerations

- Injection and ambiguity are mitigated by quoting rules:
  - Strings with colon, the relevant delimiter (document or active), hyphen marker cases ("-" or strings starting with "-"), control characters, or brackets/braces MUST be quoted.
- Strict-mode checks (Section 14) detect malformed strings, truncation, or injected rows/items via length and width mismatches.
- Encoders SHOULD avoid excessive memory on large inputs; implement streaming/tabular row emission where feasible.
- Unicode:
  - Encoders SHOULD avoid altering Unicode beyond required escaping; decoders SHOULD accept valid UTF-8 in quoted strings/keys (with only the five escapes).

## 16. Internationalization

- Full Unicode is supported in keys and values, subject to quoting and escaping rules.
- Encoders MUST NOT apply locale-dependent formatting for numbers or booleans (e.g., no thousands separators).
- ISO 8601 strings SHOULD be used for Date normalization.

## Appendix A: Examples (Informative)

### Objects

```
id: 123
name: Ada
active: true
```

### Nested objects

```
user:
  id: 123
  name: Ada
```

### Primitive arrays

```
tags[3]: admin,ops,dev
```

### Arrays of arrays (primitives)

```
pairs[2]:
  - [2]: 1,2
  - [2]: 3,4
```

### Tabular arrays

```
items[2]{sku,qty,price}:
  A1,2,9.99
  B2,1,14.5
```

### Mixed arrays

```
items[3]:
  - 1
  - a: 1
  - text
```

### Objects as list items

```
items[2]:
  - id: 1
    name: First
  - id: 2
    name: Second
    extra: true
```

### Delimiter variations

```
items[2	]{sku	name	qty	price}:
  A1	Widget	2	9.99
  B2	Gadget	1	14.5

tags[3|]: reading|gaming|coding
```

### Error cases (invalid TOON)

```
key value

name: "bad\xescape"

items[1]:
   - value

items[3]{id,name}:
  1,Alice
  2,Bob

tags[5]: a,b,c
```

## Appendix B: Parsing Helpers (Informative)

### Decoding Overview

- Split input into lines; compute depth from leading spaces and indent size (Section 12).
- Skip ignorable blank lines outside arrays/tabular rows (Section 12).
- Decide root form per Section 5.
- For objects at depth d: process lines at depth d; for arrays at depth d: read rows/list items at depth d+1.

### Array Header Parsing

- Locate the first "[ … ]" segment on the line; parse:
  - Optional leading "#" marker (ignored semantically).
  - Length N as decimal integer.
  - Optional delimiter symbol at the end: HTAB or pipe (comma otherwise).
- If a "{ … }" fields segment occurs between the "]" and the ":", parse field names using the active delimiter; unescape quoted names.
- Require a colon ":" after the bracket/fields segment.
- Return the header (key?, length, delimiter, fields?, hasLengthMarker) and any inline values after the colon.
- Absence of a delimiter symbol in the bracket segment ALWAYS means comma for that header (no inheritance).

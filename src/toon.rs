//! Native TOON v2.0 implementation
//!
//! This module implements TOON (Token-Oriented Object Notation) serialization
//! and deserialization according to the TOON Specification v2.0 (2025-11-10).
//!
//! Key features:
//! - Direct Python object integration (no JSON intermediate representation)
//! - Full TOON v2.0 spec compliance
//! - Tabular format support for uniform arrays of objects
//! - Configurable delimiters (comma, tab, pipe)
//! - Strict mode parsing with validation

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use std::fmt::Write as FmtWrite;

/// Serialize a Python object to TOON format string.
///
/// Implements TOON Specification v2.0 encoding rules:
/// - Objects: key: value with proper indentation
/// - Arrays: headers with inline or tabular format
/// - Primitives: proper quoting and escaping
/// - Tabular optimization for uniform object arrays
pub fn serialize(py: Python, obj: &Bound<'_, PyAny>, indent: usize) -> PyResult<String> {
    // Validate indent parameter (must be >= 2 per TOON spec v2.0)
    if indent < 2 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "indent must be >= 2 (TOON spec v2.0 uses 2-space indentation)",
        ));
    }

    // Special case: empty dict serializes as empty string per TOON spec
    if let Ok(dict) = obj.cast::<PyDict>() {
        if dict.is_empty() {
            return Ok(String::new());
        }
    }

    let mut output = String::new();
    serialize_value(py, obj, &mut output, 0, ',', true, indent)?;

    Ok(output)
}

/// Deserialize a TOON format string to a Python object.
///
/// Implements TOON Specification v2.0 decoding rules:
/// - Automatic root form detection (object/array/primitive)
/// - Header parsing with delimiter detection
/// - Tabular array reconstruction
/// - Strict validation in strict mode
pub fn deserialize(py: Python, input: &str) -> PyResult<Py<PyAny>> {
    let mut parser = Parser::new(input);
    parser.parse(py)
}

// ============================================================================
// SERIALIZATION
// ============================================================================

/// Serialize a value at a given depth with specified delimiter context
fn serialize_value(
    py: Python,
    obj: &Bound<'_, PyAny>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    is_root: bool,
    indent_size: usize,
) -> PyResult<()> {
    if obj.is_none() {
        output.push_str("null");
    } else if let Ok(b) = obj.extract::<bool>() {
        output.push_str(if b { "true" } else { "false" });
    } else if let Ok(i) = obj.extract::<i64>() {
        write!(output, "{}", i).unwrap();
    } else if let Ok(f) = obj.extract::<f64>() {
        // TOON v2.0: normalize -0 to 0, no exponential notation
        if f == 0.0 {
            output.push('0');
        } else if f.is_finite() {
            // Format without exponential notation
            write!(output, "{}", f).unwrap();
        } else {
            // NaN, Infinity → null (per spec Section 3)
            output.push_str("null");
        }
    } else if let Ok(s) = obj.extract::<String>() {
        serialize_string(&s, output, delimiter);
    } else if let Ok(list) = obj.cast::<PyList>() {
        serialize_array(py, &list, output, depth, delimiter, is_root, indent_size)?;
    } else if let Ok(dict) = obj.cast::<PyDict>() {
        serialize_object(py, &dict, output, depth, delimiter, is_root, indent_size)?;
    } else {
        // Unknown type → null (per spec Section 3)
        output.push_str("null");
    }
    Ok(())
}

/// Serialize a string with proper quoting and escaping per TOON v2.0 Section 7
fn serialize_string(s: &str, output: &mut String, delimiter: char) {
    if needs_quoting(s, delimiter) {
        output.push('"');
        for ch in s.chars() {
            match ch {
                '\\' => output.push_str("\\\\"),
                '"' => output.push_str("\\\""),
                '\n' => output.push_str("\\n"),
                '\r' => output.push_str("\\r"),
                '\t' => output.push_str("\\t"),
                _ => output.push(ch),
            }
        }
        output.push('"');
    } else {
        output.push_str(s);
    }
}

/// Check if a string needs quoting per TOON v2.0 Section 7.2
fn needs_quoting(s: &str, delimiter: char) -> bool {
    if s.is_empty() {
        return true;
    }

    // Check for leading/trailing whitespace
    if s.starts_with(|c: char| c.is_whitespace()) || s.ends_with(|c: char| c.is_whitespace()) {
        return true;
    }

    // Check for reserved keywords
    if s == "true" || s == "false" || s == "null" {
        return true;
    }

    // Check if numeric-like
    if is_numeric_like(s) {
        return true;
    }

    // Check for special characters
    for ch in s.chars() {
        match ch {
            ':' | '"' | '\\' | '[' | ']' | '{' | '}' | '\n' | '\r' | '\t' => return true,
            _ if ch == delimiter => return true,
            _ => {}
        }
    }

    // Check if starts with hyphen
    if s.starts_with('-') {
        return true;
    }

    false
}

/// Check if string looks numeric per TOON v2.0 Section 7.2
fn is_numeric_like(s: &str) -> bool {
    // Matches: -?\d+(\.\d+)?(e[+-]?\d+)? or 0\d+
    if s.chars().next().unwrap_or(' ').is_ascii_digit() {
        // Check for leading zero with more digits (e.g., "05")
        if s.starts_with('0') && s.len() > 1 && s.chars().nth(1).unwrap().is_ascii_digit() {
            return true;
        }
    }

    // Try to parse as number
    s.parse::<f64>().is_ok()
}

/// Serialize an object (dict) per TOON v2.0 Section 8
fn serialize_object(
    py: Python,
    dict: &Bound<'_, PyDict>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    is_root: bool,
    indent_size: usize,
) -> PyResult<()> {
    let items: Vec<_> = dict.items().iter().collect();

    if items.is_empty() {
        // Empty object: no output at root, empty line with key elsewhere
        return Ok(());
    }

    for (i, item) in items.iter().enumerate() {
        let (key, value) = item.extract::<(String, Bound<'_, PyAny>)>()?;

        // Add newline and indentation before each field (except first at root)
        if i > 0 || !is_root {
            output.push('\n');
            write_indent(output, depth, indent_size);
        }

        // Check if value is an array - need to write key with array header inline
        if value.is_instance_of::<PyList>() {
            if let Ok(list) = value.cast::<PyList>() {
                serialize_array_with_key(py, &key, &list, output, depth, delimiter, indent_size)?;
            }
        } else {
            // Encode key per Section 7.3
            serialize_key(&key, output);
            output.push(':');

            // Check if value needs nesting
            if value.is_instance_of::<PyDict>() {
                // Nested object
                if let Ok(nested_dict) = value.cast::<PyDict>() {
                    serialize_object(
                        py,
                        &nested_dict,
                        output,
                        depth + 1,
                        delimiter,
                        false,
                        indent_size,
                    )?;
                }
            } else {
                // Primitive: space after colon
                output.push(' ');
                serialize_value(py, &value, output, depth, delimiter, false, indent_size)?;
            }
        }
    }

    Ok(())
}

/// Serialize object key per TOON v2.0 Section 7.3
fn serialize_key(key: &str, output: &mut String) {
    // Key can be unquoted if matches: ^[A-Za-z_][\w.]*$
    if is_valid_unquoted_key(key) {
        output.push_str(key);
    } else {
        // Quote and escape
        output.push('"');
        for ch in key.chars() {
            match ch {
                '\\' => output.push_str("\\\\"),
                '"' => output.push_str("\\\""),
                '\n' => output.push_str("\\n"),
                '\r' => output.push_str("\\r"),
                '\t' => output.push_str("\\t"),
                _ => output.push(ch),
            }
        }
        output.push('"');
    }
}

/// Check if key can be unquoted
fn is_valid_unquoted_key(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }

    let mut chars = key.chars();
    let first = chars.next().unwrap();

    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }

    for ch in chars {
        if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '.' {
            return false;
        }
    }

    true
}

/// Serialize an array with its key inline (for arrays as object values)
fn serialize_array_with_key(
    py: Python,
    key: &str,
    list: &Bound<'_, PyList>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    indent_size: usize,
) -> PyResult<()> {
    let len = list.len();

    // Check if all elements are primitives
    let all_primitives = list.iter().all(|item| is_primitive(&item));

    if all_primitives {
        // Inline primitive array: key[N]: v1,v2,v3
        serialize_key(key, output);
        write!(output, "[{}]:", len).unwrap();

        if len > 0 {
            output.push(' ');
            for (i, item) in list.iter().enumerate() {
                if i > 0 {
                    output.push(delimiter);
                }
                serialize_value(py, &item, output, depth, delimiter, false, indent_size)?;
            }
        }
    } else {
        // Check for tabular format (Section 9.3)
        if let Some(fields) = detect_tabular(list)? {
            serialize_tabular_with_key(
                py,
                key,
                list,
                output,
                depth,
                delimiter,
                &fields,
                indent_size,
            )?;
        } else {
            // Expanded list format
            serialize_expanded_list_with_key(py, key, list, output, depth, delimiter, indent_size)?;
        }
    }

    Ok(())
}

/// Serialize an array (list) per TOON v2.0 Section 9
fn serialize_array(
    py: Python,
    list: &Bound<'_, PyList>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    is_root: bool,
    indent_size: usize,
) -> PyResult<()> {
    let len = list.len();

    // Check if all elements are primitives
    let all_primitives = list.iter().all(|item| is_primitive(&item));

    if all_primitives {
        // Inline primitive array: [N]: v1,v2,v3
        if !is_root {
            output.push('\n');
            write_indent(output, depth, indent_size);
        }
        write!(output, "[{}]:", len).unwrap();

        if len > 0 {
            output.push(' ');
            for (i, item) in list.iter().enumerate() {
                if i > 0 {
                    output.push(delimiter);
                }
                serialize_value(py, &item, output, depth, delimiter, false, indent_size)?;
            }
        }
    } else {
        // Check for tabular format (Section 9.3)
        if let Some(fields) = detect_tabular(list)? {
            serialize_tabular(
                py,
                list,
                output,
                depth,
                delimiter,
                &fields,
                is_root,
                indent_size,
            )?;
        } else {
            // Expanded list format
            serialize_expanded_list(py, list, output, depth, delimiter, is_root, indent_size)?;
        }
    }

    Ok(())
}

/// Check if value is a primitive (not dict or list)
fn is_primitive(obj: &Bound<'_, PyAny>) -> bool {
    !obj.is_instance_of::<PyDict>() && !obj.is_instance_of::<PyList>()
}

/// Detect if list qualifies for tabular format per Section 9.3
fn detect_tabular(list: &Bound<'_, PyList>) -> PyResult<Option<Vec<String>>> {
    if list.is_empty() {
        return Ok(None);
    }

    // All elements must be dicts
    let mut all_dicts = true;
    for item in list.iter() {
        if !item.is_instance_of::<PyDict>() {
            all_dicts = false;
            break;
        }
    }

    if !all_dicts {
        return Ok(None);
    }

    // Get keys from first dict
    let first_item = list.get_item(0)?;
    let first_dict = first_item.cast::<PyDict>()?;
    let first_keys: Vec<String> = first_dict
        .keys()
        .iter()
        .map(|k| k.extract::<String>())
        .collect::<Result<Vec<_>, _>>()?;

    if first_keys.is_empty() {
        return Ok(None);
    }

    // Check all dicts have same keys and all values are primitives
    for item in list.iter() {
        let dict = item.cast::<PyDict>()?;

        // Check key count
        if dict.len() != first_keys.len() {
            return Ok(None);
        }

        // Check all keys present and values are primitives
        for key in &first_keys {
            if let Ok(value) = dict.get_item(key) {
                if let Some(v) = value {
                    if !is_primitive(&v) {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }
        }
    }

    Ok(Some(first_keys))
}

/// Serialize array in tabular format per Section 9.3
fn serialize_tabular(
    py: Python,
    list: &Bound<'_, PyList>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    fields: &[String],
    is_root: bool,
    indent_size: usize,
) -> PyResult<()> {
    let len = list.len();

    // Header: [N]{f1,f2,f3}:
    if !is_root {
        output.push('\n');
        write_indent(output, depth, indent_size);
    }
    write!(output, "[{}]{{", len).unwrap();
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            output.push(delimiter);
        }
        serialize_key(field, output);
    }
    output.push_str("}:");

    // Rows: one per object
    for item in list.iter() {
        output.push('\n');
        write_indent(output, depth + 1, indent_size);

        let dict = item.cast::<PyDict>()?;
        for (i, field) in fields.iter().enumerate() {
            if i > 0 {
                output.push(delimiter);
            }
            let value = dict.get_item(field)?.unwrap();
            serialize_value(py, &value, output, depth + 1, delimiter, false, indent_size)?;
        }
    }

    Ok(())
}

/// Serialize array in tabular format with key (for object values)
fn serialize_tabular_with_key(
    py: Python,
    key: &str,
    list: &Bound<'_, PyList>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    fields: &[String],
    indent_size: usize,
) -> PyResult<()> {
    let len = list.len();

    // Header: key[N]{f1,f2,f3}:
    serialize_key(key, output);
    write!(output, "[{}]{{", len).unwrap();
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            output.push(delimiter);
        }
        serialize_key(field, output);
    }
    output.push_str("}:");

    // Rows: one per object
    for item in list.iter() {
        output.push('\n');
        write_indent(output, depth + 1, indent_size);

        let dict = item.cast::<PyDict>()?;
        for (i, field) in fields.iter().enumerate() {
            if i > 0 {
                output.push(delimiter);
            }
            let value = dict.get_item(field)?.unwrap();
            serialize_value(py, &value, output, depth + 1, delimiter, false, indent_size)?;
        }
    }

    Ok(())
}

/// Serialize array in expanded list format with key (for object values)
fn serialize_expanded_list_with_key(
    py: Python,
    key: &str,
    list: &Bound<'_, PyList>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    indent_size: usize,
) -> PyResult<()> {
    let len = list.len();

    // Header: key[N]:
    serialize_key(key, output);
    write!(output, "[{}]:", len).unwrap();

    // List items with "- " prefix
    for item in list.iter() {
        output.push('\n');
        write_indent(output, depth + 1, indent_size);
        output.push_str("- ");

        // Check if item itself is a primitive array
        if let Ok(inner_list) = item.cast::<PyList>() {
            if inner_list.iter().all(|x| is_primitive(&x)) {
                // Inline inner array
                let inner_len = inner_list.len();
                write!(output, "[{}]:", inner_len).unwrap();
                if inner_len > 0 {
                    output.push(' ');
                    for (i, inner_item) in inner_list.iter().enumerate() {
                        if i > 0 {
                            output.push(delimiter);
                        }
                        serialize_value(
                            py,
                            &inner_item,
                            output,
                            depth + 1,
                            delimiter,
                            false,
                            indent_size,
                        )?;
                    }
                }
            } else {
                // Nested complex array
                serialize_value(py, &item, output, depth + 1, delimiter, false, indent_size)?;
            }
        } else if let Ok(dict) = item.cast::<PyDict>() {
            // Object as list item - serialize with first field on same line as "-"
            serialize_list_item_object(py, &dict, output, depth + 1, delimiter, indent_size)?;
        } else {
            serialize_value(py, &item, output, depth + 1, delimiter, false, indent_size)?;
        }
    }

    Ok(())
}

/// Serialize array in expanded list format per Section 9.2/9.4
fn serialize_expanded_list(
    py: Python,
    list: &Bound<'_, PyList>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    is_root: bool,
    indent_size: usize,
) -> PyResult<()> {
    let len = list.len();

    // Header: [N]:
    if !is_root {
        output.push('\n');
        write_indent(output, depth, indent_size);
    }
    write!(output, "[{}]:", len).unwrap();

    // List items with "- " prefix
    for item in list.iter() {
        output.push('\n');
        write_indent(output, depth + 1, indent_size);
        output.push_str("- ");

        // Check if item itself is a primitive array
        if let Ok(inner_list) = item.cast::<PyList>() {
            if inner_list.iter().all(|x| is_primitive(&x)) {
                // Inline inner array
                let inner_len = inner_list.len();
                write!(output, "[{}]:", inner_len).unwrap();
                if inner_len > 0 {
                    output.push(' ');
                    for (i, inner_item) in inner_list.iter().enumerate() {
                        if i > 0 {
                            output.push(delimiter);
                        }
                        serialize_value(
                            py,
                            &inner_item,
                            output,
                            depth + 1,
                            delimiter,
                            false,
                            indent_size,
                        )?;
                    }
                }
            } else {
                // Nested complex array
                serialize_value(py, &item, output, depth + 1, delimiter, false, indent_size)?;
            }
        } else if let Ok(dict) = item.cast::<PyDict>() {
            // Object as list item - serialize with first field on same line as "-"
            serialize_list_item_object(py, &dict, output, depth + 1, delimiter, indent_size)?;
        } else {
            serialize_value(py, &item, output, depth + 1, delimiter, false, indent_size)?;
        }
    }

    Ok(())
}

/// Serialize an object as a list item with first field on same line as "- "
fn serialize_list_item_object(
    py: Python,
    dict: &Bound<'_, PyDict>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    indent_size: usize,
) -> PyResult<()> {
    let items: Vec<_> = dict.items().iter().collect();

    if items.is_empty() {
        return Ok(());
    }

    // First field on same line as "- "
    let (first_key, first_value) = items[0].extract::<(String, Bound<'_, PyAny>)>()?;

    // Check if first value is an array
    if first_value.is_instance_of::<PyList>() {
        if let Ok(list) = first_value.cast::<PyList>() {
            serialize_array_with_key(py, &first_key, &list, output, depth, delimiter, indent_size)?;
        }
    } else {
        serialize_key(&first_key, output);
        output.push(':');

        if first_value.is_instance_of::<PyDict>() {
            // Nested object
            if let Ok(nested_dict) = first_value.cast::<PyDict>() {
                serialize_object(
                    py,
                    &nested_dict,
                    output,
                    depth + 1,
                    delimiter,
                    false,
                    indent_size,
                )?;
            }
        } else {
            // Primitive
            output.push(' ');
            serialize_value(
                py,
                &first_value,
                output,
                depth,
                delimiter,
                false,
                indent_size,
            )?;
        }
    }

    // Remaining fields on new lines
    for item in items.iter().skip(1) {
        let (key, value) = item.extract::<(String, Bound<'_, PyAny>)>()?;

        output.push('\n');
        // Fields of list item object are indented one level deeper than the "- " line
        write_indent(output, depth + 1, indent_size);

        if value.is_instance_of::<PyList>() {
            if let Ok(list) = value.cast::<PyList>() {
                // Pass depth+1 so tabular rows are correctly indented at depth+2
                serialize_array_with_key(
                    py,
                    &key,
                    &list,
                    output,
                    depth + 1,
                    delimiter,
                    indent_size,
                )?;
            }
        } else {
            serialize_key(&key, output);
            output.push(':');

            if value.is_instance_of::<PyDict>() {
                if let Ok(nested_dict) = value.cast::<PyDict>() {
                    serialize_object(
                        py,
                        &nested_dict,
                        output,
                        depth + 1,
                        delimiter,
                        false,
                        indent_size,
                    )?;
                }
            } else {
                output.push(' ');
                serialize_value(py, &value, output, depth, delimiter, false, indent_size)?;
            }
        }
    }

    Ok(())
}

/// Write indentation (2 spaces per level per spec default)
fn write_indent(output: &mut String, depth: usize, indent_size: usize) {
    for _ in 0..depth * indent_size {
        output.push(' ');
    }
}

// ============================================================================
// DESERIALIZATION
// ============================================================================

struct Parser<'a> {
    lines: Vec<&'a str>,
    pos: usize,
    indent_size: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let lines: Vec<&str> = input.lines().collect();
        Parser {
            lines,
            pos: 0,
            indent_size: 0,
        }
    }

    fn detect_indent_size(&mut self) {
        // Auto-detect indent size by finding first indented line
        for line in &self.lines {
            if !line.trim().is_empty() && line.starts_with(' ') {
                let spaces = line.chars().take_while(|&c| c == ' ').count();
                if spaces > 0 {
                    self.indent_size = spaces;
                    return;
                }
            }
        }
        // Default to 2 if no indented lines found
        self.indent_size = 2;
    }

    fn parse(&mut self, py: Python) -> PyResult<Py<PyAny>> {
        // Auto-detect indentation size
        self.detect_indent_size();

        // Root form detection per TOON Spec v2.0 Section 5

        // Skip empty lines at start
        while self.pos < self.lines.len() && self.lines[self.pos].trim().is_empty() {
            self.pos += 1;
        }

        if self.pos >= self.lines.len() {
            // Empty document → empty object per TOON v2.0 Section 5
            return Ok(PyDict::new(py).into());
        }

        let first_line = self.lines[self.pos];
        let first_line_trimmed = first_line.trim();

        // Check if it's a root array header - can be [N]: or [N]{fields}:
        if first_line_trimmed.starts_with('[') && first_line_trimmed.contains(':') {
            // Make sure it's not an object field by checking there's no space before [
            if first_line == first_line_trimmed {
                return self.parse_root_array(py);
            }
        }

        // Check if it's a single primitive (one line, no colon, not a header)
        if self.lines.len() == 1 && !first_line_trimmed.contains(':') {
            return self.parse_primitive(py, first_line_trimmed);
        }

        // Otherwise, parse as object
        self.parse_object(py, 0)
    }

    fn parse_root_array(&mut self, py: Python) -> PyResult<Py<PyAny>> {
        let header = self.lines[self.pos];
        let (length, delimiter, fields) = self.parse_header(header)?;
        self.pos += 1;

        if let Some(field_names) = fields {
            // Tabular array
            self.parse_tabular_array(py, length, delimiter, &field_names, 1)
        } else {
            // Check if inline or expanded
            let header_trimmed = header.trim();
            if let Some(colon_pos) = header_trimmed.find("]:") {
                let after_colon = &header_trimmed[colon_pos + 2..].trim();
                if !after_colon.is_empty() {
                    // Inline primitive array (values on same line)
                    self.parse_inline_array(py, after_colon, delimiter)
                } else {
                    // Expanded list array (values on following lines)
                    self.parse_expanded_array(py, length, delimiter, 1)
                }
            } else {
                // Malformed
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "TOON parse error: Invalid array header",
                ))
            }
        }
    }

    fn parse_object(&mut self, py: Python, depth: usize) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);

        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            let line_depth = self.get_depth(line);

            if line_depth < depth {
                // End of this object
                break;
            }

            if line_depth > depth {
                // Shouldn't happen at start, skip
                self.pos += 1;
                continue;
            }

            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                self.pos += 1;
                continue;
            }

            // Parse key-value line
            if let Some(colon_pos) = line_trimmed.find(':') {
                let key_part = &line_trimmed[..colon_pos];
                let value_part = line_trimmed[colon_pos + 1..].trim();

                // Check if key contains array header (e.g., key[N] or key[N]{fields})
                if key_part.contains('[') && key_part.contains(']') {
                    // Array as object value
                    let value = self.parse_field_array(py, line_trimmed, depth)?;
                    // Extract key name before '['
                    let key_name = key_part.split('[').next().unwrap();
                    let key = self.parse_key(key_name)?;
                    dict.set_item(key, value)?;
                    // parse_field_array has already advanced self.pos
                    continue;
                }

                let key = self.parse_key(key_part)?;
                self.pos += 1;

                if value_part.is_empty() {
                    // Nested object or empty
                    if self.pos < self.lines.len() {
                        let next_depth = self.get_depth(self.lines[self.pos]);
                        if next_depth > depth {
                            // Nested object
                            let value = self.parse_object(py, depth + 1)?;
                            dict.set_item(key, value)?;
                        } else {
                            // Empty object
                            dict.set_item(key, PyDict::new(py))?;
                        }
                    } else {
                        // Empty object at end
                        dict.set_item(key, PyDict::new(py))?;
                    }
                } else {
                    // Primitive value
                    let value = self.parse_primitive(py, value_part)?;
                    dict.set_item(key, value)?;
                }
            } else {
                self.pos += 1;
            }
        }

        Ok(dict.into())
    }

    fn parse_field_array(
        &mut self,
        py: Python,
        header_line: &str,
        depth: usize,
    ) -> PyResult<Py<PyAny>> {
        let (length, delimiter, fields) = self.parse_header(header_line)?;
        // Advance position since we've consumed the header line
        self.pos += 1;

        if let Some(field_names) = fields {
            // Tabular array
            self.parse_tabular_array(py, length, delimiter, &field_names, depth + 1)
        } else {
            // Check if inline or expanded
            let header_trimmed = header_line.trim();
            if let Some(bracket_end) = header_trimmed.find("]:") {
                let after_colon = &header_trimmed[bracket_end + 2..].trim();
                if !after_colon.is_empty() {
                    // Inline primitive array (values on same line)
                    // Don't advance position - already done above
                    self.parse_inline_array(py, after_colon, delimiter)
                } else {
                    // Expanded list array (values on following lines)
                    self.parse_expanded_array(py, length, delimiter, depth + 1)
                }
            } else {
                // Malformed
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "TOON parse error: Invalid array header",
                ))
            }
        }
    }

    fn parse_header(&self, header: &str) -> PyResult<(usize, char, Option<Vec<String>>)> {
        let trimmed = header.trim();

        // Find bracket segment
        let bracket_start = trimmed.find('[').ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "TOON parse error: Invalid array header: missing '['",
            )
        })?;

        let bracket_end = trimmed.find(']').ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "TOON parse error: Invalid array header: missing ']'",
            )
        })?;

        let bracket_content = &trimmed[bracket_start + 1..bracket_end];

        // Parse length and delimiter
        if bracket_content.trim_start().starts_with('#') {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "TOON parse error: [#N] headers were removed in v2.0; use [N]",
            ));
        }

        let (length_str, delimiter) = if bracket_content.contains('\t') {
            let parts: Vec<&str> = bracket_content.split('\t').collect();
            (parts[0], '\t')
        } else if bracket_content.contains('|') {
            let parts: Vec<&str> = bracket_content.split('|').collect();
            (parts[0], '|')
        } else {
            (bracket_content, ',')
        };

        let length = length_str.parse::<usize>().map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "TOON parse error: Invalid array length: {}",
                length_str
            ))
        })?;

        // Check for field list (tabular)
        let fields = if let Some(brace_start) = trimmed[bracket_end..].find('{') {
            let brace_end = trimmed[bracket_end..].find('}').ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "TOON parse error: Invalid field list: missing '}'",
                )
            })?;

            let field_content = &trimmed[bracket_end + brace_start + 1..bracket_end + brace_end];
            let field_names: Vec<String> = field_content
                .split(delimiter)
                .map(|f| {
                    self.parse_key(f.trim())
                        .unwrap_or_else(|_| f.trim().to_string())
                })
                .collect();
            Some(field_names)
        } else {
            None
        };

        Ok((length, delimiter, fields))
    }

    fn parse_tabular_array(
        &mut self,
        py: Python,
        length: usize,
        delimiter: char,
        fields: &[String],
        expected_depth: usize,
    ) -> PyResult<Py<PyAny>> {
        let list = PyList::empty(py);

        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            let line_depth = self.get_depth(line);

            if line_depth < expected_depth {
                break;
            }

            if line_depth > expected_depth {
                self.pos += 1;
                continue;
            }

            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                self.pos += 1;
                continue;
            }

            // Check if this is a row (no colon or colon after delimiter)
            if !self.is_tabular_row(line_trimmed, delimiter) {
                // Not a row, end of tabular data
                break;
            }

            // Parse row
            let values = self.split_by_delimiter(line_trimmed, delimiter);
            let dict = PyDict::new(py);

            for (i, field) in fields.iter().enumerate() {
                if i < values.len() {
                    let value = self.parse_primitive(py, values[i])?;
                    dict.set_item(field, value)?;
                }
            }

            list.append(dict)?;
            self.pos += 1;
        }

        // Validate length if specified
        let actual_len = list.len();
        if length > 0 && actual_len != length {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "TOON parse error: Array declared length {} but found {} elements",
                length, actual_len
            )));
        }

        Ok(list.into())
    }

    fn parse_inline_array(
        &self,
        py: Python,
        values_str: &str,
        delimiter: char,
    ) -> PyResult<Py<PyAny>> {
        let list = PyList::empty(py);

        if values_str.is_empty() {
            return Ok(list.into());
        }

        let values = self.split_by_delimiter(values_str, delimiter);
        for value_str in values {
            let value = self.parse_primitive(py, value_str)?;
            list.append(value)?;
        }

        Ok(list.into())
    }

    fn parse_expanded_array(
        &mut self,
        py: Python,
        length: usize,
        delimiter: char,
        expected_depth: usize,
    ) -> PyResult<Py<PyAny>> {
        let list = PyList::empty(py);

        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            let line_depth = self.get_depth(line);

            if line_depth < expected_depth {
                break;
            }

            if line_depth > expected_depth {
                self.pos += 1;
                continue;
            }

            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                self.pos += 1;
                continue;
            }

            // Must start with "- "
            if !line_trimmed.starts_with("- ") {
                break;
            }

            let item_str = &line_trimmed[2..];
            self.pos += 1;

            // Check if it's an inline array
            if item_str.starts_with('[') && item_str.contains("]:") {
                let value = self.parse_inline_array(
                    py,
                    item_str.split("]: ").nth(1).unwrap_or(""),
                    delimiter,
                )?;
                list.append(value)?;
            } else if item_str.contains(':') {
                // Object as list item
                self.pos -= 1;
                let value = self.parse_list_item_object(py, expected_depth)?;
                list.append(value)?;
            } else {
                // Primitive
                let value = self.parse_primitive(py, item_str)?;
                list.append(value)?;
            }
        }

        // Validate length if specified
        let actual_len = list.len();
        if length > 0 && actual_len != length {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "TOON parse error: Array declared length {} but found {} elements",
                length, actual_len
            )));
        }

        Ok(list.into())
    }

    fn parse_list_item_object(&mut self, py: Python, list_depth: usize) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        let line = self.lines[self.pos];
        let line_trimmed = line.trim();

        // Parse first field from hyphen line
        if let Some(item_content) = line_trimmed.strip_prefix("- ") {
            if let Some(colon_pos) = item_content.find(':') {
                let key = self.parse_key(&item_content[..colon_pos])?;
                let value_part = item_content[colon_pos + 1..].trim();

                self.pos += 1;

                if value_part.is_empty() {
                    // Nested or subsequent fields
                    if self.pos < self.lines.len() {
                        let next_depth = self.get_depth(self.lines[self.pos]);
                        if next_depth > list_depth + 1 {
                            // Nested object
                            let value = self.parse_object(py, list_depth + 2)?;
                            dict.set_item(key, value)?;
                        }
                    }
                } else {
                    // Primitive value
                    let value = self.parse_primitive(py, value_part)?;
                    dict.set_item(key, value)?;
                }
            }
        }

        // Parse remaining fields at depth +1
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            let line_depth = self.get_depth(line);

            if line_depth <= list_depth {
                break;
            }

            if line_depth != list_depth + 1 {
                self.pos += 1;
                continue;
            }

            let line_trimmed = line.trim();
            if let Some(colon_pos) = line_trimmed.find(':') {
                let key_part = &line_trimmed[..colon_pos];
                let value_part = line_trimmed[colon_pos + 1..].trim();

                // Check if key contains array header (e.g., key[N] or key[N]{fields})
                if key_part.contains('[') && key_part.contains(']') {
                    // Array as object field
                    let value = self.parse_field_array(py, line_trimmed, list_depth + 1)?;
                    // Extract key name before '['
                    let key_name = key_part.split('[').next().unwrap();
                    let key = self.parse_key(key_name)?;
                    dict.set_item(key, value)?;
                    // parse_field_array has already advanced self.pos
                    continue;
                }

                let key = self.parse_key(key_part)?;
                self.pos += 1;

                if value_part.is_empty() {
                    // Nested
                    let value = self.parse_object(py, line_depth + 1)?;
                    dict.set_item(key, value)?;
                } else {
                    let value = self.parse_primitive(py, value_part)?;
                    dict.set_item(key, value)?;
                }
            } else {
                self.pos += 1;
            }
        }

        Ok(dict.into())
    }

    fn parse_primitive(&self, py: Python, s: &str) -> PyResult<Py<PyAny>> {
        let trimmed = s.trim();

        // Check if quoted
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            let unescaped = self.unescape_string(&trimmed[1..trimmed.len() - 1])?;
            return Ok(PyString::new(py, &unescaped).into());
        }

        // Unquoted - check type
        match trimmed {
            "null" => Ok(py.None()),
            "true" => Ok(PyBool::new(py, true).to_owned().into()),
            "false" => Ok(PyBool::new(py, false).to_owned().into()),
            _ => {
                // Try to parse as number
                if let Ok(i) = trimmed.parse::<i64>() {
                    Ok(PyInt::new(py, i).into())
                } else if let Ok(f) = trimmed.parse::<f64>() {
                    Ok(PyFloat::new(py, f).into())
                } else {
                    // String
                    Ok(PyString::new(py, trimmed).into())
                }
            }
        }
    }

    fn parse_key(&self, s: &str) -> PyResult<String> {
        let trimmed = s.trim();

        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            self.unescape_string(&trimmed[1..trimmed.len() - 1])
        } else {
            Ok(trimmed.to_string())
        }
    }

    fn unescape_string(&self, s: &str) -> PyResult<String> {
        let mut result = String::new();
        let mut chars = s.chars();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some(other) => {
                        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "Invalid escape sequence: \\{}",
                            other
                        )));
                    }
                    None => {
                        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "Unterminated escape sequence",
                        ));
                    }
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    fn get_depth(&self, line: &str) -> usize {
        let leading_spaces = line.len() - line.trim_start().len();
        if self.indent_size > 0 {
            leading_spaces / self.indent_size
        } else {
            0
        }
    }

    fn is_tabular_row(&self, line: &str, delimiter: char) -> bool {
        // A tabular row has no unquoted colon, or has delimiter before colon
        let mut in_quotes = false;
        let mut first_delim_pos = None;
        let mut first_colon_pos = None;

        for (i, ch) in line.chars().enumerate() {
            if ch == '"' {
                in_quotes = !in_quotes;
            } else if !in_quotes {
                if ch == delimiter && first_delim_pos.is_none() {
                    first_delim_pos = Some(i);
                }
                if ch == ':' && first_colon_pos.is_none() {
                    first_colon_pos = Some(i);
                }
            }
        }

        match (first_delim_pos, first_colon_pos) {
            (None, None) => true,        // No special chars - it's a row
            (Some(_), None) => true,     // Has delimiter, no colon - it's a row
            (None, Some(_)) => false,    // Has colon, no delimiter - key-value
            (Some(d), Some(c)) => d < c, // Delimiter before colon - it's a row
        }
    }

    fn split_by_delimiter<'b>(&self, s: &'b str, delimiter: char) -> Vec<&'b str> {
        let mut result = Vec::new();
        let mut start = 0;
        let mut in_quotes = false;
        let chars: Vec<char> = s.chars().collect();

        for i in 0..chars.len() {
            if chars[i] == '"' && (i == 0 || chars[i - 1] != '\\') {
                in_quotes = !in_quotes;
            } else if chars[i] == delimiter && !in_quotes {
                let segment = &s[start..i];
                result.push(segment.trim());
                start = i + delimiter.len_utf8();
            }
        }

        // Add last segment
        if start < s.len() {
            result.push(s[start..].trim());
        } else if start == s.len() && s.ends_with(delimiter) {
            result.push("");
        }

        result
    }
}

//! Native TOON v3.0 implementation
//!
//! This module implements TOON (Token-Oriented Object Notation) serialization
//! and deserialization according to the TOON Specification v3.0 (2025-11-24).
//!
//! Key features:
//! - Direct Python object integration (no JSON intermediate representation)
//! - Full TOON v3.0 spec compliance
//! - Tabular format support for uniform arrays of objects
//! - Configurable delimiters (comma, tab, pipe)
//! - Strict mode parsing with validation

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;

/// Serialization context for key folding options
#[derive(Clone)]
struct SerializationContext {
    key_folding: bool,
    flatten_depth: usize,
}

impl SerializationContext {
    fn new(key_folding: bool, flatten_depth: Option<usize>) -> Self {
        Self {
            key_folding,
            flatten_depth: flatten_depth.unwrap_or(usize::MAX),
        }
    }
}

/// Serialize a Python object to TOON format string.
///
/// Implements TOON Specification v3.0 encoding rules:
/// - Objects: key: value with proper indentation
/// - Arrays: headers with inline or tabular format
/// - Primitives: proper quoting and escaping
/// - Tabular optimization for uniform object arrays
pub fn serialize(
    py: Python,
    obj: &Bound<'_, PyAny>,
    indent: usize,
    delimiter: char,
    key_folding: Option<&str>,
    flatten_depth: Option<usize>,
) -> PyResult<String> {
    // Validate indent parameter (must be >= 2 per TOON spec v3.0)
    if indent < 2 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "indent must be >= 2 (TOON spec v3.0 uses 2-space indentation)",
        ));
    }

    // Special case: empty dict serializes as empty string per TOON spec
    if let Ok(dict) = obj.cast::<PyDict>() {
        if dict.is_empty() {
            return Ok(String::new());
        }
    }

    // Create serialization context
    let key_folding_enabled = key_folding == Some("safe");
    let ctx = SerializationContext::new(key_folding_enabled, flatten_depth);

    let mut output = String::new();
    serialize_value(py, obj, &mut output, 0, delimiter, true, indent, &ctx)?;

    Ok(output)
}

/// Deserialize a TOON format string to a Python object.
///
/// Implements TOON Specification v3.0 decoding rules:
/// - Automatic root form detection (object/array/primitive)
/// - Header parsing with delimiter detection
/// - Tabular array reconstruction
/// - Strict validation in strict mode
pub fn deserialize(
    py: Python,
    input: &str,
    strict: bool,
    expand_paths: &str,
    explicit_indent: Option<usize>,
) -> PyResult<Py<PyAny>> {
    let mut parser = Parser::new(input, strict, expand_paths, explicit_indent);
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
    ctx: &SerializationContext,
) -> PyResult<()> {
    if obj.is_none() {
        output.push_str("null");
    } else if let Ok(b) = obj.extract::<bool>() {
        output.push_str(if b { "true" } else { "false" });
    } else if let Ok(i) = obj.extract::<i64>() {
        write!(output, "{}", i).unwrap();
    } else if let Ok(f) = obj.extract::<f64>() {
        // TOON v3.0: normalize -0 to 0, no exponential notation
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
        serialize_array(
            py,
            &list,
            output,
            depth,
            delimiter,
            is_root,
            indent_size,
            ctx,
        )?;
    } else if let Ok(dict) = obj.cast::<PyDict>() {
        serialize_object(
            py,
            &dict,
            output,
            depth,
            delimiter,
            is_root,
            indent_size,
            ctx,
        )?;
    } else {
        // Unknown type → null (per spec Section 3)
        output.push_str("null");
    }
    Ok(())
}

/// Serialize a string with proper quoting and escaping per TOON v3.0 Section 7
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

/// Check if a string needs quoting per TOON v3.0 Section 7.2
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

/// Check if string looks numeric per TOON v3.0 Section 7.2
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

/// Write array header with delimiter per TOON v3.0 Section 6
fn write_array_header(output: &mut String, len: usize, delimiter: char, inline: bool) {
    write!(output, "[{}", len).unwrap();
    // Only include delimiter in header if it's not comma (default)
    if delimiter != ',' {
        output.push(delimiter);
    }
    output.push_str("]:");
    // Add space for inline arrays with elements
    if inline && len > 0 {
        output.push(' ');
    }
}

/// Write tabular array header with delimiter per TOON v3.0 Section 9.3
fn write_tabular_header(output: &mut String, len: usize, delimiter: char, fields: &[String]) {
    write!(output, "[{}", len).unwrap();
    // Only include delimiter in header if it's not comma (default)
    if delimiter != ',' {
        output.push(delimiter);
    }
    output.push_str("]{");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            output.push(delimiter);
        }
        serialize_key(field, output);
    }
    output.push_str("}:");
}

/// Serialize an object (dict) per TOON v3.0 Section 8
fn serialize_object(
    py: Python,
    dict: &Bound<'_, PyDict>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    is_root: bool,
    indent_size: usize,
    ctx: &SerializationContext,
) -> PyResult<()> {
    let items: Vec<_> = dict.items().iter().collect();

    if items.is_empty() {
        // Empty object: no output at root, empty line with key elsewhere
        return Ok(());
    }

    // Collect all top-level keys for collision detection
    let all_keys: HashSet<String> = items
        .iter()
        .map(|item| item.extract::<(String, Bound<'_, PyAny>)>().unwrap().0)
        .collect();

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
                serialize_array_with_key(
                    py,
                    &key,
                    &list,
                    output,
                    depth,
                    delimiter,
                    indent_size,
                    ctx,
                )?;
            }
        } else {
            // Try key folding if enabled (only at root level to avoid collisions)
            if ctx.key_folding && depth == 0 && value.is_instance_of::<PyDict>() {
                if let Ok(nested_dict) = value.cast::<PyDict>() {
                    if let Some((folded_key, final_value)) = try_fold_key_chain(
                        py,
                        &key,
                        &nested_dict,
                        depth,
                        ctx.flatten_depth,
                        &all_keys,
                    )? {
                        // Successfully folded - emit folded key
                        serialize_key(&folded_key, output);

                        if final_value.is_instance_of::<PyList>() {
                            // Folded to array - write array inline (no colon yet, array header will add it)
                            if let Ok(list) = final_value.cast::<PyList>() {
                                write_array_inline(
                                    py,
                                    &list,
                                    output,
                                    depth,
                                    delimiter,
                                    indent_size,
                                    ctx,
                                )?;
                            }
                        } else if final_value.is_instance_of::<PyDict>() {
                            // Folded to object - serialize nested without further folding
                            output.push(':');
                            if let Ok(dict) = final_value.cast::<PyDict>() {
                                // Create a context with folding disabled for nested serialization
                                let no_fold_ctx = SerializationContext {
                                    key_folding: false,
                                    flatten_depth: 0,
                                };
                                serialize_object(
                                    py,
                                    &dict,
                                    output,
                                    depth + 1,
                                    delimiter,
                                    false,
                                    indent_size,
                                    &no_fold_ctx,
                                )?;
                            }
                        } else {
                            // Folded to primitive
                            output.push(':');
                            output.push(' ');
                            serialize_value(
                                py,
                                &final_value,
                                output,
                                depth,
                                delimiter,
                                false,
                                indent_size,
                                ctx,
                            )?;
                        }
                        continue;
                    }
                }
            }

            // Standard serialization (no folding)
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
                        delimiter, // Use document delimiter per Section 11.1
                        false,
                        indent_size,
                        ctx,
                    )?;
                }
            } else {
                // Primitive: space after colon
                output.push(' ');
                // Use document delimiter per Section 11.1
                serialize_value(
                    py,
                    &value,
                    output,
                    depth,
                    delimiter,
                    false,
                    indent_size,
                    ctx,
                )?;
            }
        }
    }

    Ok(())
}

/// Serialize object key per TOON v3.0 Section 7.3
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

/// Try to fold a chain of single-key objects into a dot-notation key
/// Returns Some((folded_key, final_value)) if folding is possible, None otherwise
fn try_fold_key_chain<'py>(
    py: Python<'py>,
    start_key: &str,
    start_dict: &Bound<'py, PyDict>,
    depth: usize,
    max_depth: usize,
    sibling_keys: &HashSet<String>,
) -> PyResult<Option<(String, Bound<'py, PyAny>)>> {
    // If max_depth is 0 or 1, no folding is possible (need at least 2 keys to fold)
    if max_depth < 2 {
        return Ok(None);
    }

    // If the start_key requires quotes, it cannot be part of a folded chain
    if !is_valid_unquoted_key(start_key) {
        return Ok(None);
    }

    let mut key_chain = vec![start_key.to_string()];
    let mut current_dict = start_dict.clone();
    let mut current_value: Option<Bound<'py, PyAny>> = Some(current_dict.clone().into_any());

    loop {
        // Must have exactly one key
        if current_dict.len() != 1 {
            break;
        }

        // Get the single key and value
        let items: Vec<_> = current_dict.items().iter().collect();
        let (next_key, next_value) = items[0].extract::<(String, Bound<'_, PyAny>)>()?;

        // Check if the key can be represented unquoted (safe for folding)
        if !is_valid_unquoted_key(&next_key) {
            break;
        }

        key_chain.push(next_key.clone());
        current_value = Some(next_value.clone());

        // Check if we've reached the flatten depth limit
        if key_chain.len() >= max_depth {
            // Reached depth limit - return what we have folded so far
            let folded_key = key_chain.join(".");
            if sibling_keys.contains(&folded_key) {
                return Ok(None);
            }
            return Ok(Some((folded_key, next_value)));
        }

        // Check what the value is
        if next_value.is_instance_of::<PyDict>() {
            if let Ok(dict) = next_value.downcast::<PyDict>() {
                if dict.is_empty() {
                    // Empty dict - treat as terminal value
                    let folded_key = key_chain.join(".");
                    if sibling_keys.contains(&folded_key) {
                        return Ok(None);
                    }
                    return Ok(Some((folded_key, next_value)));
                }
                // Continue folding into non-empty nested dict
                current_dict = dict.clone();
            }
        } else {
            // Reached a non-object value (primitive or array)
            // Check for collision with literal keys at current level
            let folded_key = key_chain.join(".");
            if sibling_keys.contains(&folded_key) {
                // Collision detected - cannot fold
                return Ok(None);
            }

            // Folding is safe
            return Ok(Some((folded_key, next_value)));
        }
    }

    // Folding not applicable (multi-key object or depth limit reached)
    Ok(None)
}

/// Write an array inline (used for folded keys ending in arrays)
fn write_array_inline(
    py: Python,
    list: &Bound<'_, PyList>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    indent_size: usize,
    ctx: &SerializationContext,
) -> PyResult<()> {
    let len = list.len();
    let all_primitives = list.iter().all(|item| is_primitive(&item));

    if all_primitives {
        // Inline primitive array
        write_array_header(output, len, delimiter, true);
        if len > 0 {
            for (i, item) in list.iter().enumerate() {
                if i > 0 {
                    output.push(delimiter);
                }
                serialize_value(py, &item, output, depth, delimiter, false, indent_size, ctx)?;
            }
        }
    } else {
        // Check for tabular format
        if let Some(fields) = detect_tabular(list)? {
            // Tabular array
            write_tabular_header(output, len, delimiter, &fields);
            for item in list.iter() {
                output.push('\n');
                write_indent(output, depth + 1, indent_size);
                let dict = item.cast::<PyDict>()?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        output.push(delimiter);
                    }
                    let value = dict.get_item(field)?.unwrap();
                    serialize_value(
                        py,
                        &value,
                        output,
                        depth + 1,
                        delimiter,
                        false,
                        indent_size,
                        ctx,
                    )?;
                }
            }
        } else {
            // Expanded array format
            write_array_header(output, len, delimiter, false);
            for item in list.iter() {
                output.push('\n');
                write_indent(output, depth + 1, indent_size);
                output.push_str("- ");
                serialize_value(
                    py,
                    &item,
                    output,
                    depth + 1,
                    delimiter,
                    false,
                    indent_size,
                    ctx,
                )?;
            }
        }
    }
    Ok(())
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
    ctx: &SerializationContext,
) -> PyResult<()> {
    let len = list.len();

    // Check if all elements are primitives
    let all_primitives = list.iter().all(|item| is_primitive(&item));

    if all_primitives {
        // Inline primitive array: key[N]: v1,v2,v3
        serialize_key(key, output);
        write_array_header(output, len, delimiter, true);

        if len > 0 {
            for (i, item) in list.iter().enumerate() {
                if i > 0 {
                    output.push(delimiter);
                }
                serialize_value(py, &item, output, depth, delimiter, false, indent_size, ctx)?;
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
                ctx,
            )?;
        } else {
            // Expanded list format
            serialize_expanded_list_with_key(
                py,
                key,
                list,
                output,
                depth,
                delimiter,
                indent_size,
                ctx,
            )?;
        }
    }

    Ok(())
}

/// Serialize an array (list) per TOON v3.0 Section 9
fn serialize_array(
    py: Python,
    list: &Bound<'_, PyList>,
    output: &mut String,
    depth: usize,
    delimiter: char,
    is_root: bool,
    indent_size: usize,
    ctx: &SerializationContext,
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
        write_array_header(output, len, delimiter, true);

        if len > 0 {
            for (i, item) in list.iter().enumerate() {
                if i > 0 {
                    output.push(delimiter);
                }
                serialize_value(py, &item, output, depth, delimiter, false, indent_size, ctx)?;
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
                ctx,
            )?;
        } else {
            // Expanded list format
            serialize_expanded_list(
                py,
                list,
                output,
                depth,
                delimiter,
                is_root,
                indent_size,
                ctx,
            )?;
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
    ctx: &SerializationContext,
) -> PyResult<()> {
    let len = list.len();

    // Header: [N]{f1,f2,f3}:
    if !is_root {
        output.push('\n');
        write_indent(output, depth, indent_size);
    }
    write_tabular_header(output, len, delimiter, fields);

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
            serialize_value(
                py,
                &value,
                output,
                depth + 1,
                delimiter,
                false,
                indent_size,
                ctx,
            )?;
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
    ctx: &SerializationContext,
) -> PyResult<()> {
    let len = list.len();

    // Header: key[N]{f1,f2,f3}:
    serialize_key(key, output);
    write_tabular_header(output, len, delimiter, fields);

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
            serialize_value(
                py,
                &value,
                output,
                depth + 1,
                delimiter,
                false,
                indent_size,
                ctx,
            )?;
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
    ctx: &SerializationContext,
) -> PyResult<()> {
    let len = list.len();

    // Header: key[N]:
    serialize_key(key, output);
    write_array_header(output, len, delimiter, false);

    // List items with "- " prefix
    for item in list.iter() {
        output.push('\n');
        write_indent(output, depth + 1, indent_size);

        // Check if item is empty dict - encode as bare hyphen without space
        if let Ok(dict) = item.cast::<PyDict>() {
            if dict.is_empty() {
                output.push('-');
                continue;
            }
        }

        output.push_str("- ");

        // Check if item itself is a primitive array
        if let Ok(inner_list) = item.cast::<PyList>() {
            if inner_list.iter().all(|x| is_primitive(&x)) {
                // Inline inner array
                let inner_len = inner_list.len();
                write_array_header(output, inner_len, delimiter, true);
                if inner_len > 0 {
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
                            ctx,
                        )?;
                    }
                }
            } else {
                // Nested complex array
                serialize_value(
                    py,
                    &item,
                    output,
                    depth + 1,
                    delimiter,
                    false,
                    indent_size,
                    ctx,
                )?;
            }
        } else if let Ok(dict) = item.cast::<PyDict>() {
            // Object as list item - serialize with first field on same line as "-"
            serialize_list_item_object(py, &dict, output, depth + 1, delimiter, indent_size, ctx)?;
        } else {
            serialize_value(
                py,
                &item,
                output,
                depth + 1,
                delimiter,
                false,
                indent_size,
                ctx,
            )?;
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
    ctx: &SerializationContext,
) -> PyResult<()> {
    let len = list.len();

    // Header: [N]:
    if !is_root {
        output.push('\n');
        write_indent(output, depth, indent_size);
    }
    write_array_header(output, len, delimiter, false);

    // List items with "- " prefix
    for item in list.iter() {
        output.push('\n');
        write_indent(output, depth + 1, indent_size);

        // Check if item is empty dict - encode as bare hyphen without space
        if let Ok(dict) = item.cast::<PyDict>() {
            if dict.is_empty() {
                output.push('-');
                continue;
            }
        }

        output.push_str("- ");

        // Check if item itself is a primitive array
        if let Ok(inner_list) = item.cast::<PyList>() {
            if inner_list.iter().all(|x| is_primitive(&x)) {
                // Inline inner array
                let inner_len = inner_list.len();
                write_array_header(output, inner_len, delimiter, true);
                if inner_len > 0 {
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
                            ctx,
                        )?;
                    }
                }
            } else {
                // Nested complex array - header should be on same line as hyphen
                if let Some(fields) = detect_tabular(&inner_list)? {
                    // Tabular format: [N]{f1,f2}:
                    write_tabular_header(output, inner_list.len(), delimiter, &fields);
                    // Rows at depth + 2
                    for row_item in inner_list.iter() {
                        output.push('\n');
                        write_indent(output, depth + 2, indent_size);
                        let dict = row_item.cast::<PyDict>()?;
                        for (i, field) in fields.iter().enumerate() {
                            if i > 0 {
                                output.push(delimiter);
                            }
                            let value = dict.get_item(field)?.unwrap();
                            serialize_value(
                                py,
                                &value,
                                output,
                                depth + 2,
                                delimiter,
                                false,
                                indent_size,
                                ctx,
                            )?;
                        }
                    }
                } else {
                    // Expanded list format: [N]:
                    write_array_header(output, inner_list.len(), delimiter, false);
                    // Items at depth + 2 with hyphen
                    for list_item in inner_list.iter() {
                        output.push('\n');
                        write_indent(output, depth + 2, indent_size);
                        output.push_str("- ");
                        if let Ok(item_dict) = list_item.cast::<PyDict>() {
                            serialize_list_item_object(
                                py,
                                &item_dict,
                                output,
                                depth + 2,
                                delimiter,
                                indent_size,
                                ctx,
                            )?;
                        } else {
                            serialize_value(
                                py,
                                &list_item,
                                output,
                                depth + 2,
                                delimiter,
                                false,
                                indent_size,
                                ctx,
                            )?;
                        }
                    }
                }
            }
        } else if let Ok(dict) = item.cast::<PyDict>() {
            // Object as list item - serialize with first field on same line as "-"
            serialize_list_item_object(py, &dict, output, depth + 1, delimiter, indent_size, ctx)?;
        } else {
            serialize_value(
                py,
                &item,
                output,
                depth + 1,
                delimiter,
                false,
                indent_size,
                ctx,
            )?;
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
    ctx: &SerializationContext,
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
            // For both tabular and list format, items must be at depth + 2
            // (one level deeper than the "- " line)
            // So we pass depth + 1 to serialize_array_with_key which will add another +1
            serialize_array_with_key(
                py,
                &first_key,
                &list,
                output,
                depth + 1,
                delimiter,
                indent_size,
                ctx,
            )?;
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
                    depth + 2,
                    delimiter,
                    false,
                    indent_size,
                    ctx,
                )?;
            }
        } else {
            // Primitive
            output.push(' ');
            serialize_value(
                py,
                &first_value,
                output,
                depth + 1,
                delimiter,
                false,
                indent_size,
                ctx,
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
                    ctx,
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
                        depth + 2,
                        delimiter,
                        false,
                        indent_size,
                        ctx,
                    )?;
                }
            } else {
                output.push(' ');
                serialize_value(
                    py,
                    &value,
                    output,
                    depth + 1,
                    delimiter,
                    false,
                    indent_size,
                    ctx,
                )?;
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

/// Check if a segment is a valid identifier for path expansion (unquoted alphanumeric with dots/underscores)
fn is_valid_identifier_segment(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Must start with letter or underscore
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    // Rest can be alphanumeric, underscore, or dot
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' && c != '.' {
            return false;
        }
    }
    true
}

/// Check if setting a key would conflict with existing path-expanded keys
fn check_key_conflict(
    target: &Bound<'_, PyDict>,
    key: &str,
    new_value: &Bound<'_, PyAny>,
    strict: bool,
) -> PyResult<()> {
    if !strict {
        return Ok(());
    }

    if let Some(existing) = target.get_item(key)? {
        // Check type compatibility
        let existing_is_dict = existing.cast::<PyDict>().is_ok();
        let new_is_dict = new_value.cast::<PyDict>().is_ok();
        let existing_is_list = existing.cast::<PyList>().is_ok();
        let new_is_list = new_value.cast::<PyList>().is_ok();

        if (existing_is_dict && !new_is_dict)
            || (!existing_is_dict && new_is_dict)
            || (existing_is_list && !new_is_list)
            || (!existing_is_list && new_is_list)
        {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "TOON parse error: Path expansion conflict at key '{}'",
                key
            )));
        }
    }

    Ok(())
}

/// Split a dotted key into segments for path expansion
/// Returns None if the key should not be expanded (e.g., contains invalid segments)
fn split_dotted_key(key: &str) -> Option<Vec<&str>> {
    if !key.contains('.') {
        return None;
    }

    let segments: Vec<&str> = key.split('.').collect();

    // All segments must be valid identifiers
    for segment in &segments {
        if !is_valid_identifier_segment(segment) {
            return None;
        }
    }

    Some(segments)
}

/// Deep merge a value into an existing object at the given path
/// Returns Ok if successful, Err if there's a type conflict in strict mode
fn deep_merge_path(
    py: Python,
    target: &Bound<'_, PyDict>,
    path_segments: &[&str],
    value: Py<PyAny>,
    strict: bool,
) -> PyResult<()> {
    if path_segments.is_empty() {
        return Ok(());
    }

    if path_segments.len() == 1 {
        // Last segment - set the value
        let key = path_segments[0];

        if strict && target.contains(key)? {
            // In strict mode, check for conflicts
            let existing = target.get_item(key)?;
            if let Some(existing_val) = existing {
                // Check if types are incompatible
                let existing_is_dict = existing_val.cast::<PyDict>().is_ok();
                let new_is_dict = value.bind(py).cast::<PyDict>().is_ok();
                let existing_is_list = existing_val.cast::<PyList>().is_ok();
                let new_is_list = value.bind(py).cast::<PyList>().is_ok();

                if (existing_is_dict && !new_is_dict)
                    || (!existing_is_dict && new_is_dict)
                    || (existing_is_list && !new_is_list)
                    || (!existing_is_list && new_is_list)
                {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "TOON parse error: Path expansion conflict at key '{}'",
                        key
                    )));
                }
            }
        }

        target.set_item(key, value)?;
        return Ok(());
    }

    // Navigate/create intermediate objects
    let first_segment = path_segments[0];
    let remaining_segments = &path_segments[1..];

    let next_obj = if let Some(existing) = target.get_item(first_segment)? {
        // Check if it's a dict
        if let Ok(dict) = existing.cast::<PyDict>() {
            dict.clone()
        } else {
            // Type conflict - existing value is not an object
            if strict {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "TOON parse error: Path expansion conflict at key '{}'",
                    first_segment
                )));
            }
            // In non-strict mode, overwrite with new object (LWW)
            let new_dict = PyDict::new(py);
            target.set_item(first_segment, &new_dict)?;
            new_dict
        }
    } else {
        // Create new intermediate object
        let new_dict = PyDict::new(py);
        target.set_item(first_segment, &new_dict)?;
        new_dict
    };

    deep_merge_path(py, &next_obj, remaining_segments, value, strict)
}

struct Parser<'a> {
    lines: Vec<&'a str>,
    pos: usize,
    indent_size: usize,
    explicit_indent: Option<usize>,
    strict: bool,
    expand_paths: &'a str,
}

impl<'a> Parser<'a> {
    fn new(
        input: &'a str,
        strict: bool,
        expand_paths: &'a str,
        explicit_indent: Option<usize>,
    ) -> Self {
        let lines: Vec<&str> = input.lines().collect();
        Parser {
            lines,
            pos: 0,
            indent_size: 0,
            explicit_indent,
            strict,
            expand_paths,
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

    fn validate_indentation(&self, line: &str) -> PyResult<()> {
        if !self.strict {
            return Ok(());
        }

        // Skip validation for lines that are only whitespace (empty lines)
        if line.trim().is_empty() {
            return Ok(());
        }

        let indent_len = line.len() - line.trim_start().len();
        let indent_part = &line[..indent_len];

        if indent_part.contains('\t') {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "TOON parse error: Tabs are not allowed in indentation",
            ));
        }

        // Use explicit_indent if provided, otherwise use auto-detected indent_size
        let check_indent = if let Some(explicit) = self.explicit_indent {
            explicit
        } else {
            self.indent_size
        };

        if check_indent > 0 && indent_len % check_indent != 0 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "TOON parse error: Indentation {} is not a multiple of indent size {}",
                indent_len, check_indent
            )));
        }

        Ok(())
    }

    fn parse(&mut self, py: Python) -> PyResult<Py<PyAny>> {
        // Auto-detect indentation size
        self.detect_indent_size();

        // Root form detection per TOON Spec v3.0 Section 5

        // Skip empty lines at start
        while self.pos < self.lines.len() && self.lines[self.pos].trim().is_empty() {
            self.pos += 1;
        }

        if self.pos >= self.lines.len() {
            // Empty document → empty object per TOON v3.0 Section 5
            return Ok(PyDict::new(py).into());
        }

        let first_line = self.lines[self.pos];
        self.validate_indentation(first_line)?;
        let first_line_trimmed = first_line.trim();

        // Check if it's a root array header - can be [N]: or [N]{fields}:
        if first_line_trimmed.starts_with('[') && first_line_trimmed.contains(':') {
            // Make sure it's not an object field by checking there's no space before [
            if first_line == first_line_trimmed {
                return self.parse_root_array(py);
            }
        }

        // Check if it's a single primitive (one line, no colon outside quotes, not a header)
        if self.lines.len() == 1 && self.find_key_value_colon(first_line_trimmed).is_none() {
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
                    self.parse_inline_array(py, after_colon, delimiter, length)
                } else {
                    // Expanded list array (values on following lines)
                    self.parse_expanded_array(py, length, 1)
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
            self.validate_indentation(line)?;

            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                // Blank line - check if there are more fields at this depth
                let mut lookahead = self.pos + 1;
                while lookahead < self.lines.len() && self.lines[lookahead].trim().is_empty() {
                    lookahead += 1;
                }

                if lookahead < self.lines.len() {
                    let next_depth = self.get_depth(self.lines[lookahead]);
                    if next_depth >= depth {
                        // More fields at this depth, skip blank line and continue
                        self.pos += 1;
                        continue;
                    }
                }

                // No more fields at this depth, end object
                break;
            }

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

            // Parse key-value line
            if let Some(colon_pos) = self.find_key_value_colon(line_trimmed) {
                let key_part = &line_trimmed[..colon_pos];
                let value_part = line_trimmed[colon_pos + 1..].trim();

                // Check if this is an array syntax like: key[N] or "key"[N] or key[N]{fields}
                // We need to check if brackets appear OUTSIDE any quoted portion for keys
                // Examples:
                //   "key"[3]: ... -> has array syntax (brackets after quoted key)
                //   "[index]": ... -> no array syntax (brackets inside quotes, no unquoted brackets)
                //   "key[test]"[3]: ... -> has array syntax (brackets after closing quote)
                //   items[2]{"a|b"}: ... -> has array syntax (brackets in key, quotes in field list)

                // Find the position of the first opening quote and bracket
                let first_quote_pos = key_part.find('"');
                let first_bracket_pos = key_part.find('[');

                let has_array_syntax = match (first_quote_pos, first_bracket_pos) {
                    (None, Some(_)) => {
                        // No quotes, has brackets - check if closing bracket exists
                        key_part.contains(']')
                    }
                    (Some(q), Some(b)) if b < q => {
                        // Bracket comes before first quote - array syntax (e.g., key[N]{...})
                        key_part.contains(']')
                    }
                    (Some(q), Some(b)) if b > q => {
                        // Quote comes before bracket - check if bracket is after last quote
                        if let Some(last_quote) = key_part.rfind('"') {
                            key_part[last_quote + 1..].contains('[')
                                && key_part[last_quote + 1..].contains(']')
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                // Check if key contains array header (e.g., key[N] or key[N]{fields})
                if has_array_syntax {
                    // Array as object value
                    let value = self.parse_field_array(py, line_trimmed, depth)?;

                    // Extract key name before the array bracket
                    // For quoted keys like "key"[3] or "key[test]"[3], we want the full quoted part
                    // For unquoted keys like key[N], we want everything before the bracket
                    let key_name = if key_part.starts_with('"') {
                        // Quoted key - find the closing quote
                        if let Some(close_quote) = key_part[1..].find('"').map(|p| p + 1) {
                            &key_part[..close_quote + 1]
                        } else {
                            key_part.split('[').next().unwrap()
                        }
                    } else if let Some(first_bracket) = key_part.find('[') {
                        &key_part[..first_bracket]
                    } else {
                        key_part
                    };

                    // Check for path expansion on the key name
                    let (should_expand, was_quoted) = self.should_expand_key(key_name);
                    if should_expand {
                        if let Some(segments) = split_dotted_key(key_name) {
                            deep_merge_path(py, &dict, &segments, value, self.strict)?;
                        } else {
                            check_key_conflict(&dict, key_name, value.bind(py), self.strict)?;
                            let key = self.parse_key(key_name)?;
                            dict.set_item(key, value)?;
                        }
                    } else {
                        let key = if was_quoted {
                            // Remove quotes from quoted key
                            self.parse_key(key_name)?
                        } else {
                            key_name.to_string()
                        };
                        check_key_conflict(&dict, &key, value.bind(py), self.strict)?;
                        dict.set_item(key, value)?;
                    }
                    // parse_field_array has already advanced self.pos
                    continue;
                }

                // Parse the key and check if it was quoted
                let (should_expand, was_quoted) = self.should_expand_key(key_part);
                let parsed_key = self.parse_key(key_part)?;
                self.pos += 1;

                if value_part.is_empty() {
                    // Nested object or empty
                    let value = if self.pos < self.lines.len() {
                        let next_line = self.lines[self.pos];
                        let next_depth = self.get_depth(next_line);

                        // In non-strict mode, use actual indentation comparison
                        // to handle non-multiple indentation correctly
                        let is_nested = if !self.strict && self.explicit_indent.is_none() {
                            let current_indent = self.get_indent_spaces(line);
                            let next_indent = self.get_indent_spaces(next_line);
                            let next_trimmed = next_line.trim();
                            next_indent > current_indent
                                && !next_trimmed.is_empty()
                                && !next_trimmed.starts_with('-')
                        } else {
                            next_depth > depth
                        };

                        if is_nested {
                            // Nested object - in non-strict mode with auto-detected indent,
                            // use the actual depth of the next line instead of depth+1
                            let nested_depth = if !self.strict && self.explicit_indent.is_none() {
                                next_depth
                            } else {
                                depth + 1
                            };
                            self.parse_object(py, nested_depth)?
                        } else {
                            // Empty object
                            PyDict::new(py).into()
                        }
                    } else {
                        // Empty object at end
                        PyDict::new(py).into()
                    };

                    // Apply path expansion if enabled
                    if should_expand && !was_quoted {
                        if let Some(segments) = split_dotted_key(&parsed_key) {
                            deep_merge_path(py, &dict, &segments, value, self.strict)?;
                        } else {
                            check_key_conflict(&dict, &parsed_key, value.bind(py), self.strict)?;
                            dict.set_item(parsed_key, value)?;
                        }
                    } else {
                        check_key_conflict(&dict, &parsed_key, value.bind(py), self.strict)?;
                        dict.set_item(parsed_key, value)?;
                    }
                } else {
                    // Primitive value
                    let value = self.parse_primitive(py, value_part)?;

                    // Apply path expansion if enabled
                    if should_expand && !was_quoted {
                        if let Some(segments) = split_dotted_key(&parsed_key) {
                            deep_merge_path(py, &dict, &segments, value, self.strict)?;
                        } else {
                            check_key_conflict(&dict, &parsed_key, value.bind(py), self.strict)?;
                            dict.set_item(parsed_key, value)?;
                        }
                    } else {
                        check_key_conflict(&dict, &parsed_key, value.bind(py), self.strict)?;
                        dict.set_item(parsed_key, value)?;
                    }
                }
            } else {
                // Missing colon error (Section 14.2)
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "TOON parse error: Missing colon in line: {}",
                    line_trimmed
                )));
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
                let after_colon = header_trimmed[bracket_end + 2..].trim();
                if !after_colon.is_empty() {
                    // Inline primitive array (values on same line)
                    // Don't advance position - already done above
                    self.parse_inline_array(py, after_colon, delimiter, length)
                } else {
                    // Expanded list array (values on following lines)
                    self.parse_expanded_array(py, length, depth + 1)
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

        // Find bracket segment that's part of array syntax (outside quotes)
        // For "key[test]"[3]:, we want to find the [3], not [test]
        let bracket_start = self.find_array_bracket_start(trimmed).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "TOON parse error: Invalid array header: missing '['",
            )
        })?;

        let bracket_end = trimmed[bracket_start..]
            .find(']')
            .map(|pos| pos + bracket_start)
            .ok_or_else(|| {
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
        // Look for {fieldlist} immediately after ] and before :
        // e.g., items[3]{id,name}: not items[3]: "{key}"
        // Need to find colon that's outside any quotes
        let substring_after_bracket = &trimmed[bracket_end..];
        let colon_pos = self
            .find_unquoted_char(substring_after_bracket, ':')
            .unwrap_or(substring_after_bracket.len());
        let fields = if let Some(brace_start) = substring_after_bracket[..colon_pos].find('{') {
            let brace_end_relative =
                substring_after_bracket[..colon_pos]
                    .find('}')
                    .ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "TOON parse error: Invalid field list: missing '}'",
                        )
                    })?;

            let field_content = &substring_after_bracket[brace_start + 1..brace_end_relative];
            let field_parts = self.split_by_delimiter(field_content, delimiter);
            let field_names: Vec<String> = field_parts
                .iter()
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
            let line_trimmed = line.trim();

            // Check depth before blank line check
            if !line_trimmed.is_empty() {
                self.validate_indentation(line)?;
                let line_depth = self.get_depth(line);

                if line_depth < expected_depth {
                    break;
                }

                if line_depth > expected_depth {
                    self.pos += 1;
                    continue;
                }
            } else {
                // Blank line - check if it's actually inside the array or after it ends
                // Look ahead to see if next non-blank line is at lower depth
                let mut lookahead = self.pos + 1;
                while lookahead < self.lines.len() && self.lines[lookahead].trim().is_empty() {
                    lookahead += 1;
                }

                if lookahead < self.lines.len() {
                    let next_depth = self.get_depth(self.lines[lookahead]);
                    if next_depth < expected_depth {
                        // Blank line is after array ends, not inside it
                        break;
                    }
                }

                // Blank line is inside array
                if self.strict {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "TOON parse error: Blank line inside array",
                    ));
                }
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

            // Strict mode: check width (Section 14.1)
            if values.len() != fields.len() {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "TOON parse error: Tabular row has {} values but header defines {} fields",
                    values.len(),
                    fields.len()
                )));
            }

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
        length: usize,
    ) -> PyResult<Py<PyAny>> {
        let list = PyList::empty(py);

        if values_str.is_empty() {
            if length > 0 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "TOON parse error: Array declared length {} but found 0 elements",
                    length
                )));
            }
            return Ok(list.into());
        }

        let values = self.split_by_delimiter(values_str, delimiter);

        // Strict mode: check length (Section 14.1)
        if length > 0 && values.len() != length {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "TOON parse error: Array declared length {} but found {} elements",
                length,
                values.len()
            )));
        }

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
        expected_depth: usize,
    ) -> PyResult<Py<PyAny>> {
        let list = PyList::empty(py);

        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            let line_trimmed = line.trim();

            // Check depth before blank line check
            if !line_trimmed.is_empty() {
                self.validate_indentation(line)?;
                let line_depth = self.get_depth(line);

                if line_depth < expected_depth {
                    break;
                }

                if line_depth > expected_depth {
                    self.pos += 1;
                    continue;
                }
            } else {
                // Blank line - check if it's actually inside the array or after it ends
                // Look ahead to see if next non-blank line is at lower depth
                let mut lookahead = self.pos + 1;
                while lookahead < self.lines.len() && self.lines[lookahead].trim().is_empty() {
                    lookahead += 1;
                }

                if lookahead < self.lines.len() {
                    let next_depth = self.get_depth(self.lines[lookahead]);
                    if next_depth < expected_depth {
                        // Blank line is after array ends, not inside it
                        break;
                    }
                }

                // Blank line is inside array
                if self.strict {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "TOON parse error: Blank line inside array",
                    ));
                }
                self.pos += 1;
                continue;
            }

            // Must start with "-" (may or may not have space after)
            if !line_trimmed.starts_with('-') {
                break;
            }

            let item_str = if line_trimmed.len() > 1 && line_trimmed.chars().nth(1) == Some(' ') {
                &line_trimmed[2..]
            } else if line_trimmed.len() == 1 {
                "" // Just a dash, empty item
            } else {
                &line_trimmed[1..] // Dash with no space after
            };
            self.pos += 1;

            // Check if empty (trailing hyphen case: "  - " or "  -")
            if item_str.is_empty() {
                // Empty object per spec Section 9.4
                let empty_obj = PyDict::new(py);
                list.append(empty_obj)?;
                continue;
            }

            // Check if it's an inline array header: [N]: or [N|]: etc
            if item_str.starts_with('[') && item_str.contains("]:") {
                // Parse header to get correct delimiter for nested array
                let header_part = item_str.split("]:").next().unwrap();
                let header_with_bracket = format!("{}]", header_part); // Reconstruct header for parsing
                let (inner_len, inner_delim, _) = self.parse_header(&header_with_bracket)?;

                let after_colon = item_str.split("]:").nth(1).unwrap_or("").trim();

                if after_colon.is_empty() {
                    // Expanded nested array: "- [2]:" followed by indented items
                    // Parse as expanded array at depth expected_depth + 1
                    let value = self.parse_expanded_array(py, inner_len, expected_depth + 1)?;
                    list.append(value)?;
                } else {
                    // Inline nested array: "- [2]: a,b"
                    let value = self.parse_inline_array(py, after_colon, inner_delim, inner_len)?;
                    list.append(value)?;
                }
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
                let key_part = &item_content[..colon_pos];
                let value_part = item_content[colon_pos + 1..].trim();

                // Check if this is an array syntax (brackets outside quotes)
                let quote_end_pos = key_part.rfind('"');

                let has_array_syntax = if let Some(quote_end) = quote_end_pos {
                    key_part[quote_end + 1..].contains('[')
                        && key_part[quote_end + 1..].contains(']')
                } else {
                    key_part.contains('[') && key_part.contains(']')
                };

                // Check if key contains array header (e.g., key[N] or key[N]{fields})
                if has_array_syntax {
                    // Array as object field on hyphen line
                    // Section 10: Tabular rows MUST appear at depth +2
                    // parse_field_array expects depth of the field.
                    // If we pass list_depth + 1, it expects rows at list_depth + 2.
                    // This matches the spec for tabular arrays.
                    let value = self.parse_field_array(py, item_content, list_depth + 1)?;

                    // Extract key name before the array bracket
                    let key_name = if let Some(quote_end) = quote_end_pos {
                        &key_part[..quote_end + 1]
                    } else {
                        key_part.split('[').next().unwrap()
                    };
                    let key = self.parse_key(key_name)?;
                    dict.set_item(key, value)?;

                    // parse_field_array has already advanced self.pos
                } else {
                    let key = self.parse_key(key_part)?;
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
        }

        // Parse remaining fields at depth +1
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            self.validate_indentation(line)?;
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

                // Check if this is an array syntax (brackets outside quotes)
                let quote_end_pos = key_part.rfind('"');

                let has_array_syntax = if let Some(quote_end) = quote_end_pos {
                    key_part[quote_end + 1..].contains('[')
                        && key_part[quote_end + 1..].contains(']')
                } else {
                    key_part.contains('[') && key_part.contains(']')
                };

                // Check if key contains array header (e.g., key[N] or key[N]{fields})
                if has_array_syntax {
                    // Array as object field
                    let value = self.parse_field_array(py, line_trimmed, list_depth + 1)?;

                    // Extract key name before the array bracket
                    let key_name = if let Some(quote_end) = quote_end_pos {
                        &key_part[..quote_end + 1]
                    } else {
                        key_part.split('[').next().unwrap()
                    };
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
        if trimmed.starts_with('"') {
            if !trimmed.ends_with('"') || trimmed.len() < 2 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "TOON parse error: Unterminated string",
                ));
            }
            let unescaped = self.unescape_string(&trimmed[1..trimmed.len() - 1])?;
            return Ok(PyString::new(py, &unescaped).into());
        }

        // Unquoted - check type
        match trimmed {
            "null" => Ok(py.None()),
            "true" => Ok(PyBool::new(py, true).to_owned().into()),
            "false" => Ok(PyBool::new(py, false).to_owned().into()),
            _ => {
                // Check for forbidden leading zeros: 0 followed by digits
                // Section 4: "05", "0001", "-05" are strings.
                // "0.5", "0e1", "-0.5" are numbers.
                let check_s = if trimmed.starts_with('-') {
                    &trimmed[1..]
                } else {
                    trimmed
                };

                if check_s.len() > 1
                    && check_s.starts_with('0')
                    && check_s.chars().nth(1).unwrap().is_ascii_digit()
                {
                    // Treat as string
                    return Ok(PyString::new(py, trimmed).into());
                }

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

    /// Check if a key should be expanded based on expand_paths mode
    /// Returns (should_expand, was_quoted)
    fn should_expand_key(&self, key: &str) -> (bool, bool) {
        let trimmed = key.trim();
        let was_quoted = trimmed.starts_with('"') && trimmed.ends_with('"');

        match self.expand_paths {
            "off" | "never" => (false, was_quoted),
            "safe" => {
                // In safe mode, only expand unquoted keys
                (!was_quoted, was_quoted)
            }
            "always" => (true, was_quoted),
            _ => (false, was_quoted), // Default to off for unknown modes
        }
    }

    /// Find the position of the key-value separator colon, accounting for quoted keys
    /// Find the position of the first '[' that's part of array syntax (outside quotes)
    /// For "key[test]"[3]:, returns the position of the second '['
    fn find_array_bracket_start(&self, line: &str) -> Option<usize> {
        let mut in_quotes = false;
        let mut escape_next = false;

        for (i, ch) in line.chars().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                in_quotes = !in_quotes;
                continue;
            }

            if !in_quotes && ch == '[' {
                return Some(i);
            }
        }

        None
    }

    fn find_key_value_colon(&self, line: &str) -> Option<usize> {
        let mut in_quotes = false;
        let mut escape_next = false;

        for (i, ch) in line.chars().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                in_quotes = !in_quotes;
                continue;
            }

            if ch == ':' && !in_quotes {
                return Some(i);
            }
        }

        None
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
        // Use explicit_indent if provided, otherwise use auto-detected indent_size
        let indent_to_use = if let Some(explicit) = self.explicit_indent {
            explicit
        } else {
            self.indent_size
        };
        if indent_to_use > 0 {
            leading_spaces / indent_to_use
        } else {
            0
        }
    }

    fn get_indent_spaces(&self, line: &str) -> usize {
        line.len() - line.trim_start().len()
    }

    fn is_tabular_row(&self, line: &str, delimiter: char) -> bool {
        // A tabular row has no unquoted colon, or has delimiter before colon
        // Single-field tabular arrays have no delimiters at all - just values
        let mut in_quotes = false;
        let mut escape_next = false;
        let mut first_delim_pos = None;
        let mut first_colon_pos = None;

        for (i, ch) in line.chars().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                continue;
            }

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
            (None, None) => true,     // No special chars - it's a row (single field case)
            (Some(_), None) => true,  // Has delimiter, no colon - it's a row
            (None, Some(_)) => false, // Has colon, no delimiter - key-value
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

    fn find_unquoted_char(&self, s: &str, target: char) -> Option<usize> {
        let mut in_quotes = false;
        let mut escape_next = false;

        for (i, ch) in s.chars().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                in_quotes = !in_quotes;
            } else if ch == target && !in_quotes {
                return Some(i);
            }
        }

        None
    }
}

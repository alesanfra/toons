use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateTime, PyDict, PyList, PyTime};
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;

/// Serialization context for key folding options
#[derive(Clone)]
pub struct SerializationContext {
    pub key_folding: bool,
    pub flatten_depth: usize,
}

impl SerializationContext {
    pub fn new(key_folding: bool, flatten_depth: Option<usize>) -> Self {
        Self {
            key_folding,
            flatten_depth: flatten_depth.unwrap_or(usize::MAX),
        }
    }
}

/// Serialize a Python object to TOON format string.
///
/// # Arguments
///
/// * `py` - Python interpreter handle
/// * `obj` - Python object to serialize (dict, list, or primitive)
/// * `delimiter` - Delimiter character for arrays/tables (',' | '\t' | '|')
/// * `indent_size` - Number of spaces per indentation level
/// * `key_folding` - Enable key folding (e.g., `a.b: value` for `a: {b: value}`)
/// * `flatten_depth` - Maximum depth for key folding (None for unlimited)
///
/// # Returns
///
/// TOON format string
pub fn serialize(
    py: Python,
    obj: &Bound<'_, PyAny>,
    delimiter: char,
    indent_size: usize,
    key_folding: bool,
    flatten_depth: Option<usize>,
) -> PyResult<String> {
    let mut output = String::new();
    let ctx = SerializationContext::new(key_folding, flatten_depth);

    // Detect root form
    if let Ok(dict) = obj.cast::<PyDict>() {
        serialize_object(
            py,
            &dict,
            &mut output,
            0,
            delimiter,
            true,
            indent_size,
            &ctx,
        )?;
    } else if let Ok(list) = obj.cast::<pyo3::types::PyList>() {
        serialize_array(
            py,
            &list,
            &mut output,
            0,
            delimiter,
            true,
            indent_size,
            &ctx,
        )?;
    } else {
        // Single primitive at root
        serialize_value(py, obj, &mut output, 0, delimiter, true, indent_size, &ctx)?;
    }

    Ok(output)
}

/// Serialize a value at a given depth with specified delimiter context
pub fn serialize_value(
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
    } else if let Ok(dt) = obj.cast::<PyDateTime>() {
        let iso_str: String = dt.call_method0("isoformat")?.extract()?;
        serialize_string(&iso_str, output, delimiter);
    } else if let Ok(date) = obj.cast::<PyDate>() {
        let iso_str: String = date.call_method0("isoformat")?.extract()?;
        serialize_string(&iso_str, output, delimiter);
    } else if let Ok(time) = obj.cast::<PyTime>() {
        let iso_str: String = time.call_method0("isoformat")?.extract()?;
        serialize_string(&iso_str, output, delimiter);
    } else {
        // Unknown type → null (per spec Section 3)
        output.push_str("null");
    }
    Ok(())
}

/// Serialize a string with proper quoting and escaping per TOON v3.0 Section 7
pub fn serialize_string(s: &str, output: &mut String, delimiter: char) {
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
            // Non-alphanumeric characters in Latin-1 Supplement range (U+0080–U+00FF)
            // need quoting to avoid encoding ambiguities (e.g., ®, ©, ¢)
            // but alphanumeric characters like é, ñ, ü don't need quoting
            _ if (ch as u32 >= 0x80 && ch as u32 <= 0xFF && !ch.is_alphanumeric()) => return true,
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
pub fn write_array_header(output: &mut String, len: usize, delimiter: char, inline: bool) {
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
pub fn write_tabular_header(output: &mut String, len: usize, delimiter: char, fields: &[String]) {
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
pub fn serialize_object(
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
pub fn serialize_key(key: &str, output: &mut String) {
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
pub fn is_valid_unquoted_key(key: &str) -> bool {
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
    _py: Python<'py>,
    start_key: &str,
    start_dict: &Bound<'py, PyDict>,
    _depth: usize,
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
            if let Ok(dict) = next_value.cast::<PyDict>() {
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
pub fn serialize_array(
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
pub fn write_indent(output: &mut String, depth: usize, indent_size: usize) {
    for _ in 0..depth * indent_size {
        output.push(' ');
    }
}

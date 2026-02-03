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
use pyo3::types::PyDict;

mod deserialize;
mod serialize;

use deserialize::Parser;
use serialize::SerializationContext;

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
        serialize::serialize_object(
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
        serialize::serialize_array(
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
        serialize::serialize_value(py, obj, &mut output, 0, delimiter, true, indent_size, &ctx)?;
    }

    Ok(output)
}

/// Deserialize a TOON format string to a Python object.
///
/// # Arguments
///
/// * `py` - Python interpreter handle
/// * `input` - TOON format string
/// * `strict` - Enable strict mode validation
/// * `expand_paths` - Path expansion mode ("off" | "safe" | "always")
/// * `indent` - Expected indentation size (None for auto-detect)
///
/// # Returns
///
/// Python object (dict, list, or primitive)
pub fn deserialize(
    py: Python,
    input: &str,
    strict: bool,
    expand_paths: &str,
    indent: Option<usize>,
) -> PyResult<Py<PyAny>> {
    let mut parser = Parser::new(input, strict, expand_paths, indent);
    parser.parse(py)
}

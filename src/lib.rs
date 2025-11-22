//! Python bindings for TOON (Token-Oriented Object Notation)
//!
//! TOON is a compact, human-readable serialization format optimized for
//! Large Language Model contexts. This library provides a native Rust
//! implementation with Python bindings for high-performance encoding
//! and decoding of TOON data.
//!
//! # Features
//!
//! - **Full TOON v2.0 Specification Compliance**: Implements all features
//!   from the official TOON specification dated 2025-11-10
//! - **Direct Python Integration**: No intermediate JSON representation
//! - **Configurable Indentation**: Support for custom indent sizes (≥2 spaces)
//! - **Smart Parser**: Automatic indentation detection when parsing
//! - **Tabular Arrays**: Optimized format for uniform arrays of objects
//! - **Multiple Delimiters**: Support for comma, tab, and pipe delimiters
//!
//! # Quick Start
//!
//! ```python
//! import toons
//!
//! # Serialize Python objects to TOON
//! data = {"name": "Alice", "age": 30, "tags": ["admin", "user"]}
//! toon_str = toons.dumps(data)
//! print(toon_str)
//! # Output:
//! # name: Alice
//! # age: 30
//! # tags[2]: admin,user
//!
//! # Deserialize TOON back to Python
//! data = toons.loads(toon_str)
//!
//! # File operations
//! with open('data.toon', 'w') as f:
//!     toons.dump(data, f)
//!
//! with open('data.toon', 'r') as f:
//!     data = toons.load(f)
//!
//! # Custom indentation
//! toon_str = toons.dumps(data, indent=4)
//! ```
//!
//! # TOON Format Overview
//!
//! TOON uses a simple, whitespace-significant syntax:
//!
//! - **Objects**: key-value pairs with colon separator
//! - **Arrays**: inline format for primitives, tabular for uniform objects
//! - **Primitives**: strings, numbers, booleans, null
//! - **Indentation**: 2 spaces per level (default, configurable)
//!
//! # Specification
//!
//! This implementation follows TOON Specification v2.0 (2025-11-10).
//! For complete specification details, see:
//! <https://github.com/johannschopplich/toon>

use pyo3::prelude::*;

mod toon;

/// Deserialize a TOON formatted string to a Python object.
///
/// Parse a string containing TOON (Token-Oriented Object Notation) data
/// and return the corresponding Python object.
///
/// Args:
///     s: A string containing TOON formatted data
///     strict: If True (default), enforce strict TOON v2.0 compliance.
///             If False, allow some leniency (e.g. blank lines in arrays).
///
/// Returns:
///     A Python object (dict, list, or primitive) decoded from the TOON string
///
/// Raises:
///     ValueError: If the TOON string is malformed or contains invalid syntax
///
/// Example:
///     >>> import toons
///     >>> data = toons.loads("name: Alice\nage: 30")
///     >>> print(data)
///     {'name': 'Alice', 'age': 30}
#[pyfunction]
#[pyo3(signature = (s, *, strict=true))]
fn loads(py: Python, s: String, strict: bool) -> PyResult<Py<PyAny>> {
    toon::deserialize(py, &s, strict)
}

/// Deserialize a TOON formatted file to a Python object.
///
/// Read TOON data from a file-like object and return the corresponding
/// Python object.
///
/// Args:
///     fp: A file-like object with a read() method returning a string
///     strict: If True (default), enforce strict TOON v2.0 compliance.
///             If False, allow some leniency (e.g. blank lines in arrays).
///
/// Returns:
///     A Python object (dict, list, or primitive) decoded from the file
///
/// Raises:
///     ValueError: If the TOON data is malformed or contains invalid syntax
///
/// Example:
///     >>> import toons
///     >>> with open('data.toon', 'r') as f:
///     ...     data = toons.load(f)
#[pyfunction]
#[pyo3(signature = (fp, *, strict=true))]
fn load(py: Python, fp: &Bound<'_, PyAny>, strict: bool) -> PyResult<Py<PyAny>> {
    let read_method = fp.getattr("read")?;
    let content = read_method.call0()?;
    let content_str: String = content.extract()?;
    toon::deserialize(py, &content_str, strict)
}

/// Serialize a Python object to a TOON formatted string.
///
/// Convert a Python object (dict, list, or primitive) to its TOON
/// representation.
///
/// Args:
///     obj: A Python object to serialize (dict, list, str, int, float, bool, None)
///     indent: Number of spaces per indentation level (default: 2, minimum: 2)
///
/// Returns:
///     A string containing the TOON representation of the object
///
/// Raises:
///     ValueError: If indent is less than 2
///
/// Example:
///     >>> import toons
///     >>> data = {"name": "Alice", "tags": ["admin", "user"]}
///     >>> toon_str = toons.dumps(data)
///     >>> print(toon_str)
///     name: Alice
///     tags[2]: admin,user
///
///     >>> # Custom indentation
///     >>> toon_str = toons.dumps(data, indent=4)
#[pyfunction]
#[pyo3(signature = (obj, *, indent=2))]
fn dumps(py: Python, obj: &Bound<'_, PyAny>, indent: usize) -> PyResult<String> {
    toon::serialize(py, obj, indent)
}

/// Serialize a Python object to a TOON formatted file.
///
/// Convert a Python object to TOON format and write it to a file-like object.
///
/// Args:
///     obj: A Python object to serialize (dict, list, str, int, float, bool, None)
///     fp: A file-like object with a write() method
///     indent: Number of spaces per indentation level (default: 2, minimum: 2)
///
/// Raises:
///     ValueError: If indent is less than 2
///
/// Example:
///     >>> import toons
///     >>> data = {"name": "Alice", "age": 30}
///     >>> with open('data.toon', 'w') as f:
///     ...     toons.dump(data, f)
///
///     >>> # Custom indentation
///     >>> with open('data.toon', 'w') as f:
///     ...     toons.dump(data, f, indent=4)
#[pyfunction]
#[pyo3(signature = (obj, fp, *, indent=2))]
fn dump(py: Python, obj: &Bound<'_, PyAny>, fp: &Bound<'_, PyAny>, indent: usize) -> PyResult<()> {
    let toon_str = toon::serialize(py, obj, indent)?;
    let write_method = fp.getattr("write")?;
    write_method.call1((toon_str,))?;
    Ok(())
}

#[pymodule]
#[pyo3(module = "toons")]
fn toons(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(loads, m)?)?;
    m.add_function(wrap_pyfunction!(dump, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add(
        "__doc__",
        "Python bindings for TOON (Token-Oriented Object Notation)

TOON is a compact, human-readable serialization format optimized for
Large Language Model contexts. This library provides a native Rust
implementation with Python bindings for high-performance encoding
and decoding of TOON data.

Features:
    - Full TOON v2.0 Specification Compliance
    - Direct Python Integration (no JSON overhead)
    - Configurable Indentation (≥2 spaces)
    - Smart Parser with automatic indentation detection
    - Tabular format for uniform arrays of objects
    - Multiple delimiter support (comma, tab, pipe)

Quick Start:
    >>> import toons
    >>> data = {\"name\": \"Alice\", \"age\": 30}
    >>> toon_str = toons.dumps(data)
    >>> print(toon_str)
    name: Alice
    age: 30
    >>> data = toons.loads(toon_str)

Specification:
    TOON Specification v2.0 (2025-11-10)
    https://github.com/johannschopplich/toon
",
    )?;
    Ok(())
}

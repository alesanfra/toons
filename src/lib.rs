mod deserialization;
mod serialization;

/// Python bindings for TOON (Token-Oriented Object Notation)
///
/// TOON is a compact, human-readable serialization format optimized for
/// Large Language Model contexts. This library provides a native Rust
/// implementation with Python bindings for high-performance encoding
/// and decoding of TOON data.
///
/// # Quick Start
///
/// ```python
/// import toons
///
/// # Serialize Python objects to TOON
/// data = {"name": "Alice", "age": 30, "tags": ["admin", "user"]}
/// toon_str = toons.dumps(data)
/// print(toon_str)
/// # Output:
/// # name: Alice
/// # age: 30
/// # tags[2]: admin,user
///
/// # Deserialize TOON back to Python
/// data = toons.loads(toon_str)
///
/// # File operations
/// with open('data.toon', 'w') as f:
///     toons.dump(data, f)
///
/// with open('data.toon', 'r') as f:
///     data = toons.load(f)
///
/// # Custom indentation
/// toon_str = toons.dumps(data, indent=4)
/// ```
#[pyo3::pymodule]
mod toons {
    use pyo3::prelude::*;

    #[allow(non_upper_case_globals)]
    #[pymodule_export]
    const __version__: &str = env!("CARGO_PKG_VERSION");

    /// Deserialize a TOON formatted string to a Python object.
    ///
    /// Parse a string containing TOON (Token-Oriented Object Notation) data
    /// and return the corresponding Python object.
    ///
    /// Args:
    ///     s: A string containing TOON formatted data
    ///     strict: If True (default), enforce strict TOON v3.0 compliance.
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
    #[pyo3(signature = (s, *, strict=true, expand_paths=None, indent=None))]
    fn loads(
        py: Python,
        s: String,
        strict: bool,
        expand_paths: Option<&str>,
        indent: Option<usize>,
    ) -> PyResult<Py<PyAny>> {
        let expand_mode = expand_paths.unwrap_or("off");
        crate::deserialization::deserialize(py, &s, strict, expand_mode, indent)
    }

    /// Deserialize a TOON formatted file to a Python object.
    ///
    /// Read TOON data from a file-like object and return the corresponding
    /// Python object.
    ///
    /// Args:
    ///     fp: A file-like object with a read() method returning a string
    ///     strict: If True (default), enforce strict TOON v3.0 compliance.
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
    #[pyo3(signature = (fp, *, strict=true, expand_paths=None, indent=None))]
    fn load(
        py: Python,
        fp: &Bound<'_, PyAny>,
        strict: bool,
        expand_paths: Option<&str>,
        indent: Option<usize>,
    ) -> PyResult<Py<PyAny>> {
        let expand_mode = expand_paths.unwrap_or("off");
        let read_method = fp.getattr("read")?;
        let content = read_method.call0()?;
        let content_str: String = content.extract()?;
        crate::deserialization::deserialize(py, &content_str, strict, expand_mode, indent)
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
    #[pyo3(signature = (obj, *, indent=2, delimiter=",", key_folding=None, flatten_depth=None))]
    fn dumps(
        py: Python,
        obj: &Bound<'_, PyAny>,
        indent: usize,
        delimiter: &str,
        key_folding: Option<&str>,
        flatten_depth: Option<usize>,
    ) -> PyResult<String> {
        if indent < 2 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "indent must be >= 2",
            ));
        }
        // key_folding: only enable when explicitly set to "safe", "on", or "always"
        let enable_key_folding = matches!(key_folding, Some("safe") | Some("on") | Some("always"));
        crate::serialization::serialize(
            py,
            obj,
            delimiter.chars().next().unwrap(),
            indent,
            enable_key_folding,
            flatten_depth,
        )
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
    #[pyo3(signature = (obj, fp, *, indent=2, delimiter=",", key_folding=None, flatten_depth=None))]
    fn dump(
        py: Python,
        obj: &Bound<'_, PyAny>,
        fp: &Bound<'_, PyAny>,
        indent: usize,
        delimiter: &str,
        key_folding: Option<&str>,
        flatten_depth: Option<usize>,
    ) -> PyResult<()> {
        if indent < 2 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "indent must be >= 2",
            ));
        }
        // key_folding: only enable when explicitly set to "safe", "on", or "always"
        let enable_key_folding = matches!(key_folding, Some("safe") | Some("on") | Some("always"));
        let toon_str = crate::serialization::serialize(
            py,
            obj,
            delimiter.chars().next().unwrap(),
            indent,
            enable_key_folding,
            flatten_depth,
        )?;
        let write_method = fp.getattr("write")?;
        write_method.call1((toon_str,))?;
        Ok(())
    }
}

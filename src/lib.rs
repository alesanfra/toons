mod deserialization;
mod serialization;

pyo3::create_exception!(
    toons,
    ToonDecodeError,
    pyo3::exceptions::PyValueError,
    "Raised when the TOON decoder cannot parse the input. Subclass of ValueError. Carries `.line` (1-based int or None) and `.source` (raw line string or None) attributes."
);

/// Python bindings for TOON (Token-Oriented Object Notation).
///
/// TOON is a compact, human-readable serialization format aimed at Large
/// Language Model contexts. This module exposes a `json`-like API backed by
/// a native Rust implementation.
///
/// # Quick start
///
/// ```python
/// import toons
///
/// toon_str = toons.dumps({"name": "Alice", "tags": ["admin", "user"]})
/// # name: Alice
/// # tags[2]: admin,user
///
/// data = toons.loads(toon_str)
///
/// with open("data.toon", "w") as f:
///     toons.dump(data, f)
///
/// with open("data.toon", "r") as f:
///     data = toons.load(f)
/// ```
#[pyo3::pymodule]
mod toons {
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    /// Version of this library.
    #[allow(non_upper_case_globals)]
    #[pymodule_export]
    const __version__: &str = env!("CARGO_PKG_VERSION");

    /// Version of the TOON specification this library implements, as
    /// recommended by spec Section 13. Independent of `__version__`.
    #[allow(non_upper_case_globals)]
    #[pymodule_export]
    const __toon_spec__: &str = "4.1";

    #[pymodule_export]
    use super::ToonDecodeError;

    /// Spaces per indentation level when the caller passes neither
    /// `indent_size` nor its `indent` alias (spec Section 13 default).
    const DEFAULT_INDENT_SIZE: usize = 2;

    /// Validate the `delimiter` argument and return it as a single char.
    fn parse_delimiter(delimiter: &str) -> PyResult<char> {
        match delimiter {
            "," => Ok(','),
            "\t" => Ok('\t'),
            "|" => Ok('|'),
            other => Err(PyValueError::new_err(format!(
                "delimiter must be ',', '\\t', or '|', got {:?}",
                other
            ))),
        }
    }

    /// Resolve `indent_size` and its deprecated `indent` alias into one
    /// value. Passing both is an error unless they agree.
    fn resolve_indent(
        indent_size: Option<usize>,
        indent: Option<usize>,
    ) -> PyResult<Option<usize>> {
        match (indent_size, indent) {
            (Some(size), Some(alias)) if size != alias => Err(PyValueError::new_err(
                "indent_size and its alias indent disagree; pass only one",
            )),
            (Some(size), _) => Ok(Some(size)),
            (None, alias) => Ok(alias),
        }
    }

    /// Validate an encoder indentation, which must leave room for nesting.
    fn check_encode_indent(indent_size: usize) -> PyResult<()> {
        if indent_size < 2 {
            return Err(PyValueError::new_err("indent_size must be >= 2"));
        }
        Ok(())
    }

    /// Validate a decoder indentation.
    fn check_decode_indent(indent_size: Option<usize>) -> PyResult<()> {
        if indent_size == Some(0) {
            return Err(PyValueError::new_err("indent_size must be >= 1"));
        }
        Ok(())
    }

    /// Deserialize a TOON formatted string to a Python object.
    ///
    /// Args:
    ///     s: String containing TOON data.
    ///     strict: If True (default), enforce strict TOON v4.1 compliance.
    ///         If False, allow the documented leniencies, such as blank
    ///         lines inside arrays and count mismatches.
    ///     indent_size: Expected spaces per indentation level (default 2).
    ///     indent: Deprecated alias of `indent_size`.
    ///
    /// Returns:
    ///     The decoded Python object (dict, list, or primitive).
    ///
    /// Raises:
    ///     ToonDecodeError: If the input is malformed. Subclass of
    ///         `ValueError`; carries `.line` (1-based) and `.source`
    ///         (raw line) attributes.
    ///     ValueError: If an option value is invalid.
    ///
    /// Example:
    ///     >>> import toons
    ///     >>> toons.loads("name: Alice\nage: 30")
    ///     {'name': 'Alice', 'age': 30}
    #[pyfunction]
    #[pyo3(signature = (s, *, strict=true, indent_size=None, indent=None))]
    fn loads(
        py: Python,
        s: String,
        strict: bool,
        indent_size: Option<usize>,
        indent: Option<usize>,
    ) -> PyResult<Py<PyAny>> {
        let indent_size = resolve_indent(indent_size, indent)?;
        check_decode_indent(indent_size)?;
        crate::deserialization::deserialize(py, &s, strict, indent_size)
    }

    /// Deserialize TOON data read from a file-like object.
    ///
    /// Args:
    ///     fp: File-like object with a read() method returning a string.
    ///     strict: If True (default), enforce strict TOON v4.1 compliance.
    ///     indent_size: Expected spaces per indentation level (default 2).
    ///     indent: Deprecated alias of `indent_size`.
    ///
    /// Returns:
    ///     The decoded Python object (dict, list, or primitive).
    ///
    /// Raises:
    ///     ToonDecodeError: If the input is malformed. See `loads`.
    ///     ValueError: If an option value is invalid.
    ///
    /// Example:
    ///     >>> import toons
    ///     >>> with open('data.toon', 'r') as f:
    ///     ...     data = toons.load(f)
    #[pyfunction]
    #[pyo3(signature = (fp, *, strict=true, indent_size=None, indent=None))]
    fn load(
        py: Python,
        fp: &Bound<'_, PyAny>,
        strict: bool,
        indent_size: Option<usize>,
        indent: Option<usize>,
    ) -> PyResult<Py<PyAny>> {
        let indent_size = resolve_indent(indent_size, indent)?;
        check_decode_indent(indent_size)?;
        let content: String = fp.call_method0("read")?.extract()?;
        crate::deserialization::deserialize(py, &content, strict, indent_size)
    }

    /// Convert a TOON formatted string to a JSON formatted string.
    ///
    /// The conversion goes through the standard library `json` module, so
    /// the result matches `json.dumps(toons.loads(s), indent=indent)`.
    ///
    /// Args:
    ///     s: String containing TOON data.
    ///     strict: If True (default), enforce strict TOON v4.1 compliance.
    ///     indent_size: Expected spaces per TOON indentation level
    ///         (default 2).
    ///     indent: Spaces per JSON indentation level, or None (default) for
    ///         compact JSON.
    ///
    /// Returns:
    ///     The JSON representation of the decoded data.
    ///
    /// Raises:
    ///     ToonDecodeError: If the input is malformed. See `loads`.
    ///     ValueError: If an option value is invalid.
    ///
    /// Example:
    ///     >>> import toons
    ///     >>> toons.to_json("name: Alice\nage: 30")
    ///     '{"name": "Alice", "age": 30}'
    #[pyfunction]
    #[pyo3(signature = (s, *, strict=true, indent_size=None, indent=None))]
    fn to_json(
        py: Python,
        s: String,
        strict: bool,
        indent_size: Option<usize>,
        indent: Option<usize>,
    ) -> PyResult<String> {
        check_decode_indent(indent_size)?;
        let parsed_obj = crate::deserialization::deserialize(py, &s, strict, indent_size)?;
        let json = py.import("json")?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("indent", indent)?;
        json.getattr("dumps")?
            .call((parsed_obj,), Some(&kwargs))?
            .extract()
    }

    /// Serialize a Python object to a TOON formatted string.
    ///
    /// Args:
    ///     obj: Object to serialize. dict, list, tuple, str, int, float,
    ///         bool, None, and date/time/datetime objects are supported.
    ///     indent_size: Spaces per indentation level (default 2, minimum 2).
    ///     delimiter: Document delimiter for arrays and tables: ","
    ///         (default), "\t", or "|".
    ///     indent: Deprecated alias of `indent_size`.
    ///
    /// Returns:
    ///     The TOON representation of the object.
    ///
    /// Raises:
    ///     TypeError: If the object contains a type that cannot be encoded,
    ///         or a non-string object key.
    ///     ValueError: If an option value is invalid, or the object contains
    ///         a reference cycle.
    ///
    /// Example:
    ///     >>> import toons
    ///     >>> print(toons.dumps({"name": "Alice", "tags": ["admin"]}))
    ///     name: Alice
    ///     tags[1]: admin
    #[pyfunction]
    #[pyo3(signature = (obj, *, indent_size=None, delimiter=",", indent=None))]
    fn dumps(
        _py: Python,
        obj: &Bound<'_, PyAny>,
        indent_size: Option<usize>,
        delimiter: &str,
        indent: Option<usize>,
    ) -> PyResult<String> {
        let indent_size = resolve_indent(indent_size, indent)?.unwrap_or(DEFAULT_INDENT_SIZE);
        check_encode_indent(indent_size)?;
        crate::serialization::serialize(obj, parse_delimiter(delimiter)?, indent_size)
    }

    /// Serialize a Python object as TOON and write it to a file-like object.
    ///
    /// Args:
    ///     obj: Object to serialize. See `dumps` for supported types.
    ///     fp: File-like object with a write() method.
    ///     indent_size: Spaces per indentation level (default 2, minimum 2).
    ///     delimiter: Document delimiter for arrays and tables: ","
    ///         (default), "\t", or "|".
    ///     indent: Deprecated alias of `indent_size`.
    ///
    /// Raises:
    ///     TypeError: If the object contains a type that cannot be encoded,
    ///         or a non-string object key.
    ///     ValueError: If an option value is invalid, or the object contains
    ///         a reference cycle.
    ///
    /// Example:
    ///     >>> import toons
    ///     >>> with open('data.toon', 'w') as f:
    ///     ...     toons.dump({"name": "Alice"}, f)
    #[pyfunction]
    #[pyo3(signature = (obj, fp, *, indent_size=None, delimiter=",", indent=None))]
    fn dump(
        _py: Python,
        obj: &Bound<'_, PyAny>,
        fp: &Bound<'_, PyAny>,
        indent_size: Option<usize>,
        delimiter: &str,
        indent: Option<usize>,
    ) -> PyResult<()> {
        let indent_size = resolve_indent(indent_size, indent)?.unwrap_or(DEFAULT_INDENT_SIZE);
        check_encode_indent(indent_size)?;
        let toon_str =
            crate::serialization::serialize(obj, parse_delimiter(delimiter)?, indent_size)?;
        fp.call_method1("write", (toon_str,))?;
        Ok(())
    }
}

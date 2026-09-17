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
    const __toon_spec__: &str = "3.0";

    #[pymodule_export]
    use super::ToonDecodeError;

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

    /// Validate the `key_folding` argument and return whether folding is on.
    ///
    /// `"on"` and `"always"` are accepted as aliases of `"safe"`.
    fn parse_key_folding(key_folding: Option<&str>) -> PyResult<bool> {
        match key_folding {
            None | Some("off") => Ok(false),
            Some("safe") | Some("on") | Some("always") => Ok(true),
            Some(other) => Err(PyValueError::new_err(format!(
                "key_folding must be 'off' or 'safe', got {:?}",
                other
            ))),
        }
    }

    /// Validate the `expand_paths` argument and return the decoder mode.
    fn parse_expand_paths(expand_paths: Option<&str>) -> PyResult<&str> {
        match expand_paths {
            None | Some("off") => Ok("off"),
            Some("safe") => Ok("safe"),
            Some("always") => Ok("always"),
            Some(other) => Err(PyValueError::new_err(format!(
                "expand_paths must be 'off', 'safe', or 'always', got {:?}",
                other
            ))),
        }
    }

    /// Validate an encoder `indent`, which must leave room for nesting.
    fn check_encode_indent(indent: usize) -> PyResult<()> {
        if indent < 2 {
            return Err(PyValueError::new_err("indent must be >= 2"));
        }
        Ok(())
    }

    /// Validate a decoder `indent` hint.
    fn check_decode_indent(indent: Option<usize>) -> PyResult<()> {
        if indent == Some(0) {
            return Err(PyValueError::new_err("indent must be >= 1"));
        }
        Ok(())
    }

    /// Deserialize a TOON formatted string to a Python object.
    ///
    /// Args:
    ///     s: String containing TOON data.
    ///     strict: If True (default), enforce strict TOON v3.0 compliance.
    ///         If False, allow leniency such as blank lines inside arrays.
    ///     expand_paths: Expand dotted keys into nested objects:
    ///         None (default), "off", "safe", or "always".
    ///     indent: Expected spaces per indentation level, or None to detect
    ///         it from the input.
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
    #[pyo3(signature = (s, *, strict=true, expand_paths=None, indent=None))]
    fn loads(
        py: Python,
        s: String,
        strict: bool,
        expand_paths: Option<&str>,
        indent: Option<usize>,
    ) -> PyResult<Py<PyAny>> {
        let expand_mode = parse_expand_paths(expand_paths)?;
        check_decode_indent(indent)?;
        crate::deserialization::deserialize(py, &s, strict, expand_mode, indent)
    }

    /// Deserialize TOON data read from a file-like object.
    ///
    /// Args:
    ///     fp: File-like object with a read() method returning a string.
    ///     strict: If True (default), enforce strict TOON v3.0 compliance.
    ///     expand_paths: Expand dotted keys into nested objects:
    ///         None (default), "off", "safe", or "always".
    ///     indent: Expected spaces per indentation level, or None to detect
    ///         it from the input.
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
    #[pyo3(signature = (fp, *, strict=true, expand_paths=None, indent=None))]
    fn load(
        py: Python,
        fp: &Bound<'_, PyAny>,
        strict: bool,
        expand_paths: Option<&str>,
        indent: Option<usize>,
    ) -> PyResult<Py<PyAny>> {
        let expand_mode = parse_expand_paths(expand_paths)?;
        check_decode_indent(indent)?;
        let content: String = fp.call_method0("read")?.extract()?;
        crate::deserialization::deserialize(py, &content, strict, expand_mode, indent)
    }

    /// Convert a TOON formatted string to a JSON formatted string.
    ///
    /// The conversion goes through the standard library `json` module, so
    /// the result matches `json.dumps(toons.loads(s), indent=indent)`.
    ///
    /// Args:
    ///     s: String containing TOON data.
    ///     strict: If True (default), enforce strict TOON v3.0 compliance.
    ///     expand_paths: Expand dotted keys into nested objects:
    ///         None (default), "off", "safe", or "always".
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
    #[pyo3(signature = (s, *, strict=true, expand_paths=None, indent=None))]
    fn to_json(
        py: Python,
        s: String,
        strict: bool,
        expand_paths: Option<&str>,
        indent: Option<usize>,
    ) -> PyResult<String> {
        let expand_mode = parse_expand_paths(expand_paths)?;
        let parsed_obj = crate::deserialization::deserialize(py, &s, strict, expand_mode, None)?;
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
    ///     indent: Spaces per indentation level (default 2, minimum 2).
    ///     delimiter: Array and tabular delimiter: "," (default), "\t",
    ///         or "|".
    ///     key_folding: Fold single-key object chains into dotted keys:
    ///         None (default) or "off" to disable, "safe" to enable.
    ///     flatten_depth: Maximum number of segments in a folded key.
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
    #[pyo3(signature = (obj, *, indent=2, delimiter=",", key_folding=None, flatten_depth=None))]
    fn dumps(
        _py: Python,
        obj: &Bound<'_, PyAny>,
        indent: usize,
        delimiter: &str,
        key_folding: Option<&str>,
        flatten_depth: Option<usize>,
    ) -> PyResult<String> {
        check_encode_indent(indent)?;
        crate::serialization::serialize(
            obj,
            parse_delimiter(delimiter)?,
            indent,
            parse_key_folding(key_folding)?,
            flatten_depth,
        )
    }

    /// Serialize a Python object as TOON and write it to a file-like object.
    ///
    /// Args:
    ///     obj: Object to serialize. See `dumps` for supported types.
    ///     fp: File-like object with a write() method.
    ///     indent: Spaces per indentation level (default 2, minimum 2).
    ///     delimiter: Array and tabular delimiter: "," (default), "\t",
    ///         or "|".
    ///     key_folding: Fold single-key object chains into dotted keys:
    ///         None (default) or "off" to disable, "safe" to enable.
    ///     flatten_depth: Maximum number of segments in a folded key.
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
    #[pyo3(signature = (obj, fp, *, indent=2, delimiter=",", key_folding=None, flatten_depth=None))]
    fn dump(
        _py: Python,
        obj: &Bound<'_, PyAny>,
        fp: &Bound<'_, PyAny>,
        indent: usize,
        delimiter: &str,
        key_folding: Option<&str>,
        flatten_depth: Option<usize>,
    ) -> PyResult<()> {
        check_encode_indent(indent)?;
        let toon_str = crate::serialization::serialize(
            obj,
            parse_delimiter(delimiter)?,
            indent,
            parse_key_folding(key_folding)?,
            flatten_depth,
        )?;
        fp.call_method1("write", (toon_str,))?;
        Ok(())
    }
}

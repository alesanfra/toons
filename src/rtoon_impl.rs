//! rtoon-based TOON implementation
//! Uses rtoon crate (v0.1.3, TOON Spec v1.2) with serde_json as intermediate

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use rtoon::{decode_default, encode_default};
use serde_json::Value as JsonValue;

/// Convert a JSON Value to a Python object
fn json_value_to_python(value: JsonValue, py: Python) -> PyResult<Py<PyAny>> {
    match value {
        JsonValue::Object(map) => {
            let dict = PyDict::new(py);
            for (key, val) in map {
                let py_key = PyString::new(py, &key);
                let py_val = json_value_to_python(val, py)?;
                dict.set_item(py_key, py_val)?;
            }
            Ok(dict.into())
        }
        JsonValue::Array(vec) => {
            let list = PyList::empty(py);
            for val in vec {
                let py_val = json_value_to_python(val, py)?;
                list.append(py_val)?;
            }
            Ok(list.into())
        }
        JsonValue::String(s) => Ok(PyString::new(py, &s).into()),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(PyInt::new(py, i).into())
            } else if let Some(f) = n.as_f64() {
                Ok(PyFloat::new(py, f).into())
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Invalid number",
                ))
            }
        }
        JsonValue::Bool(b) => Ok(PyBool::new(py, b).to_owned().into()),
        JsonValue::Null => Ok(py.None()),
    }
}

/// Convert a Python object to a serde_json Value
fn python_to_json_value(obj: &Bound<'_, PyAny>) -> PyResult<JsonValue> {
    if obj.is_none() {
        Ok(JsonValue::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(JsonValue::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(JsonValue::Number(serde_json::Number::from(i)))
    } else if let Ok(f) = obj.extract::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            Ok(JsonValue::Number(n))
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Invalid floating point number",
            ))
        }
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(JsonValue::String(s))
    } else if let Ok(list) = obj.cast::<PyList>() {
        let mut vec = Vec::new();
        for item in list.iter() {
            vec.push(python_to_json_value(&item)?);
        }
        Ok(JsonValue::Array(vec))
    } else if let Ok(dict) = obj.cast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            map.insert(key_str, python_to_json_value(&value)?);
        }
        Ok(JsonValue::Object(map))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
            "Cannot convert type {} to TOON",
            obj.get_type().name()?
        )))
    }
}

/// Parse TOON data from a string.
///
/// This function parses a TOON-formatted string and returns the corresponding
/// Python object (dict, list, str, int, float, bool, or None).
///
/// # Arguments
/// * `s` - A TOON-formatted string to parse
///
/// # Returns
/// A Python object representing the parsed TOON data
///
/// # Errors
/// Returns ValueError if the TOON string is malformed
///
/// # Example
/// ```python
/// import toons
///
/// data = toons.loads("""
/// name: John Doe
/// age: 30
/// tags[3]: admin,developer,ops
/// """)
/// print(data)  # {'name': 'John Doe', 'age': 30, 'tags': ['admin', 'developer', 'ops']}
/// ```
/// Parse TOON data from a string using rtoon decoder
pub fn loads_impl(py: Python, s: String) -> PyResult<Py<PyAny>> {
    match decode_default(&s) {
        Ok(value) => json_value_to_python(value, py),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "TOON parse error: {}",
            e
        ))),
    }
}

/// Parse TOON data from a file object.
///
/// This function reads TOON data from a file-like object and returns the
/// corresponding Python object.
///
/// # Arguments
/// * `fp` - A file-like object with a `read()` method
///
/// # Returns
/// A Python object representing the parsed TOON data
///
/// # Errors
/// Returns ValueError if the TOON content is malformed or IOError if reading fails
///
/// # Example
/// ```python
/// import toons
///
/// with open('data.toon', 'r') as f:
///     data = toons.load(f)
///     print(data)
/// ```
/// Parse TOON data from a file object using rtoon decoder
pub fn load_impl(py: Python, fp: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    // Call the read() method on the file object
    let read_method = fp.getattr("read")?;
    let content = read_method.call0()?;
    let content_str: String = content.extract()?;

    // Use loads_impl to parse the content
    loads_impl(py, content_str)
}

/// Serialize a Python object to a TOON-formatted string.
///
/// This function serializes a Python object (dict, list, str, int, float, bool,
/// or None) to a TOON-formatted string.
///
/// # Arguments
/// * `obj` - A Python object to serialize
///
/// # Returns
/// A TOON-formatted string
///
/// # Errors
/// Returns ValueError if the object cannot be serialized to TOON format
///
/// # Example
/// ```python
/// import toons
///
/// data = {
///     "name": "John Doe",
///     "age": 30,
///     "tags": ["admin", "developer", "ops"]
/// }
/// toon_str = toons.dumps(data)
/// print(toon_str)
/// # Output:
/// # name: John Doe
/// # age: 30
/// # tags[3]: admin,developer,ops
/// ```
/// Serialize a Python object to TOON using rtoon encoder
pub fn dumps_impl(_py: Python, obj: &Bound<'_, PyAny>, _indent: usize) -> PyResult<String> {
    // Note: rtoon does not support configurable indentation, it always uses 2 spaces
    // The indent parameter is ignored for compatibility with native implementation
    let json_value = python_to_json_value(obj)?;

    encode_default(&json_value).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("TOON serialization error: {}", e))
    })
}

/// Serialize a Python object to TOON format and write to a file object.
///
/// This function serializes a Python object to TOON format and writes it to
/// a file-like object.
///
/// # Arguments
/// * `obj` - A Python object to serialize
/// * `fp` - A file-like object with a `write()` method
///
/// # Errors
/// Returns ValueError if serialization fails or IOError if writing fails
///
/// # Example
/// ```python
/// import toons
///
/// data = {"name": "John", "age": 30}
/// with open('output.toon', 'w') as f:
///     toons.dump(data, f)
/// ```
/// Serialize a Python object to TOON and write to file using rtoon encoder
pub fn dump_impl(
    py: Python,
    obj: &Bound<'_, PyAny>,
    fp: &Bound<'_, PyAny>,
    indent: usize,
) -> PyResult<()> {
    // Serialize the object to TOON string
    let toon_str = dumps_impl(py, obj, indent)?;

    // Call the write() method on the file object
    let write_method = fp.getattr("write")?;
    write_method.call1((toon_str,))?;

    Ok(())
}

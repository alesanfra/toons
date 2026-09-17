use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateTime, PyDict, PyInt, PyList, PyTime, PyTuple};
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;

/// Maximum container nesting accepted by the encoder. Mirrors CPython's
/// default recursion limit and keeps deep inputs from overflowing the stack.
const MAX_DEPTH: usize = 1000;

/// Serialize a Python object to a TOON string.
///
/// # Arguments
///
/// * `obj` - Object to serialize (dict, list, tuple, or primitive)
/// * `delimiter` - Delimiter for arrays and tables (',' | '\t' | '|')
/// * `indent_size` - Spaces per indentation level
/// * `key_folding` - Fold single-key object chains into `a.b: value`
/// * `flatten_depth` - Maximum number of folded segments (None for unlimited)
pub fn serialize(
    obj: &Bound<'_, PyAny>,
    delimiter: char,
    indent_size: usize,
    key_folding: bool,
    flatten_depth: Option<usize>,
) -> PyResult<String> {
    let mut encoder = Encoder {
        out: String::new(),
        delimiter,
        indent_size,
        key_folding,
        flatten_depth: flatten_depth.unwrap_or(usize::MAX),
        open_containers: Vec::new(),
    };
    encoder.write_value(obj, 0, true)?;
    Ok(encoder.out)
}

/// Writes the TOON text of one document.
///
/// Every method appends to `out`. `depth` is the indentation level of the
/// line being written; containers place their children one level deeper.
struct Encoder {
    out: String,
    delimiter: char,
    indent_size: usize,
    key_folding: bool,
    flatten_depth: usize,
    /// Addresses of the containers on the current path, used to detect
    /// reference cycles and to bound nesting depth.
    open_containers: Vec<usize>,
}

impl Encoder {
    /// Mark a container as being written. Paired with [`Encoder::leave`].
    fn enter(&mut self, container: &Bound<'_, PyAny>) -> PyResult<()> {
        let address = container.as_ptr() as usize;

        if self.open_containers.contains(&address) {
            return Err(PyValueError::new_err("Circular reference detected"));
        }
        if self.open_containers.len() >= MAX_DEPTH {
            return Err(PyValueError::new_err(format!(
                "Maximum nesting depth of {} exceeded while serializing",
                MAX_DEPTH
            )));
        }

        self.open_containers.push(address);
        Ok(())
    }

    fn leave(&mut self) {
        self.open_containers.pop();
    }

    fn write_indent(&mut self, depth: usize) {
        for _ in 0..depth * self.indent_size {
            self.out.push(' ');
        }
    }

    /// Write any value. Containers emit their own line breaks and
    /// indentation; primitives are written at the current position.
    fn write_value(&mut self, obj: &Bound<'_, PyAny>, depth: usize, is_root: bool) -> PyResult<()> {
        if obj.is_none() {
            self.out.push_str("null");
        } else if let Ok(b) = obj.extract::<bool>() {
            self.out.push_str(if b { "true" } else { "false" });
        } else if let Ok(int_obj) = obj.cast::<PyInt>() {
            self.write_int(int_obj)?;
        } else if let Ok(f) = obj.extract::<f64>() {
            self.write_float(f);
        } else if let Ok(s) = obj.extract::<String>() {
            self.write_string(&s);
        } else if let Some(list) = as_sequence(obj)? {
            if !is_root {
                self.out.push('\n');
                self.write_indent(depth);
            }
            self.write_array(&list, depth)?;
        } else if let Ok(dict) = obj.cast::<PyDict>() {
            self.write_object(dict, depth, is_root)?;
        } else if obj.is_instance_of::<PyDateTime>()
            || obj.is_instance_of::<PyDate>()
            || obj.is_instance_of::<PyTime>()
        {
            let iso_str: String = obj.call_method0("isoformat")?.extract()?;
            self.write_string(&iso_str);
        } else {
            return Err(PyTypeError::new_err(format!(
                "Object of type {} is not TOON serializable",
                obj.get_type().name()?
            )));
        }

        Ok(())
    }

    /// Write an integer of any magnitude without losing precision.
    fn write_int(&mut self, int_obj: &Bound<'_, PyInt>) -> PyResult<()> {
        match int_obj.extract::<i64>() {
            Ok(i) => write!(self.out, "{}", i).unwrap(),
            // Outside i64: ask Python for the exact decimal representation.
            Err(_) => {
                let digits: String = int_obj.call_method0("__index__")?.str()?.extract()?;
                self.out.push_str(&digits);
            }
        }

        Ok(())
    }

    /// Write a float per Section 3: normalize -0 to 0, never use exponent
    /// notation, and encode non-finite values as null.
    fn write_float(&mut self, value: f64) {
        if value == 0.0 {
            self.out.push('0');
        } else if value.is_finite() {
            write!(self.out, "{}", value).unwrap();
        } else {
            self.out.push_str("null");
        }
    }

    /// Write a string, quoting and escaping it when required (Section 7).
    fn write_string(&mut self, s: &str) {
        if needs_quoting(s, self.delimiter) {
            self.out.push('"');
            push_escaped(s, &mut self.out);
            self.out.push('"');
        } else {
            self.out.push_str(s);
        }
    }

    /// Write an object key, quoting it when it is not a bare identifier
    /// (Section 7.3).
    fn write_key(&mut self, key: &str) {
        if is_valid_unquoted_key(key) {
            self.out.push_str(key);
        } else {
            self.out.push('"');
            push_escaped(key, &mut self.out);
            self.out.push('"');
        }
    }

    /// Write a dict (Section 8). The caller has already written any key and
    /// its colon; the root object starts at the current position.
    fn write_object(
        &mut self,
        dict: &Bound<'_, PyDict>,
        depth: usize,
        is_root: bool,
    ) -> PyResult<()> {
        self.enter(dict.as_any())?;
        let result = self.write_object_entries(dict, depth, is_root);
        self.leave();
        result
    }

    fn write_object_entries(
        &mut self,
        dict: &Bound<'_, PyDict>,
        depth: usize,
        is_root: bool,
    ) -> PyResult<()> {
        let entries = entries_of(dict)?;

        if entries.is_empty() {
            // An empty object adds nothing after the key and colon.
            return Ok(());
        }

        // Literal keys are needed to detect collisions with folded keys.
        let literal_keys: HashSet<&str> = entries.iter().map(|(key, _)| key.as_str()).collect();

        for (i, (key, value)) in entries.iter().enumerate() {
            if i > 0 || !is_root {
                self.out.push('\n');
                self.write_indent(depth);
            }

            if let Some(list) = as_sequence(value)? {
                self.write_key(key);
                self.write_array(&list, depth)?;
                continue;
            }

            // Key folding applies at the root only, where collisions with
            // literal sibling keys can be checked.
            if self.key_folding
                && depth == 0
                && let Ok(nested_dict) = value.cast::<PyDict>()
                && let Some((folded_key, folded_value)) =
                    fold_key_chain(key, nested_dict, self.flatten_depth, &literal_keys)?
            {
                self.write_folded_entry(&folded_key, &folded_value, depth)?;
                continue;
            }

            self.write_key(key);
            self.out.push(':');

            if let Ok(nested_dict) = value.cast::<PyDict>() {
                self.write_object(nested_dict, depth + 1, false)?;
            } else {
                self.out.push(' ');
                self.write_value(value, depth, false)?;
            }
        }

        Ok(())
    }

    /// Write an entry whose key chain was folded into `a.b.c`.
    fn write_folded_entry(
        &mut self,
        folded_key: &str,
        value: &Bound<'_, PyAny>,
        depth: usize,
    ) -> PyResult<()> {
        self.write_key(folded_key);

        if let Some(list) = as_sequence(value)? {
            // The array header supplies its own colon.
            self.write_array(&list, depth)
        } else if let Ok(dict) = value.cast::<PyDict>() {
            self.out.push(':');
            self.write_object(dict, depth + 1, false)
        } else {
            self.out.push_str(": ");
            self.write_value(value, depth, false)
        }
    }

    /// Write an array header and body at the current position (Section 9).
    /// `depth` is the indentation level of the header line; rows and items
    /// go one level deeper.
    fn write_array(&mut self, list: &Bound<'_, PyList>, depth: usize) -> PyResult<()> {
        self.enter(list.as_any())?;
        let result = self.write_array_body(list, depth);
        self.leave();
        result
    }

    fn write_array_body(&mut self, list: &Bound<'_, PyList>, depth: usize) -> PyResult<()> {
        let len = list.len();

        if list.iter().all(|item| is_primitive(&item)) {
            self.write_array_header(len, true);
            for (i, item) in list.iter().enumerate() {
                if i > 0 {
                    self.out.push(self.delimiter);
                }
                self.write_value(&item, depth, false)?;
            }
        } else if let Some(fields) = detect_tabular(list)? {
            self.write_tabular_header(len, &fields);
            self.write_tabular_rows(list, depth + 1, &fields)?;
        } else {
            self.write_array_header(len, false);
            self.write_expanded_items(list, depth + 1)?;
        }

        Ok(())
    }

    /// Write `[N]:` (Section 6). The delimiter appears only when non-default.
    fn write_array_header(&mut self, len: usize, inline: bool) {
        write!(self.out, "[{}", len).unwrap();
        if self.delimiter != ',' {
            self.out.push(self.delimiter);
        }
        self.out.push_str("]:");
        if inline && len > 0 {
            self.out.push(' ');
        }
    }

    /// Write `[N]{f1,f2}:` for tabular arrays (Section 9.3).
    fn write_tabular_header(&mut self, len: usize, fields: &[String]) {
        write!(self.out, "[{}", len).unwrap();
        if self.delimiter != ',' {
            self.out.push(self.delimiter);
        }
        self.out.push_str("]{");
        for (i, field) in fields.iter().enumerate() {
            if i > 0 {
                self.out.push(self.delimiter);
            }
            self.write_key(field);
        }
        self.out.push_str("}:");
    }

    /// Write one delimited row per object (Section 9.3).
    fn write_tabular_rows(
        &mut self,
        list: &Bound<'_, PyList>,
        depth: usize,
        fields: &[String],
    ) -> PyResult<()> {
        for item in list.iter() {
            self.out.push('\n');
            self.write_indent(depth);

            let dict = item.cast::<PyDict>()?;
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    self.out.push(self.delimiter);
                }
                let value = dict.get_item(field)?.unwrap();
                self.write_value(&value, depth, false)?;
            }
        }

        Ok(())
    }

    /// Write one `- ` item per element (Sections 9.2 and 9.4).
    fn write_expanded_items(&mut self, list: &Bound<'_, PyList>, depth: usize) -> PyResult<()> {
        for item in list.iter() {
            self.out.push('\n');
            self.write_indent(depth);

            if let Ok(dict) = item.cast::<PyDict>() {
                if dict.is_empty() {
                    // An empty object is a bare hyphen.
                    self.out.push('-');
                    continue;
                }
                self.out.push_str("- ");
                self.write_list_item_object(dict, depth)?;
            } else if let Some(inner_list) = as_sequence(&item)? {
                self.out.push_str("- ");
                self.write_array(&inner_list, depth)?;
            } else {
                self.out.push_str("- ");
                self.write_value(&item, depth, false)?;
            }
        }

        Ok(())
    }

    /// Write an object that is a list item: its first field shares the line
    /// with the `- ` marker and the rest align below it.
    fn write_list_item_object(&mut self, dict: &Bound<'_, PyDict>, depth: usize) -> PyResult<()> {
        self.enter(dict.as_any())?;
        let result = self.write_list_item_fields(dict, depth);
        self.leave();
        result
    }

    fn write_list_item_fields(&mut self, dict: &Bound<'_, PyDict>, depth: usize) -> PyResult<()> {
        for (i, (key, value)) in entries_of(dict)?.iter().enumerate() {
            if i > 0 {
                self.out.push('\n');
                // Fields align under the first one, past the "- " marker.
                self.write_indent(depth + 1);
            }

            if let Some(list) = as_sequence(value)? {
                self.write_key(key);
                self.write_array(&list, depth + 1)?;
                continue;
            }

            self.write_key(key);
            self.out.push(':');

            if let Ok(nested_dict) = value.cast::<PyDict>() {
                self.write_object(nested_dict, depth + 2, false)?;
            } else {
                self.out.push(' ');
                self.write_value(value, depth + 1, false)?;
            }
        }

        Ok(())
    }
}

/// Collect a dict as `(key, value)` pairs, rejecting non-string keys.
fn entries_of<'py>(dict: &Bound<'py, PyDict>) -> PyResult<Vec<(String, Bound<'py, PyAny>)>> {
    let mut entries = Vec::with_capacity(dict.len());

    for (key, value) in dict.iter() {
        match key.extract::<String>() {
            Ok(key) => entries.push((key, value)),
            Err(_) => {
                return Err(PyTypeError::new_err(format!(
                    "TOON object keys must be strings, got {}",
                    key.get_type().name()?
                )));
            }
        }
    }

    Ok(entries)
}

/// Return `obj` as a list when it is a list or a tuple, so both encode as
/// TOON arrays.
fn as_sequence<'py>(obj: &Bound<'py, PyAny>) -> PyResult<Option<Bound<'py, PyList>>> {
    if let Ok(list) = obj.cast::<PyList>() {
        Ok(Some(list.clone()))
    } else if let Ok(tuple) = obj.cast::<PyTuple>() {
        Ok(Some(PyList::new(obj.py(), tuple.iter())?))
    } else {
        Ok(None)
    }
}

fn push_escaped(s: &str, output: &mut String) {
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
}

/// Decide whether a string must be quoted (Section 7.2).
fn needs_quoting(s: &str, delimiter: char) -> bool {
    if s.is_empty() {
        return true;
    }

    if s.starts_with(|c: char| c.is_whitespace()) || s.ends_with(|c: char| c.is_whitespace()) {
        return true;
    }

    // Literals and numbers would decode back as a different type.
    if s == "true" || s == "false" || s == "null" || is_numeric_like(s) {
        return true;
    }

    for ch in s.chars() {
        match ch {
            ':' | '"' | '\\' | '[' | ']' | '{' | '}' | '\n' | '\r' | '\t' => return true,
            _ if ch == delimiter => return true,
            _ => {}
        }
    }

    // A leading hyphen would read as a list item marker.
    s.starts_with('-')
}

/// Report whether a string would decode as a number (Section 7.2).
fn is_numeric_like(s: &str) -> bool {
    // Leading zeros ("05") decode as strings but stay ambiguous enough to quote.
    if let Some(rest) = s.strip_prefix('0')
        && rest.starts_with(|c: char| c.is_ascii_digit())
    {
        return true;
    }

    s.parse::<f64>().is_ok()
}

/// Report whether a key matches `^[A-Za-z_][A-Za-z0-9_.]*$`.
fn is_valid_unquoted_key(key: &str) -> bool {
    let mut chars = key.chars();

    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }

    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
}

/// Report whether a value can appear inline in an array or tabular row.
fn is_primitive(obj: &Bound<'_, PyAny>) -> bool {
    !obj.is_instance_of::<PyDict>()
        && !obj.is_instance_of::<PyList>()
        && !obj.is_instance_of::<PyTuple>()
}

/// Fold a chain of single-key objects into a dotted key.
///
/// Returns the folded key and the value it points at, or `None` when the
/// chain cannot be folded: a multi-key object, a segment that needs quoting,
/// or a collision with a literal sibling key.
fn fold_key_chain<'py>(
    start_key: &str,
    start_dict: &Bound<'py, PyDict>,
    max_depth: usize,
    sibling_keys: &HashSet<&str>,
) -> PyResult<Option<(String, Bound<'py, PyAny>)>> {
    // Folding needs at least two segments.
    if max_depth < 2 || !is_valid_unquoted_key(start_key) {
        return Ok(None);
    }

    let mut key_chain = vec![start_key.to_string()];
    let mut current_dict = start_dict.clone();

    loop {
        if current_dict.len() != 1 {
            return Ok(None);
        }

        let (next_key, next_value) = entries_of(&current_dict)?.remove(0);

        if !is_valid_unquoted_key(&next_key) {
            return Ok(None);
        }

        key_chain.push(next_key);

        let nested_dict = next_value.cast::<PyDict>().ok();
        let stop_here = key_chain.len() >= max_depth
            || match &nested_dict {
                Some(dict) => dict.is_empty(),
                None => true,
            };

        if stop_here {
            let folded_key = key_chain.join(".");
            if sibling_keys.contains(folded_key.as_str()) {
                return Ok(None);
            }
            return Ok(Some((folded_key, next_value)));
        }

        current_dict = nested_dict.unwrap().clone();
    }
}

/// Return the shared field names when a list qualifies for tabular form:
/// every element is an object with the same keys and primitive values
/// (Section 9.3).
fn detect_tabular(list: &Bound<'_, PyList>) -> PyResult<Option<Vec<String>>> {
    let Some(first_item) = list.iter().next() else {
        return Ok(None);
    };

    let Ok(first_dict) = first_item.cast::<PyDict>() else {
        return Ok(None);
    };

    let mut fields = Vec::with_capacity(first_dict.len());
    for key in first_dict.keys().iter() {
        match key.extract::<String>() {
            Ok(field) => fields.push(field),
            Err(_) => return Ok(None),
        }
    }

    if fields.is_empty() {
        return Ok(None);
    }

    for item in list.iter() {
        let Ok(dict) = item.cast::<PyDict>() else {
            return Ok(None);
        };

        if dict.len() != fields.len() {
            return Ok(None);
        }

        for field in &fields {
            match dict.get_item(field)? {
                Some(value) if is_primitive(&value) => {}
                _ => return Ok(None),
            }
        }
    }

    Ok(Some(fields))
}

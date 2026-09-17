use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateTime, PyDict, PyInt, PyList, PyString, PyTime, PyTuple};
use std::fmt::Write as FmtWrite;

/// Maximum container nesting accepted by the encoder. Mirrors CPython's
/// default recursion limit and keeps deep inputs from overflowing the stack.
const MAX_DEPTH: usize = 1000;

/// Serialize a Python object to a TOON string.
///
/// # Arguments
///
/// * `obj` - Object to serialize (dict, list, tuple, or primitive)
/// * `delimiter` - Document delimiter for arrays and tables (',' | '\t' | '|')
/// * `indent_size` - Spaces per indentation level
pub fn serialize(obj: &Bound<'_, PyAny>, delimiter: char, indent_size: usize) -> PyResult<String> {
    let mut encoder = Encoder {
        out: String::new(),
        delimiter,
        indent_size,
        open_containers: Vec::new(),
    };
    encoder.write_root(obj)?;
    Ok(encoder.out)
}

/// One entry of a header's field list (Section 1.4). A group renders a
/// nested-uniform column as `name{sub1,sub2}` and consumes as many row
/// cells as it has leaf fields.
enum Field {
    Leaf(String),
    Group(String, Vec<Field>),
}

/// Writes the TOON text of one document.
///
/// Every method appends to `out`. `depth` is the indentation level of the
/// line being written; containers place their children one level deeper.
struct Encoder {
    out: String,
    delimiter: char,
    indent_size: usize,
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

    fn write_newline(&mut self, depth: usize) {
        self.out.push('\n');
        self.write_indent(depth);
    }

    /// Write the document root (Section 5).
    fn write_root(&mut self, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Some(list) = as_sequence(obj)? {
            self.enter(list.as_any())?;
            let result = self.write_root_array(&list);
            self.leave();
            return result;
        }

        if let Ok(dict) = obj.cast::<PyDict>() {
            self.enter(dict.as_any())?;
            let result = self.write_root_object(dict);
            self.leave();
            return result;
        }

        self.write_primitive(obj)
    }

    fn write_root_array(&mut self, list: &Bound<'_, PyList>) -> PyResult<()> {
        if list.is_empty() {
            // An empty root array is the literal token (Section 9.1).
            self.out.push_str("[]");
            return Ok(());
        }

        if all_primitive(list) {
            self.write_bracket(list.len(), false);
            self.out.push_str("]: ");
            return self.write_inline_values(list);
        }

        if let Some(fields) = detect_tabular(list)? {
            self.write_tabular_header(list.len(), &fields);
            return self.write_rows(list, &fields, 1);
        }

        self.write_bracket(list.len(), false);
        self.out.push_str("]:");
        self.write_list_items(list, 1)
    }

    fn write_root_object(&mut self, dict: &Bound<'_, PyDict>) -> PyResult<()> {
        if let Some(fields) = detect_keyed(dict)? {
            self.write_bracket(dict.len(), true);
            self.out.push(']');
            self.write_field_list(&fields);
            self.out.push(':');
            return self.write_entry_rows(dict, &fields, 1);
        }

        for (index, (key, value)) in entries_of(dict)?.iter().enumerate() {
            if index > 0 {
                self.write_newline(0);
            }
            self.write_field(key, value, 0)?;
        }

        Ok(())
    }

    /// Write one object field `key…` at the current position. `depth` is the
    /// indentation level the field stands at; any scope it opens is written
    /// one level deeper (Sections 8 and 9).
    fn write_field(&mut self, key: &str, value: &Bound<'_, PyAny>, depth: usize) -> PyResult<()> {
        if let Some(list) = as_sequence(value)? {
            if list.is_empty() {
                // Empty arrays use the canonical value form (Section 9.1).
                self.write_key(key);
                self.out.push_str(": []");
                return Ok(());
            }

            self.enter(list.as_any())?;
            let result = self.write_keyed_array(key, &list, depth);
            self.leave();
            return result;
        }

        if let Ok(nested) = value.cast::<PyDict>() {
            self.enter(nested.as_any())?;
            let result = self.write_keyed_object(key, nested, depth);
            self.leave();
            return result;
        }

        self.write_key(key);
        self.out.push_str(": ");
        self.write_primitive(value)
    }

    fn write_keyed_array(
        &mut self,
        key: &str,
        list: &Bound<'_, PyList>,
        depth: usize,
    ) -> PyResult<()> {
        if all_primitive(list) {
            self.write_key(key);
            self.write_bracket(list.len(), false);
            self.out.push_str("]: ");
            return self.write_inline_values(list);
        }

        if let Some(fields) = detect_tabular(list)? {
            self.write_key(key);
            self.write_tabular_header(list.len(), &fields);
            return self.write_rows(list, &fields, depth + 1);
        }

        self.write_key(key);
        self.write_bracket(list.len(), false);
        self.out.push_str("]:");
        self.write_list_items(list, depth + 1)
    }

    fn write_keyed_object(
        &mut self,
        key: &str,
        dict: &Bound<'_, PyDict>,
        depth: usize,
    ) -> PyResult<()> {
        if let Some(fields) = detect_keyed(dict)? {
            self.write_key(key);
            self.write_bracket(dict.len(), true);
            self.out.push(']');
            self.write_field_list(&fields);
            self.out.push(':');
            return self.write_entry_rows(dict, &fields, depth + 1);
        }

        self.write_key(key);
        self.out.push(':');

        for (key, value) in entries_of(dict)?.iter() {
            self.write_newline(depth + 1);
            self.write_field(key, value, depth + 1)?;
        }

        Ok(())
    }

    /// Write `[N<delim?>` or `[N:<delim?>`; the caller closes the bracket.
    fn write_bracket(&mut self, length: usize, keyed: bool) {
        write!(self.out, "[{}", length).unwrap();
        if keyed {
            self.out.push(':');
        }
        if self.delimiter != ',' {
            self.out.push(self.delimiter);
        }
    }

    fn write_tabular_header(&mut self, length: usize, fields: &[Field]) {
        self.write_bracket(length, false);
        self.out.push(']');
        self.write_field_list(fields);
        self.out.push(':');
    }

    /// Write `{f1<delim>f2{sub}}` (Section 6).
    fn write_field_list(&mut self, fields: &[Field]) {
        self.out.push('{');
        for (index, field) in fields.iter().enumerate() {
            if index > 0 {
                self.out.push(self.delimiter);
            }
            match field {
                Field::Leaf(name) => self.write_key(name),
                Field::Group(name, children) => {
                    self.write_key(name);
                    self.write_field_list(children);
                }
            }
        }
        self.out.push('}');
    }

    fn write_inline_values(&mut self, list: &Bound<'_, PyList>) -> PyResult<()> {
        for (index, item) in list.iter().enumerate() {
            if index > 0 {
                self.out.push(self.delimiter);
            }
            self.write_primitive(&item)?;
        }

        Ok(())
    }

    /// Write one row per element at `depth` (Section 9.3).
    fn write_rows(
        &mut self,
        list: &Bound<'_, PyList>,
        fields: &[Field],
        depth: usize,
    ) -> PyResult<()> {
        for item in list.iter() {
            self.write_newline(depth);
            let dict = item.cast::<PyDict>()?;
            let mut first_cell = true;
            self.write_cells(dict, fields, &mut first_cell)?;
        }

        Ok(())
    }

    /// Write one entry row per entry at `depth` (Section 9.5).
    fn write_entry_rows(
        &mut self,
        dict: &Bound<'_, PyDict>,
        fields: &[Field],
        depth: usize,
    ) -> PyResult<()> {
        for (key, value) in entries_of(dict)?.iter() {
            self.write_newline(depth);
            self.write_key(key);
            self.out.push_str(": ");
            let entry = value.cast::<PyDict>()?;
            let mut first_cell = true;
            self.write_cells(entry, fields, &mut first_cell)?;
        }

        Ok(())
    }

    /// Write an object's leaf values in depth-first field order.
    fn write_cells(
        &mut self,
        dict: &Bound<'_, PyDict>,
        fields: &[Field],
        first_cell: &mut bool,
    ) -> PyResult<()> {
        for field in fields {
            match field {
                Field::Leaf(name) => {
                    if !*first_cell {
                        self.out.push(self.delimiter);
                    }
                    *first_cell = false;
                    let value = dict.get_item(name)?.unwrap();
                    self.write_primitive(&value)?;
                }
                Field::Group(name, children) => {
                    let nested = dict.get_item(name)?.unwrap();
                    let nested = nested.cast::<PyDict>()?;
                    self.write_cells(nested, children, first_cell)?;
                }
            }
        }

        Ok(())
    }

    /// Write one list item per element at `depth` (Sections 9.2 and 9.4).
    fn write_list_items(&mut self, list: &Bound<'_, PyList>, depth: usize) -> PyResult<()> {
        for item in list.iter() {
            self.write_newline(depth);

            if let Ok(dict) = item.cast::<PyDict>() {
                if dict.is_empty() {
                    // An empty object list item is a bare hyphen (Section 10).
                    self.out.push('-');
                    continue;
                }
                self.enter(dict.as_any())?;
                let result = self.write_list_item_object(dict, depth);
                self.leave();
                result?;
                continue;
            }

            self.out.push_str("- ");

            if let Some(inner) = as_sequence(&item)? {
                self.enter(inner.as_any())?;
                let result = self.write_inner_array(&inner, depth);
                self.leave();
                result?;
                continue;
            }

            self.write_primitive(&item)?;
        }

        Ok(())
    }

    /// Write an array that is itself a list item. A keyless header carries no
    /// field list, so tabular form is unavailable here (Section 9.4).
    fn write_inner_array(&mut self, list: &Bound<'_, PyList>, depth: usize) -> PyResult<()> {
        self.write_bracket(list.len(), false);
        self.out.push_str("]:");

        if list.is_empty() {
            return Ok(());
        }

        if all_primitive(list) {
            self.out.push(' ');
            return self.write_inline_values(list);
        }

        self.write_list_items(list, depth + 1)
    }

    /// Write an object that is a list item: its first field shares the line
    /// with the `- ` marker and stands one level deeper (Section 10).
    fn write_list_item_object(&mut self, dict: &Bound<'_, PyDict>, depth: usize) -> PyResult<()> {
        for (index, (key, value)) in entries_of(dict)?.iter().enumerate() {
            if index == 0 {
                self.out.push_str("- ");
            } else {
                self.write_newline(depth + 1);
            }
            self.write_field(key, value, depth + 1)?;
        }

        Ok(())
    }

    /// Write a primitive at the current position (Sections 2, 3 and 7).
    fn write_primitive(&mut self, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        if obj.is_none() {
            self.out.push_str("null");
        } else if let Ok(flag) = obj.extract::<bool>() {
            self.out.push_str(if flag { "true" } else { "false" });
        } else if let Ok(int_obj) = obj.cast::<PyInt>() {
            self.write_int(int_obj)?;
        } else if let Ok(number) = obj.extract::<f64>() {
            self.write_float(number);
        } else if let Ok(text) = obj.extract::<String>() {
            self.write_string(&text);
        } else if obj.is_instance_of::<PyString>() {
            // The only str that does not extract is one holding an unpaired
            // surrogate, which has no TOON representation (Section 3).
            return Err(PyValueError::new_err(
                "String contains an unpaired surrogate and cannot be encoded",
            ));
        } else if obj.is_instance_of::<PyDateTime>()
            || obj.is_instance_of::<PyDate>()
            || obj.is_instance_of::<PyTime>()
        {
            let iso: String = obj.call_method0("isoformat")?.extract()?;
            self.write_string(&iso);
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
            Ok(value) => write!(self.out, "{}", value).unwrap(),
            // Outside i64: ask Python for the exact decimal representation.
            Err(_) => {
                let digits: String = int_obj.call_method0("__index__")?.str()?.extract()?;
                self.out.push_str(&digits);
            }
        }

        Ok(())
    }

    /// Write a float per Section 2: normalize -0 to 0, prefer decimal form,
    /// and encode non-finite values as null (Section 3).
    fn write_float(&mut self, value: f64) {
        if value == 0.0 {
            self.out.push('0');
        } else if value.is_finite() {
            write!(self.out, "{}", value).unwrap();
        } else {
            self.out.push_str("null");
        }
    }

    /// Write a string, quoting and escaping it when required (Section 7.2).
    fn write_string(&mut self, text: &str) {
        if needs_quoting(text, self.delimiter) {
            self.out.push('"');
            push_escaped(text, &mut self.out);
            self.out.push('"');
        } else {
            self.out.push_str(text);
        }
    }

    /// Write an object key, entry key, or field name (Section 7.3).
    fn write_key(&mut self, key: &str) {
        if is_valid_unquoted_key(key) {
            self.out.push_str(key);
        } else {
            self.out.push('"');
            push_escaped(key, &mut self.out);
            self.out.push('"');
        }
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

fn push_escaped(text: &str, out: &mut String) {
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Remaining C0 controls are escaped numerically (Section 7.1).
            c if (c as u32) < 0x20 => write!(out, "\\u{:04x}", c as u32).unwrap(),
            c => out.push(c),
        }
    }
}

/// Decide whether a string must be quoted (Section 7.2).
fn needs_quoting(text: &str, delimiter: char) -> bool {
    if text.is_empty() {
        return true;
    }

    // Only spaces and tabs count as edge whitespace (Section 7.2).
    if text.starts_with([' ', '\t']) || text.ends_with([' ', '\t']) {
        return true;
    }

    // Literals and numbers would decode back as a different type.
    if text == "true" || text == "false" || text == "null" || is_numeric_like(text) {
        return true;
    }

    for ch in text.chars() {
        match ch {
            ':' | '"' | '\\' | '[' | ']' | '{' | '}' => return true,
            c if (c as u32) < 0x20 => return true,
            c if c == delimiter => return true,
            _ => {}
        }
    }

    // A leading hyphen reads as a list-item marker, a leading number sign
    // as a comment line (Sections 5.1 and 9.4).
    text.starts_with(['-', '#'])
}

/// Report whether a string matches the numeric-like quoting trigger
/// `^[+-]?[0-9]+(\.[0-9]+)?(e[+-]?[0-9]+)?$` (Section 7.2).
fn is_numeric_like(text: &str) -> bool {
    let body = text.strip_prefix(['+', '-']).unwrap_or(text);
    let (int_part, rest) = split_digits(body);

    if int_part.is_empty() {
        return false;
    }

    let rest = match rest.strip_prefix('.') {
        Some(after_dot) => {
            let (frac, rest) = split_digits(after_dot);
            if frac.is_empty() {
                return false;
            }
            rest
        }
        None => rest,
    };

    let rest = match rest.strip_prefix(['e', 'E']) {
        Some(after_e) => {
            let after_sign = after_e.strip_prefix(['+', '-']).unwrap_or(after_e);
            let (exp, rest) = split_digits(after_sign);
            if exp.is_empty() {
                return false;
            }
            rest
        }
        None => rest,
    };

    rest.is_empty()
}

fn split_digits(text: &str) -> (&str, &str) {
    let end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    (&text[..end], &text[end..])
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

/// Report whether a value can appear as a row cell or inline array value.
fn is_primitive(obj: &Bound<'_, PyAny>) -> bool {
    !obj.is_instance_of::<PyDict>()
        && !obj.is_instance_of::<PyList>()
        && !obj.is_instance_of::<PyTuple>()
}

fn all_primitive(list: &Bound<'_, PyList>) -> bool {
    list.iter().all(|item| is_primitive(&item))
}

/// Collect the keys of a dict in encounter order, or `None` when any key is
/// not a string.
fn string_keys(dict: &Bound<'_, PyDict>) -> Option<Vec<String>> {
    dict.keys()
        .iter()
        .map(|key| key.extract::<String>().ok())
        .collect()
}

/// Return the field list when every value shares one uniform object shape:
/// the detection used by both tabular and keyed tabular form (Sections 9.3
/// and 9.5).
fn detect_uniform(values: &[Bound<'_, PyAny>], depth: usize) -> PyResult<Option<Vec<Field>>> {
    // Detection recurses per column level; beyond the encoder's nesting
    // bound the value cannot be written anyway, so stop claiming a form.
    if depth >= MAX_DEPTH {
        return Ok(None);
    }

    let Some(first) = values.first() else {
        return Ok(None);
    };

    let Ok(first_dict) = first.cast::<PyDict>() else {
        return Ok(None);
    };

    let Some(names) = string_keys(first_dict) else {
        return Ok(None);
    };

    // An empty object disqualifies the whole collection (Section 9.3).
    if names.is_empty() {
        return Ok(None);
    }

    let mut columns: Vec<Vec<Bound<'_, PyAny>>> = vec![Vec::new(); names.len()];

    for value in values {
        let Ok(dict) = value.cast::<PyDict>() else {
            return Ok(None);
        };

        if dict.len() != names.len() {
            return Ok(None);
        }

        for (index, name) in names.iter().enumerate() {
            match dict.get_item(name)? {
                Some(cell) => columns[index].push(cell),
                None => return Ok(None),
            }
        }
    }

    let mut fields = Vec::with_capacity(names.len());

    for (name, column) in names.into_iter().zip(columns) {
        if column.iter().all(is_primitive) {
            fields.push(Field::Leaf(name));
            continue;
        }

        // A nested-uniform column becomes a nested field group.
        match detect_uniform(&column, depth + 1)? {
            Some(children) => fields.push(Field::Group(name, children)),
            None => return Ok(None),
        }
    }

    Ok(Some(fields))
}

/// Return the field list when an array qualifies for tabular form
/// (Section 9.3).
fn detect_tabular(list: &Bound<'_, PyList>) -> PyResult<Option<Vec<Field>>> {
    let elements: Vec<Bound<'_, PyAny>> = list.iter().collect();
    detect_uniform(&elements, 0)
}

/// Return the field list when an object qualifies for keyed tabular form
/// (Section 9.5).
fn detect_keyed(dict: &Bound<'_, PyDict>) -> PyResult<Option<Vec<Field>>> {
    if dict.len() < 2 || string_keys(dict).is_none() {
        return Ok(None);
    }

    let values: Vec<Bound<'_, PyAny>> = dict.values().iter().collect();
    detect_uniform(&values, 0)
}

//! TOON deserialization module
//!
//! Implements decoding of TOON format to Python objects according to
//! TOON Specification v3.0 (2025-11-24).

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};

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
pub fn check_key_conflict(
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
pub fn split_dotted_key(key: &str) -> Option<Vec<&str>> {
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
pub fn deep_merge_path(
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

pub struct Parser<'a> {
    lines: Vec<&'a str>,
    pos: usize,
    indent_size: usize,
    explicit_indent: Option<usize>,
    strict: bool,
    expand_paths: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(
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

    pub fn parse(&mut self, py: Python) -> PyResult<Py<PyAny>> {
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

    pub fn parse_object(&mut self, py: Python, depth: usize) -> PyResult<Py<PyAny>> {
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
                let first_quote_pos = key_part.find('"');
                let first_bracket_pos = key_part.find('[');

                let has_array_syntax = match (first_quote_pos, first_bracket_pos) {
                    (None, Some(_)) => key_part.contains(']'),
                    (Some(q), Some(b)) if b < q => key_part.contains(']'),
                    (Some(_q), Some(_b)) => {
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
                            self.parse_key(key_name)?
                        } else {
                            key_name.to_string()
                        };
                        check_key_conflict(&dict, &key, value.bind(py), self.strict)?;
                        dict.set_item(key, value)?;
                    }
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
                // Missing colon error
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "TOON parse error: Missing colon in line: {}",
                    line_trimmed
                )));
            }
        }

        Ok(dict.into())
    }

    pub fn parse_field_array(
        &mut self,
        py: Python,
        header_line: &str,
        depth: usize,
    ) -> PyResult<Py<PyAny>> {
        let (length, delimiter, fields) = self.parse_header(header_line)?;
        self.pos += 1;

        if let Some(field_names) = fields {
            self.parse_tabular_array(py, length, delimiter, &field_names, depth + 1)
        } else {
            let header_trimmed = header_line.trim();
            if let Some(bracket_end) = header_trimmed.find("]:") {
                let after_colon = header_trimmed[bracket_end + 2..].trim();
                if !after_colon.is_empty() {
                    self.parse_inline_array(py, after_colon, delimiter, length)
                } else {
                    self.parse_expanded_array(py, length, depth + 1)
                }
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "TOON parse error: Invalid array header",
                ))
            }
        }
    }

    pub fn parse_header(&self, header: &str) -> PyResult<(usize, char, Option<Vec<String>>)> {
        let trimmed = header.trim();

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

    pub fn parse_tabular_array(
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
                let mut lookahead = self.pos + 1;
                while lookahead < self.lines.len() && self.lines[lookahead].trim().is_empty() {
                    lookahead += 1;
                }

                if lookahead < self.lines.len() {
                    let next_depth = self.get_depth(self.lines[lookahead]);
                    if next_depth < expected_depth {
                        break;
                    }
                }

                if self.strict {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "TOON parse error: Blank line inside array",
                    ));
                }
                self.pos += 1;
                continue;
            }

            if !self.is_tabular_row(line_trimmed, delimiter) {
                break;
            }

            let values = self.split_by_delimiter(line_trimmed, delimiter);

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

        let actual_len = list.len();
        if length > 0 && actual_len != length {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "TOON parse error: Array declared length {} but found {} elements",
                length, actual_len
            )));
        }

        Ok(list.into())
    }

    pub fn parse_inline_array(
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

    pub fn parse_expanded_array(
        &mut self,
        py: Python,
        length: usize,
        expected_depth: usize,
    ) -> PyResult<Py<PyAny>> {
        let list = PyList::empty(py);

        while self.pos < self.lines.len() {
            let line = self.lines[self.pos];
            let line_trimmed = line.trim();

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
                let mut lookahead = self.pos + 1;
                while lookahead < self.lines.len() && self.lines[lookahead].trim().is_empty() {
                    lookahead += 1;
                }

                if lookahead < self.lines.len() {
                    let next_depth = self.get_depth(self.lines[lookahead]);
                    if next_depth < expected_depth {
                        break;
                    }
                }

                if self.strict {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "TOON parse error: Blank line inside array",
                    ));
                }
                self.pos += 1;
                continue;
            }

            if !line_trimmed.starts_with('-') {
                break;
            }

            let item_str = if line_trimmed.len() > 1 && line_trimmed.chars().nth(1) == Some(' ') {
                &line_trimmed[2..]
            } else if line_trimmed.len() == 1 {
                ""
            } else {
                &line_trimmed[1..]
            };
            self.pos += 1;

            if item_str.is_empty() {
                let empty_obj = PyDict::new(py);
                list.append(empty_obj)?;
                continue;
            }

            if item_str.starts_with('[') && item_str.contains("]:") {
                let header_part = item_str.split("]:").next().unwrap();
                let header_with_bracket = format!("{}]", header_part);
                let (inner_len, inner_delim, _) = self.parse_header(&header_with_bracket)?;

                let after_colon = item_str.split("]:").nth(1).unwrap_or("").trim();

                if after_colon.is_empty() {
                    let value = self.parse_expanded_array(py, inner_len, expected_depth + 1)?;
                    list.append(value)?;
                } else {
                    let value = self.parse_inline_array(py, after_colon, inner_delim, inner_len)?;
                    list.append(value)?;
                }
            } else if item_str.contains(':') {
                self.pos -= 1;
                let value = self.parse_list_item_object(py, expected_depth)?;
                list.append(value)?;
            } else {
                let value = self.parse_primitive(py, item_str)?;
                list.append(value)?;
            }
        }

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

        if let Some(item_content) = line_trimmed.strip_prefix("- ") {
            if let Some(colon_pos) = item_content.find(':') {
                let key_part = &item_content[..colon_pos];
                let value_part = item_content[colon_pos + 1..].trim();

                let quote_end_pos = key_part.rfind('"');

                let has_array_syntax = if let Some(quote_end) = quote_end_pos {
                    key_part[quote_end + 1..].contains('[')
                        && key_part[quote_end + 1..].contains(']')
                } else {
                    key_part.contains('[') && key_part.contains(']')
                };

                if has_array_syntax {
                    let value = self.parse_field_array(py, item_content, list_depth + 1)?;

                    let key_name = if let Some(quote_end) = quote_end_pos {
                        &key_part[..quote_end + 1]
                    } else {
                        key_part.split('[').next().unwrap()
                    };
                    let key = self.parse_key(key_name)?;
                    dict.set_item(key, value)?;
                } else {
                    let key = self.parse_key(key_part)?;
                    self.pos += 1;

                    if value_part.is_empty() {
                        if self.pos < self.lines.len() {
                            let next_depth = self.get_depth(self.lines[self.pos]);
                            if next_depth > list_depth + 1 {
                                let value = self.parse_object(py, list_depth + 2)?;
                                dict.set_item(key, value)?;
                            }
                        }
                    } else {
                        let value = self.parse_primitive(py, value_part)?;
                        dict.set_item(key, value)?;
                    }
                }
            }
        }

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

                let quote_end_pos = key_part.rfind('"');

                let has_array_syntax = if let Some(quote_end) = quote_end_pos {
                    key_part[quote_end + 1..].contains('[')
                        && key_part[quote_end + 1..].contains(']')
                } else {
                    key_part.contains('[') && key_part.contains(']')
                };

                if has_array_syntax {
                    let value = self.parse_field_array(py, line_trimmed, list_depth + 1)?;

                    let key_name = if let Some(quote_end) = quote_end_pos {
                        &key_part[..quote_end + 1]
                    } else {
                        key_part.split('[').next().unwrap()
                    };
                    let key = self.parse_key(key_name)?;
                    dict.set_item(key, value)?;
                    continue;
                }

                let key = self.parse_key(key_part)?;
                self.pos += 1;

                if value_part.is_empty() {
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

        if trimmed.starts_with('"') {
            if !trimmed.ends_with('"') || trimmed.len() < 2 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "TOON parse error: Unterminated string",
                ));
            }
            let unescaped = self.unescape_string(&trimmed[1..trimmed.len() - 1])?;
            return Ok(PyString::new(py, &unescaped).into());
        }

        match trimmed {
            "null" => Ok(py.None()),
            "true" => Ok(PyBool::new(py, true).to_owned().into()),
            "false" => Ok(PyBool::new(py, false).to_owned().into()),
            _ => {
                let check_s = if trimmed.starts_with('-') {
                    &trimmed[1..]
                } else {
                    trimmed
                };

                if check_s.len() > 1
                    && check_s.starts_with('0')
                    && check_s.chars().nth(1).unwrap().is_ascii_digit()
                {
                    return Ok(PyString::new(py, trimmed).into());
                }

                if let Ok(i) = trimmed.parse::<i64>() {
                    Ok(PyInt::new(py, i).into())
                } else if let Ok(f) = trimmed.parse::<f64>() {
                    Ok(PyFloat::new(py, f).into())
                } else {
                    Ok(PyString::new(py, trimmed).into())
                }
            }
        }
    }

    fn should_expand_key(&self, key: &str) -> (bool, bool) {
        let trimmed = key.trim();
        let was_quoted = trimmed.starts_with('"') && trimmed.ends_with('"');

        match self.expand_paths {
            "off" | "never" => (false, was_quoted),
            "safe" => (!was_quoted, was_quoted),
            "always" => (true, was_quoted),
            _ => (false, was_quoted),
        }
    }

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
            (None, None) => true,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (Some(d), Some(c)) => d < c,
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

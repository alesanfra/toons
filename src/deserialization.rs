use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};

/// Spaces per indentation level when the caller does not pass one
/// (Section 13 default).
const DEFAULT_INDENT_SIZE: usize = 2;

/// Maximum container nesting accepted by the decoder, the documented depth
/// limit Section 15 allows. The parser recurses per level, so without it a
/// deep document overflows the stack and crashes the interpreter.
const MAX_NESTING: usize = 1000;

/// Build a `ToonDecodeError` with `.line` and `.source` attributes set
/// (either may be `None` when the offending location is unknown).
fn make_decode_error(
    py: Python,
    message: String,
    line: Option<usize>,
    source: Option<&str>,
) -> PyErr {
    let err = PyErr::new::<crate::ToonDecodeError, _>(message);
    let value = err.value(py);
    let _ = value.setattr(pyo3::intern!(py, "line"), line);
    let _ = value.setattr(pyo3::intern!(py, "source"), source);
    err
}

/// Deserialize a TOON document into a Python object.
///
/// # Arguments
///
/// * `py` - Python interpreter handle
/// * `input` - TOON text
/// * `strict` - Enable the Section 14 checks
/// * `indent_size` - Spaces per indentation level (None for the default 2)
pub fn deserialize(
    py: Python,
    input: &str,
    strict: bool,
    indent_size: Option<usize>,
) -> PyResult<Py<PyAny>> {
    let mut parser = Parser::new(input, strict, indent_size);
    parser.parse(py)
}

/// One field entry of a header's field list (Section 1.4). A group carries
/// its own nested field list and materializes a nested object per row.
enum Field {
    Leaf(String),
    Group(String, Vec<Field>),
}

impl Field {
    fn name(&self) -> &str {
        match self {
            Field::Leaf(name) => name,
            Field::Group(name, _) => name,
        }
    }

    /// Number of cells this entry consumes from a row.
    fn leaf_count(&self) -> usize {
        match self {
            Field::Leaf(_) => 1,
            Field::Group(_, children) => children.iter().map(Field::leaf_count).sum(),
        }
    }
}

fn leaf_count(fields: &[Field]) -> usize {
    fields.iter().map(Field::leaf_count).sum()
}

/// A parsed array or keyed header (Section 6).
struct Header {
    key: Option<String>,
    length: usize,
    keyed: bool,
    delimiter: char,
    fields: Option<Vec<Field>>,
    /// Content after the header's colon, trimmed of spaces.
    rest: String,
}

/// A source line after comment removal, with its indentation measured.
struct Line<'a> {
    raw: &'a str,
    /// 1-based number in the original document.
    lineno: usize,
    /// Line content without indentation or trailing spaces.
    content: &'a str,
    spaces: usize,
    tabs: usize,
}

impl Line<'_> {
    fn is_blank(&self) -> bool {
        self.content.is_empty()
    }
}

/// Scan `s` for the first occurrence of `target` outside a quoted token.
fn find_unquoted(s: &str, target: char) -> Option<usize> {
    let mut in_quotes = false;
    let mut escaped = false;

    for (i, ch) in s.char_indices() {
        if in_quotes {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quotes = false;
            }
            continue;
        }

        if ch == '"' {
            in_quotes = true;
        } else if ch == target {
            return Some(i);
        }
    }

    None
}

/// Index of the closing quote of the quoted token starting at byte 0.
fn quoted_token_end(s: &str) -> Option<usize> {
    let mut chars = s.char_indices();
    if chars.next().map(|(_, ch)| ch) != Some('"') {
        return None;
    }

    let mut escaped = false;
    for (i, ch) in chars {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(i);
        }
    }

    None
}

/// Trim spaces (U+0020) only: any other whitespace is part of the token
/// (Section 12).
fn trim_spaces(s: &str) -> &str {
    s.trim_matches(' ')
}

/// Report whether a token decodes as a number under the Section 4 grammar.
fn is_number_token(s: &str) -> bool {
    let body = s.strip_prefix('-').unwrap_or(s);
    let (int_part, rest) = split_digits(body);

    if int_part.is_empty() || (int_part.len() > 1 && int_part.starts_with('0')) {
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

fn split_digits(s: &str) -> (&str, &str) {
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    (&s[..end], &s[end..])
}

/// Report whether a bracket length token is a canonical non-negative integer.
fn is_length_token(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) && (s.len() == 1 || !s.starts_with('0'))
}

pub struct Parser<'a> {
    lines: Vec<Line<'a>>,
    pos: usize,
    indent_size: usize,
    strict: bool,
    /// Number of containers currently open, bounded by `MAX_NESTING`.
    nesting: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str, strict: bool, indent_size: Option<usize>) -> Self {
        // A leading byte-order mark is not content (Section 12).
        let input = input.strip_prefix('\u{feff}').unwrap_or(input);
        let mut lines = Vec::new();

        for (index, raw_line) in input.split('\n').enumerate() {
            // A trailing CR belongs to the line terminator, trailing spaces
            // are not content (Section 12).
            let raw = raw_line.strip_suffix('\r').unwrap_or(raw_line);
            let body = raw.trim_end_matches(' ');

            let indent_len = body
                .find(|c: char| c != ' ' && c != '\t')
                .unwrap_or(body.len());
            let indent = &body[..indent_len];
            let content = &body[indent_len..];

            // Comment lines are removed before every other rule
            // (Section 5.1); only spaces may precede the '#'.
            if content.starts_with('#') && !indent.contains('\t') {
                continue;
            }

            lines.push(Line {
                raw,
                lineno: index + 1,
                content,
                spaces: indent.matches(' ').count(),
                tabs: indent.matches('\t').count(),
            });
        }

        Parser {
            lines,
            pos: 0,
            indent_size: indent_size.unwrap_or(DEFAULT_INDENT_SIZE),
            strict,
            nesting: 0,
        }
    }

    /// Build a `ToonDecodeError` with line context from `self.pos`.
    fn err_here(&self, py: Python, msg: impl Into<String>) -> PyErr {
        self.err_at(py, self.pos, msg)
    }

    /// Build a `ToonDecodeError` from a line index, populating `.line`
    /// (1-based) and `.source` (the raw line). Both are `None` for an
    /// empty input.
    fn err_at(&self, py: Python, line_idx: usize, msg: impl Into<String>) -> PyErr {
        let (line_num, source) = if self.lines.is_empty() {
            (None, None)
        } else {
            let line = &self.lines[line_idx.min(self.lines.len() - 1)];
            (Some(line.lineno), Some(line.raw))
        };
        let formatted = match line_num {
            Some(n) => format!("TOON parse error at line {}: {}", n, msg.into()),
            None => format!("TOON parse error: {}", msg.into()),
        };
        make_decode_error(py, formatted, line_num, source)
    }

    /// Open a container, keeping the parser off the stack limit.
    /// Paired with [`Parser::leave`].
    fn enter(&mut self, py: Python) -> PyResult<()> {
        self.nesting += 1;
        if self.nesting > MAX_NESTING {
            return Err(self.err_here(
                py,
                format!("Maximum nesting depth of {} exceeded", MAX_NESTING),
            ));
        }
        Ok(())
    }

    fn leave(&mut self) {
        self.nesting -= 1;
    }

    fn depth_of(&self, line: &Line) -> usize {
        line.spaces / self.indent_size + line.tabs
    }

    fn depth_at(&self, idx: usize) -> usize {
        self.depth_of(&self.lines[idx])
    }

    /// Enforce the Section 12 indentation invariants over every content line.
    fn validate_indentation(&self, py: Python) -> PyResult<()> {
        if !self.strict {
            return Ok(());
        }

        for (idx, line) in self.lines.iter().enumerate() {
            if line.is_blank() {
                continue;
            }
            if line.tabs > 0 {
                return Err(self.err_at(py, idx, "Tabs are not allowed in indentation"));
            }
            if line.spaces % self.indent_size != 0 {
                return Err(self.err_at(
                    py,
                    idx,
                    format!(
                        "Indentation {} is not a multiple of indent size {}",
                        line.spaces, self.indent_size
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Index of the next line that is not blank, or `self.lines.len()`.
    fn next_content_line(&self, from: usize) -> usize {
        let mut idx = from;
        while idx < self.lines.len() && self.lines[idx].is_blank() {
            idx += 1;
        }
        idx
    }

    /// Handle a blank line met while collecting a scope's content.
    ///
    /// Returns `true` when the scope continues past the blank run. A blank
    /// inside a header span is a strict-mode error (Section 12); in
    /// non-strict mode it is skipped and never counted. Blanks that only
    /// trail the scope are left for the enclosing scope to consume.
    fn blank_inside_scope(
        &mut self,
        py: Python,
        content_depth: usize,
        seen_content: bool,
    ) -> PyResult<bool> {
        let next = self.next_content_line(self.pos);
        let continues = next < self.lines.len() && self.depth_at(next) >= content_depth;

        if !continues {
            return Ok(false);
        }

        if self.strict && seen_content {
            return Err(self.err_here(py, "Blank line inside array"));
        }

        self.pos = next;
        Ok(true)
    }

    pub fn parse(&mut self, py: Python) -> PyResult<Py<PyAny>> {
        self.validate_indentation(py)?;

        self.pos = self.next_content_line(0);
        if self.pos >= self.lines.len() {
            // An empty document decodes to an empty object (Section 5).
            return Ok(PyDict::new(py).into());
        }

        let content = self.lines[self.pos].content;
        let root_form = content.starts_with('[') && self.depth_at(self.pos) == 0;

        if root_form && content == "[]" {
            self.pos += 1;
            return self.finish_root(py, PyList::empty(py).into());
        }

        // A keyless header at depth 0 opens a root array or a keyed tabular
        // root object (Section 5).
        if root_form
            && let Ok(header) = self.try_parse_header(content)
            && header.key.is_none()
        {
            let value = self.parse_header_value(py, &header, 0)?;
            return self.finish_root(py, value);
        }

        // A lone line that is neither a header nor a key-value line is a
        // root primitive (Section 5).
        if self.next_content_line(self.pos + 1) >= self.lines.len()
            && find_unquoted(content, ':').is_none()
            && !content.starts_with('[')
        {
            return self.parse_primitive(py, content);
        }

        self.parse_object(py, 0, false)
    }

    /// Reject content after a completed root array or keyed root object
    /// (Section 5); non-strict mode ignores it.
    fn finish_root(&mut self, py: Python, value: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let next = self.next_content_line(self.pos);
        if next < self.lines.len() && self.strict {
            return Err(self.err_at(py, next, "Trailing content after the root value"));
        }
        Ok(value)
    }

    /// Parse an object scope whose fields sit at `depth`.
    ///
    /// `in_span` marks a scope that lies inside a header span, where blank
    /// lines are errors in strict mode (Section 12).
    fn parse_object(&mut self, py: Python, depth: usize, in_span: bool) -> PyResult<Py<PyAny>> {
        self.enter(py)?;
        let result = self.parse_object_body(py, depth, in_span);
        self.leave();
        result
    }

    fn parse_object_body(
        &mut self,
        py: Python,
        depth: usize,
        in_span: bool,
    ) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        let mut seen_field = false;

        while self.pos < self.lines.len() {
            if self.lines[self.pos].is_blank() {
                if in_span {
                    if self.blank_inside_scope(py, depth, seen_field)? {
                        continue;
                    }
                    break;
                }
                self.pos += 1;
                continue;
            }

            let line_depth = self.depth_at(self.pos);
            if line_depth < depth {
                break;
            }

            if line_depth > depth {
                // A line deeper than the scope's content depth belongs to no
                // scope (Section 8); a bare token there is an error in every
                // mode (Section 5.2).
                let content = self.lines[self.pos].content;
                if find_unquoted(content, ':').is_none() {
                    return Err(self.err_here(py, format!("Unexpected line: {}", content)));
                }
                if self.strict {
                    return Err(self.err_here(py, "Line is indented deeper than its scope"));
                }
                self.pos += 1;
                continue;
            }

            let content = self.lines[self.pos].content;
            self.parse_object_field(py, &dict, content, depth, in_span)?;
            seen_field = true;
        }

        Ok(dict.into())
    }

    /// Parse the field on the current line into `dict`, together with any
    /// scope it opens.
    fn parse_object_field(
        &mut self,
        py: Python,
        dict: &Bound<'_, PyDict>,
        content: &str,
        depth: usize,
        in_span: bool,
    ) -> PyResult<()> {
        let line_idx = self.pos;

        let colon = find_unquoted(content, ':');
        let bracket = find_unquoted(content, '[');

        // A line whose first unquoted colon precedes its first unquoted '['
        // is a key-value line, never a header (Section 5.2).
        let header_shaped = match (bracket, colon) {
            (Some(b), Some(c)) => b < c,
            (Some(_), None) => true,
            _ => false,
        };

        if header_shaped {
            match self.try_parse_header(content) {
                Ok(header) => {
                    if header.key.is_none() {
                        if self.strict {
                            return Err(self.err_at(
                                py,
                                line_idx,
                                "Keyless array header is valid only at the document root",
                            ));
                        }
                    } else {
                        let key = header.key.clone().unwrap();
                        let value = self.parse_header_value(py, &header, depth)?;
                        return self.insert(py, dict, &key, value, line_idx);
                    }
                }
                Err(msg) => {
                    if self.strict {
                        return Err(self.err_at(py, line_idx, msg));
                    }
                }
            }
        }

        // Key-value line, including the non-strict fall-through for a
        // malformed header (Section 6).
        let Some(colon) = colon else {
            return Err(self.err_at(py, line_idx, format!("Unexpected line: {}", content)));
        };

        let key = self.parse_key(py, trim_spaces(&content[..colon]), line_idx)?;
        let value_token = trim_spaces(&content[colon + 1..]);
        self.pos += 1;

        let value = if value_token.is_empty() {
            self.parse_nested_object(py, depth, in_span)?
        } else if value_token == "[]" {
            PyList::empty(py).into()
        } else {
            self.parse_primitive_at(py, value_token, line_idx)?
        };

        self.insert(py, dict, &key, value, line_idx)
    }

    /// Parse the object opened by a bare `key:` line at `depth`.
    fn parse_nested_object(
        &mut self,
        py: Python,
        depth: usize,
        in_span: bool,
    ) -> PyResult<Py<PyAny>> {
        let next = self.next_content_line(self.pos);
        if next >= self.lines.len() {
            return Ok(PyDict::new(py).into());
        }

        let next_depth = self.depth_at(next);
        if next_depth <= depth {
            return Ok(PyDict::new(py).into());
        }

        if next_depth > depth + 1 {
            if self.strict {
                return Err(self.err_at(py, next, "Indentation jumps more than one level"));
            }
            // Non-strict mode takes the line's own depth as the scope's.
            return self.parse_object(py, next_depth, in_span);
        }

        self.parse_object(py, depth + 1, in_span)
    }

    /// Store `value` under `key`, rejecting a duplicate sibling key in
    /// strict mode and applying last-write-wins otherwise (Section 14.3).
    fn insert(
        &self,
        py: Python,
        dict: &Bound<'_, PyDict>,
        key: &str,
        value: Py<PyAny>,
        line_idx: usize,
    ) -> PyResult<()> {
        if self.strict && dict.contains(key)? {
            return Err(self.err_at(py, line_idx, format!("Duplicate key '{}'", key)));
        }
        dict.set_item(key, value)
    }

    /// Decode the value declared by `header`, whose line sits at `depth`.
    fn parse_header_value(
        &mut self,
        py: Python,
        header: &Header,
        depth: usize,
    ) -> PyResult<Py<PyAny>> {
        let header_idx = self.pos;
        self.pos += 1;

        if let Some(fields) = &header.fields {
            if header.keyed {
                return self.parse_keyed_entries(py, header, fields, depth + 1, header_idx);
            }
            return self.parse_tabular_rows(py, header, fields, depth + 1, header_idx);
        }

        if !header.rest.is_empty() {
            return self.parse_inline_array(py, &header.rest, header, header_idx);
        }

        self.parse_list_items(py, header, depth + 1, header_idx)
    }

    /// Decode `key[N]: v1,v2` (Section 9.1).
    fn parse_inline_array(
        &self,
        py: Python,
        values: &str,
        header: &Header,
        header_idx: usize,
    ) -> PyResult<Py<PyAny>> {
        let tokens = split_cells(values, header.delimiter);

        if self.strict && tokens.len() != header.length {
            return Err(self.err_at(
                py,
                header_idx,
                format!(
                    "Array declared length {} but found {} values",
                    header.length,
                    tokens.len()
                ),
            ));
        }

        let list = PyList::empty(py);
        for token in tokens {
            list.append(self.parse_primitive_at(py, token, header_idx)?)?;
        }

        Ok(list.into())
    }

    /// Decode the rows of a tabular array (Section 9.3).
    fn parse_tabular_rows(
        &mut self,
        py: Python,
        header: &Header,
        fields: &[Field],
        row_depth: usize,
        header_idx: usize,
    ) -> PyResult<Py<PyAny>> {
        let list = PyList::empty(py);
        let width = leaf_count(fields);

        while self.pos < self.lines.len() {
            if self.lines[self.pos].is_blank() {
                if self.blank_inside_scope(py, row_depth, !list.is_empty())? {
                    continue;
                }
                break;
            }

            if self.depth_at(self.pos) != row_depth {
                break;
            }

            let line_idx = self.pos;
            let content = self.lines[line_idx].content;
            if !is_row_line(content, header.delimiter) {
                break;
            }

            let cells = split_cells(content, header.delimiter);
            if self.strict && cells.len() != width {
                return Err(self.err_at(
                    py,
                    line_idx,
                    format!(
                        "Row has {} cells but the header declares {} leaf fields",
                        cells.len(),
                        width
                    ),
                ));
            }

            let mut next_cell = 0;
            let row = self.build_row(py, fields, &cells, &mut next_cell, line_idx)?;
            list.append(row)?;
            self.pos += 1;
        }

        if self.strict && list.len() != header.length {
            return Err(self.err_at(
                py,
                header_idx,
                format!(
                    "Array declared length {} but found {} rows",
                    header.length,
                    list.len()
                ),
            ));
        }

        Ok(list.into())
    }

    /// Decode the entry rows of a keyed tabular object (Section 9.5).
    fn parse_keyed_entries(
        &mut self,
        py: Python,
        header: &Header,
        fields: &[Field],
        entry_depth: usize,
        header_idx: usize,
    ) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        let width = leaf_count(fields);
        let mut entries = 0;

        while self.pos < self.lines.len() {
            if self.lines[self.pos].is_blank() {
                if self.blank_inside_scope(py, entry_depth, entries > 0)? {
                    continue;
                }
                break;
            }

            if self.depth_at(self.pos) != entry_depth {
                break;
            }

            let line_idx = self.pos;
            let content = self.lines[line_idx].content;

            // Every line at entry depth with an unquoted colon is an entry
            // row; one without is an error in strict mode.
            let Some(colon) = find_unquoted(content, ':') else {
                if self.strict {
                    return Err(self.err_at(
                        py,
                        line_idx,
                        "Line at entry depth has no unquoted colon",
                    ));
                }
                self.pos += 1;
                continue;
            };

            let entry_key = self.parse_key(py, trim_spaces(&content[..colon]), line_idx)?;
            let cells = split_cells(trim_spaces(&content[colon + 1..]), header.delimiter);

            if self.strict && cells.len() != width {
                return Err(self.err_at(
                    py,
                    line_idx,
                    format!(
                        "Entry row has {} cells but the header declares {} leaf fields",
                        cells.len(),
                        width
                    ),
                ));
            }

            let mut next_cell = 0;
            let value = self.build_row(py, fields, &cells, &mut next_cell, line_idx)?;
            self.insert(py, &dict, &entry_key, value, line_idx)?;
            entries += 1;
            self.pos += 1;
        }

        if self.strict && entries != header.length {
            return Err(self.err_at(
                py,
                header_idx,
                format!(
                    "Keyed header declared {} entries but found {}",
                    header.length, entries
                ),
            ));
        }

        Ok(dict.into())
    }

    /// Materialize one row or entry row by walking the field list
    /// depth-first (Section 9.3).
    fn build_row(
        &self,
        py: Python,
        fields: &[Field],
        cells: &[&str],
        next_cell: &mut usize,
        line_idx: usize,
    ) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);

        for field in fields {
            match field {
                Field::Leaf(name) => {
                    // On a width mismatch in non-strict mode a leaf without
                    // a cell is absent from the decoded object (Section 14.1).
                    if let Some(cell) = cells.get(*next_cell) {
                        let value = self.parse_primitive_at(py, cell, line_idx)?;
                        dict.set_item(name, value)?;
                    }
                    *next_cell += 1;
                }
                Field::Group(name, children) => {
                    let nested = self.build_row(py, children, cells, next_cell, line_idx)?;
                    dict.set_item(name, nested)?;
                }
            }
        }

        Ok(dict.into())
    }

    /// Decode the items of an array in list form (Sections 9.2 and 9.4).
    fn parse_list_items(
        &mut self,
        py: Python,
        header: &Header,
        item_depth: usize,
        header_idx: usize,
    ) -> PyResult<Py<PyAny>> {
        self.enter(py)?;
        let result = self.parse_list_items_body(py, header, item_depth, header_idx);
        self.leave();
        result
    }

    fn parse_list_items_body(
        &mut self,
        py: Python,
        header: &Header,
        item_depth: usize,
        header_idx: usize,
    ) -> PyResult<Py<PyAny>> {
        let list = PyList::empty(py);

        while self.pos < self.lines.len() {
            if self.lines[self.pos].is_blank() {
                if self.blank_inside_scope(py, item_depth, !list.is_empty())? {
                    continue;
                }
                break;
            }

            if self.depth_at(self.pos) != item_depth {
                break;
            }

            let line_idx = self.pos;
            let content = self.lines[line_idx].content;
            let item = if content == "-" {
                Some("")
            } else {
                content.strip_prefix("- ")
            };
            let Some(item) = item else {
                break;
            };
            let item = trim_spaces(item);

            let value = self.parse_list_item(py, item, item_depth, line_idx)?;
            list.append(value)?;
        }

        if self.strict && list.len() != header.length {
            return Err(self.err_at(
                py,
                header_idx,
                format!(
                    "Array declared length {} but found {} items",
                    header.length,
                    list.len()
                ),
            ));
        }

        Ok(list.into())
    }

    /// Decode one list item whose hyphen line sits at `item_depth`.
    fn parse_list_item(
        &mut self,
        py: Python,
        item: &str,
        item_depth: usize,
        line_idx: usize,
    ) -> PyResult<Py<PyAny>> {
        if item.is_empty() {
            self.pos += 1;
            return Ok(PyDict::new(py).into());
        }

        if item == "[]" {
            self.pos += 1;
            return Ok(PyList::empty(py).into());
        }

        let colon = find_unquoted(item, ':');
        let bracket = find_unquoted(item, '[');
        let header_shaped = match (bracket, colon) {
            (Some(b), Some(c)) => b < c,
            (Some(_), None) => true,
            _ => false,
        };

        if header_shaped {
            match self.try_parse_header(item) {
                Ok(header) => {
                    if header.key.is_none() {
                        if header.fields.is_some() {
                            if self.strict {
                                return Err(self.err_at(
                                    py,
                                    line_idx,
                                    "A keyless header carrying a field list is valid only at \
                                     the document root",
                                ));
                            }
                        } else {
                            // The item is the inner array itself, so its own
                            // items sit one level deeper (Section 10).
                            return self.parse_inner_array(py, &header, item_depth, line_idx);
                        }
                    } else {
                        // A keyed first field stands one level deeper than
                        // the hyphen line (Section 10).
                        return self.parse_list_item_object(py, item, item_depth);
                    }
                }
                Err(msg) => {
                    if self.strict {
                        return Err(self.err_at(py, line_idx, msg));
                    }
                }
            }
        }

        if colon.is_some() {
            return self.parse_list_item_object(py, item, item_depth);
        }

        self.pos += 1;
        self.parse_primitive_at(py, item, line_idx)
    }

    /// Decode a `- [M]: …` list item (Sections 9.2 and 9.4).
    fn parse_inner_array(
        &mut self,
        py: Python,
        header: &Header,
        item_depth: usize,
        line_idx: usize,
    ) -> PyResult<Py<PyAny>> {
        self.pos += 1;

        if !header.rest.is_empty() {
            return self.parse_inline_array(py, &header.rest, header, line_idx);
        }

        self.parse_list_items(py, header, item_depth + 1, line_idx)
    }

    /// Decode an object list item: its first field is carried on the hyphen
    /// line and stands one level deeper (Section 10).
    fn parse_list_item_object(
        &mut self,
        py: Python,
        item: &str,
        item_depth: usize,
    ) -> PyResult<Py<PyAny>> {
        self.enter(py)?;
        let result = self.parse_list_item_object_body(py, item, item_depth);
        self.leave();
        result
    }

    fn parse_list_item_object_body(
        &mut self,
        py: Python,
        item: &str,
        item_depth: usize,
    ) -> PyResult<Py<PyAny>> {
        let field_depth = item_depth + 1;
        let dict = PyDict::new(py);

        // The field carried on the hyphen line is parsed as a field line at
        // `field_depth`; the parser still points at the hyphen line, so the
        // field parser consumes it.
        self.parse_object_field(py, &dict, item, field_depth, true)?;

        while self.pos < self.lines.len() {
            if self.lines[self.pos].is_blank() {
                if self.blank_inside_scope(py, field_depth, true)? {
                    continue;
                }
                break;
            }

            let line_depth = self.depth_at(self.pos);
            if line_depth < field_depth {
                break;
            }

            if line_depth > field_depth {
                let content = self.lines[self.pos].content;
                if find_unquoted(content, ':').is_none() {
                    return Err(self.err_here(py, format!("Unexpected line: {}", content)));
                }
                if self.strict {
                    return Err(self.err_here(py, "Line is indented deeper than its scope"));
                }
                self.pos += 1;
                continue;
            }

            let content = self.lines[self.pos].content;
            if content == "-" || content.starts_with("- ") {
                break;
            }

            self.parse_object_field(py, &dict, content, field_depth, true)?;
        }

        Ok(dict.into())
    }

    /// Parse a header line per the Section 6 grammar. `Err` carries the
    /// diagnostic for a malformed header, which strict mode reports and
    /// non-strict mode replaces with key-value parsing.
    fn try_parse_header(&self, content: &str) -> Result<Header, String> {
        let bracket = find_unquoted(content, '[').ok_or("Invalid array header: missing '['")?;

        let key_token = &content[..bracket];
        if key_token != trim_spaces(key_token) {
            return Err("Whitespace is not allowed between a key and its bracket segment".into());
        }

        let key = if key_token.is_empty() {
            None
        } else if key_token.starts_with('"') {
            match quoted_token_end(key_token) {
                Some(end) if end + 1 == key_token.len() => {
                    Some(unescape(&key_token[1..end]).map_err(|e| e.to_string())?)
                }
                Some(_) => return Err("Characters after a quoted key's closing quote".into()),
                None => return Err("Unterminated quoted key".into()),
            }
        } else {
            Some(key_token.to_string())
        };

        let close = content[bracket..]
            .find(']')
            .map(|offset| offset + bracket)
            .ok_or("Invalid array header: missing ']'")?;

        let (length, keyed, delimiter) = parse_bracket_segment(&content[bracket + 1..close])?;

        let after_bracket = &content[close + 1..];
        let (fields, after_fields) = if after_bracket.starts_with('{') {
            let end = field_list_end(after_bracket)
                .ok_or("Invalid field list: unmatched brace in header")?;
            let entries = parse_field_list(&after_bracket[1..end], delimiter, self.strict, 1)?;
            (Some(entries), &after_bracket[end + 1..])
        } else {
            (None, after_bracket)
        };

        if keyed && fields.is_none() {
            return Err("A keyed header must carry a field list".into());
        }

        let rest = after_fields
            .strip_prefix(':')
            .ok_or("Invalid array header: content between the bracket segment and the colon")?;

        let rest = trim_spaces(rest);
        if fields.is_some() && !rest.is_empty() {
            return Err("A header carrying a field list takes no inline content".into());
        }

        Ok(Header {
            key,
            length,
            keyed,
            delimiter,
            fields,
            rest: rest.to_string(),
        })
    }

    /// Decode a key token: quoted keys are unescaped, unquoted ones are
    /// literal (Section 7.4).
    fn parse_key(&self, py: Python, token: &str, line_idx: usize) -> PyResult<String> {
        if !token.starts_with('"') {
            return Ok(token.to_string());
        }

        match quoted_token_end(token) {
            Some(end) if end + 1 == token.len() => {
                unescape(&token[1..end]).map_err(|msg| self.err_at(py, line_idx, msg))
            }
            Some(_) => Err(self.err_at(
                py,
                line_idx,
                "Characters after a quoted key's closing quote",
            )),
            None => Err(self.err_at(py, line_idx, "Unterminated quoted key")),
        }
    }

    fn parse_primitive(&self, py: Python, token: &str) -> PyResult<Py<PyAny>> {
        self.parse_primitive_at(py, token, self.pos)
    }

    /// Decode a value token (Section 4).
    fn parse_primitive_at(&self, py: Python, token: &str, line_idx: usize) -> PyResult<Py<PyAny>> {
        if token.starts_with('"') {
            return match quoted_token_end(token) {
                Some(end) if end + 1 == token.len() => {
                    let text =
                        unescape(&token[1..end]).map_err(|msg| self.err_at(py, line_idx, msg))?;
                    Ok(PyString::new(py, &text).into())
                }
                Some(_) => Err(self.err_at(
                    py,
                    line_idx,
                    "Characters after a quoted token's closing quote",
                )),
                None => Err(self.err_at(py, line_idx, "Unterminated string")),
            };
        }

        match token {
            "null" => return Ok(py.None()),
            "true" => return Ok(PyBool::new(py, true).to_owned().into()),
            "false" => return Ok(PyBool::new(py, false).to_owned().into()),
            _ => {}
        }

        if is_number_token(token) {
            let integral = !token.contains(['.', 'e', 'E']);
            if integral {
                return match token.parse::<i64>() {
                    Ok(value) => Ok(PyInt::new(py, value).into()),
                    // Outside i64: keep every digit as a Python int.
                    Err(_) => Ok(py.get_type::<PyInt>().call1((token,))?.unbind()),
                };
            }
            if let Ok(value) = token.parse::<f64>() {
                // A magnitude beyond f64 is outside the documented numeric
                // domain: strict mode rejects it rather than return an
                // infinity that would re-encode as null (Section 4).
                if !value.is_finite() && self.strict {
                    return Err(self.err_at(
                        py,
                        line_idx,
                        format!("Number {} is out of range", token),
                    ));
                }
                // -0 decodes as zero (Section 4).
                let value = if value == 0.0 { 0.0 } else { value };
                return Ok(PyFloat::new(py, value).into());
            }
        }

        Ok(PyString::new(py, token).into())
    }
}

/// Parse `N`, `N<delim>`, `N:` or `N:<delim>` from inside a bracket
/// segment (Section 6).
fn parse_bracket_segment(segment: &str) -> Result<(usize, bool, char), String> {
    let (digits, rest) = split_digits(segment);

    if !is_length_token(digits) {
        return Err(format!("Invalid array length: '{}'", segment));
    }

    let (keyed, rest) = match rest.strip_prefix(':') {
        Some(rest) => (true, rest),
        None => (false, rest),
    };

    let delimiter = match rest {
        "" => ',',
        "\t" => '\t',
        "|" => '|',
        _ => return Err(format!("Malformed bracket segment: '{}'", segment)),
    };

    let length = digits
        .parse::<usize>()
        .map_err(|_| format!("Invalid array length: '{}'", digits))?;

    Ok((length, keyed, delimiter))
}

/// Index of the `}` closing the field list that starts at byte 0, ignoring
/// braces inside quoted names (Section 6).
fn field_list_end(s: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_quotes = false;
    let mut escaped = false;

    for (i, ch) in s.char_indices() {
        if in_quotes {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quotes = false;
            }
            continue;
        }

        match ch {
            '"' => in_quotes = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

/// Parse the entries of a field list, recursing into nested field groups
/// (Section 6).
fn parse_field_list(
    body: &str,
    delimiter: char,
    strict: bool,
    nesting: usize,
) -> Result<Vec<Field>, String> {
    if nesting > MAX_NESTING {
        return Err(format!(
            "Maximum nesting depth of {} exceeded in a field list",
            MAX_NESTING
        ));
    }

    if trim_spaces(body).is_empty() {
        return Err("Invalid field list: a field list must declare at least one field".into());
    }

    let mut fields = Vec::new();

    for entry in split_field_entries(body, delimiter)? {
        let entry = trim_spaces(entry);
        let (name_token, group) = match entry.find('{') {
            Some(_) if entry.ends_with('}') => {
                let brace = match entry.starts_with('"') {
                    true => {
                        let end =
                            quoted_token_end(entry).ok_or("Unterminated quoted field name")?;
                        entry[end + 1..].find('{').map(|offset| offset + end + 1)
                    }
                    false => entry.find('{'),
                };
                match brace {
                    Some(brace) => {
                        let end = field_list_end(&entry[brace..])
                            .ok_or("Invalid field list: unmatched brace in header")?;
                        (
                            &entry[..brace],
                            Some(parse_field_list(
                                &entry[brace + 1..brace + end],
                                delimiter,
                                strict,
                                nesting + 1,
                            )?),
                        )
                    }
                    None => (entry, None),
                }
            }
            _ => (entry, None),
        };

        let name = decode_field_name(trim_spaces(name_token))?;

        if strict && fields.iter().any(|f: &Field| f.name() == name) {
            return Err(format!("Duplicate field name '{}' in a field list", name));
        }

        fields.push(match group {
            Some(children) => Field::Group(name, children),
            None => Field::Leaf(name),
        });
    }

    Ok(fields)
}

/// Split a field list on the active delimiter at its top brace level,
/// rejecting any other unquoted delimiter character (Section 6).
fn split_field_entries(body: &str, delimiter: char) -> Result<Vec<&str>, String> {
    let mut entries = Vec::new();
    let mut start = 0;
    let mut brace_depth = 0usize;
    let mut in_quotes = false;
    let mut escaped = false;

    for (i, ch) in body.char_indices() {
        if in_quotes {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quotes = false;
            }
            continue;
        }

        match ch {
            '"' => in_quotes = true,
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            _ if ch == delimiter => {
                // A nested group is split by its own recursive call.
                if brace_depth == 0 {
                    entries.push(&body[start..i]);
                    start = i + ch.len_utf8();
                }
            }
            ',' | '\t' | '|' => {
                return Err(
                    "Header delimiter mismatch between the bracket segment and the field list"
                        .into(),
                );
            }
            _ => {}
        }
    }

    entries.push(&body[start..]);
    Ok(entries)
}

fn decode_field_name(token: &str) -> Result<String, String> {
    if !token.starts_with('"') {
        if token.is_empty() {
            return Err("Invalid field list: empty field name".into());
        }
        return Ok(token.to_string());
    }

    match quoted_token_end(token) {
        Some(end) if end + 1 == token.len() => unescape(&token[1..end]),
        Some(_) => Err("Characters after a quoted field name's closing quote".into()),
        None => Err("Unterminated quoted field name".into()),
    }
}

/// Split a delimited value sequence, preserving empty tokens and trimming
/// spaces around each one (Section 11.2).
fn split_cells(s: &str, delimiter: char) -> Vec<&str> {
    if s.is_empty() {
        return Vec::new();
    }

    let mut cells = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    let mut escaped = false;

    for (i, ch) in s.char_indices() {
        if in_quotes {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quotes = false;
            }
            continue;
        }

        if ch == '"' {
            in_quotes = true;
        } else if ch == delimiter {
            cells.push(trim_spaces(&s[start..i]));
            start = i + ch.len_utf8();
        }
    }

    cells.push(trim_spaces(&s[start..]));
    cells
}

/// Decide whether a line at row depth is a row or the key-value line that
/// ends the rows (Section 9.3).
fn is_row_line(content: &str, delimiter: char) -> bool {
    match (
        find_unquoted(content, delimiter),
        find_unquoted(content, ':'),
    ) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(d), Some(c)) => d < c,
    }
}

/// Unescape the body of a quoted token per the Section 7.1 escape table.
fn unescape(body: &str) -> Result<String, String> {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }

        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => {
                let mut code = 0u32;
                for _ in 0..4 {
                    let digit = chars
                        .next()
                        .and_then(|c| c.to_digit(16))
                        .ok_or("Truncated \\uXXXX escape sequence")?;
                    code = code * 16 + digit;
                }
                // Lone surrogates are rejected (Section 7.1).
                let ch = char::from_u32(code)
                    .ok_or_else(|| format!("Escape \\u{:04x} is not a Unicode scalar", code))?;
                out.push(ch);
            }
            Some(other) => return Err(format!("Invalid escape sequence: \\{}", other)),
            None => return Err("Unterminated escape sequence".into()),
        }
    }

    Ok(out)
}

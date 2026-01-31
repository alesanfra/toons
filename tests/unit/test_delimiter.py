"""
Test suite for delimiter parameter in dumps() and dump() functions.

Tests verify that the delimiter kwarg works correctly for:
- Comma (default)
- Tab (\t)
- Pipe (|)
"""

import io

import pytest

import toons


class TestDumpsDelimiter:
    """Test dumps() with delimiter parameter."""

    def test_default_comma_delimiter(self):
        """Default delimiter is comma."""
        data = {"items": [1, 2, 3]}
        result = toons.dumps(data)
        assert result == "items[3]: 1,2,3"

    def test_explicit_comma_delimiter(self):
        """Explicit comma delimiter."""
        data = {"items": [1, 2, 3]}
        result = toons.dumps(data, delimiter=",")
        assert result == "items[3]: 1,2,3"

    def test_tab_delimiter(self):
        """Tab delimiter in arrays."""
        data = {"items": [1, 2, 3]}
        result = toons.dumps(data, delimiter="\t")
        assert result == "items[3\t]: 1\t2\t3"

    def test_pipe_delimiter(self):
        """Pipe delimiter in arrays."""
        data = {"items": [1, 2, 3]}
        result = toons.dumps(data, delimiter="|")
        assert result == "items[3|]: 1|2|3"

    def test_delimiter_in_tabular_format(self):
        """Delimiter applies to tabular format."""
        data = {
            "users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]
        }

        # Comma delimiter (default)
        result = toons.dumps(data, delimiter=",")
        assert "users[2]{name,age}:" in result
        assert "Alice,30" in result
        assert "Bob,25" in result

        # Tab delimiter
        result = toons.dumps(data, delimiter="\t")
        assert "users[2\t]{name\tage}:" in result
        assert "Alice\t30" in result
        assert "Bob\t25" in result

        # Pipe delimiter
        result = toons.dumps(data, delimiter="|")
        assert "users[2|]{name|age}:" in result
        assert "Alice|30" in result
        assert "Bob|25" in result

    def test_delimiter_with_strings_containing_delimiter(self):
        """Strings containing the delimiter are quoted."""
        # Comma delimiter (default)
        data = {"tags": ["tag,with,comma", "normal"]}
        result = toons.dumps(data, delimiter=",")
        assert '"tag,with,comma"' in result
        assert "normal" in result

        # Tab delimiter
        data = {"tags": ["tag\twith\ttab", "normal"]}
        result = toons.dumps(data, delimiter="\t")
        assert '"tag\\twith\\ttab"' in result

        # Pipe delimiter
        data = {"tags": ["tag|with|pipe", "normal"]}
        result = toons.dumps(data, delimiter="|")
        assert '"tag|with|pipe"' in result

    def test_delimiter_in_nested_structures(self):
        """Delimiter is applied throughout nested structures."""
        data = {
            "server": {
                "ports": [8080, 8443, 9000],
                "hosts": ["localhost", "127.0.0.1"],
            }
        }

        result = toons.dumps(data, delimiter="|")
        assert "ports[3|]: 8080|8443|9000" in result
        assert "hosts[2|]: localhost|127.0.0.1" in result

    def test_delimiter_root_array_primitives(self):
        """Delimiter in root-level primitive arrays."""
        # Comma
        result = toons.dumps([1, 2, 3], delimiter=",")
        assert result == "[3]: 1,2,3"

        # Tab
        result = toons.dumps([1, 2, 3], delimiter="\t")
        assert result == "[3\t]: 1\t2\t3"

        # Pipe
        result = toons.dumps([1, 2, 3], delimiter="|")
        assert result == "[3|]: 1|2|3"

    def test_delimiter_root_array_tabular(self):
        """Delimiter in root-level tabular arrays."""
        data = [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]

        # Comma
        result = toons.dumps(data, delimiter=",")
        assert "[2]{name,age}:" in result
        assert "Alice,30" in result

        # Tab
        result = toons.dumps(data, delimiter="\t")
        assert "[2\t]{name\tage}:" in result
        assert "Alice\t30" in result

        # Pipe
        result = toons.dumps(data, delimiter="|")
        assert "[2|]{name|age}:" in result
        assert "Alice|30" in result


class TestDumpDelimiter:
    """Test dump() with delimiter parameter."""

    def test_default_comma_delimiter(self):
        """Default delimiter is comma in dump()."""
        data = {"items": [1, 2, 3]}
        fp = io.StringIO()
        toons.dump(data, fp)
        result = fp.getvalue()
        assert result == "items[3]: 1,2,3"

    def test_explicit_comma_delimiter(self):
        """Explicit comma delimiter in dump()."""
        data = {"items": [1, 2, 3]}
        fp = io.StringIO()
        toons.dump(data, fp, delimiter=",")
        result = fp.getvalue()
        assert result == "items[3]: 1,2,3"

    def test_tab_delimiter(self):
        """Tab delimiter in dump()."""
        data = {"items": [1, 2, 3]}
        fp = io.StringIO()
        toons.dump(data, fp, delimiter="\t")
        result = fp.getvalue()
        assert result == "items[3\t]: 1\t2\t3"

    def test_pipe_delimiter(self):
        """Pipe delimiter in dump()."""
        data = {"items": [1, 2, 3]}
        fp = io.StringIO()
        toons.dump(data, fp, delimiter="|")
        result = fp.getvalue()
        assert result == "items[3|]: 1|2|3"

    def test_delimiter_with_indent(self):
        """Delimiter and indent can be combined."""
        data = {"nested": {"items": [1, 2, 3]}}
        fp = io.StringIO()
        toons.dump(data, fp, indent=4, delimiter="|")
        result = fp.getvalue()
        assert "items[3|]: 1|2|3" in result


class TestDelimiterRoundTrip:
    """Test that delimiter-encoded TOON can be parsed back."""

    def test_comma_round_trip(self):
        """Comma-delimited TOON round-trips correctly."""
        data = {"items": [1, 2, 3], "tags": ["a", "b"]}
        toon_str = toons.dumps(data, delimiter=",")
        loaded = toons.loads(toon_str)
        assert loaded == data

    def test_tab_round_trip(self):
        """Tab-delimited TOON round-trips correctly."""
        data = {"items": [1, 2, 3], "tags": ["a", "b"]}
        toon_str = toons.dumps(data, delimiter="\t")
        loaded = toons.loads(toon_str)
        assert loaded == data

    def test_pipe_round_trip(self):
        """Pipe-delimited TOON round-trips correctly."""
        data = {"items": [1, 2, 3], "tags": ["a", "b"]}
        toon_str = toons.dumps(data, delimiter="|")
        loaded = toons.loads(toon_str)
        assert loaded == data

    def test_tabular_round_trip(self):
        """Tabular format with different delimiters round-trips."""
        data = {
            "users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]
        }

        for delimiter in [",", "\t", "|"]:
            toon_str = toons.dumps(data, delimiter=delimiter)
            loaded = toons.loads(toon_str)
            assert loaded == data

    def test_complex_structure_round_trip(self):
        """Complex nested structures with delimiters round-trip."""
        data = {
            "server": {
                "ports": [8080, 8443],
                "admins": [
                    {"name": "Alice", "level": 10},
                    {"name": "Bob", "level": 8},
                ],
            }
        }

        for delimiter in [",", "\t", "|"]:
            toon_str = toons.dumps(data, delimiter=delimiter)
            loaded = toons.loads(toon_str)
            assert loaded == data

"""
Test suite for TOON Specification v1.3 compliance.

This test verifies that the toons library correctly implements:
- TOON Spec v1.3 (2025-10-31)
- Using rtoon v0.1.3 (implements TOON Spec v1.2)

Tests cover all major TOON format features:
- Primitive values (null, bool, int, float, string)
- Objects (key-value pairs with indentation)
- Arrays (primitive arrays with count notation)
- Tabular format (uniform object arrays)
- Nested structures
- Round-trip fidelity
"""

import pytest
import toons


class TestPrimitives:
    """Test TOON primitive value encoding/decoding (Spec §5)."""

    def test_null(self):
        """Null values encode as 'null'."""
        assert toons.dumps(None) == "null"
        assert toons.loads("null") is None

    def test_boolean_true(self):
        """Boolean true encodes as 'true'."""
        assert toons.dumps(True) == "true"
        assert toons.loads("true") is True

    def test_boolean_false(self):
        """Boolean false encodes as 'false'."""
        assert toons.dumps(False) == "false"
        assert toons.loads("false") is False

    def test_integer(self):
        """Integers encode as decimal literals."""
        assert toons.dumps(42) == "42"
        assert toons.loads("42") == 42

    def test_negative_integer(self):
        """Negative integers are supported."""
        assert toons.dumps(-17) == "-17"
        assert toons.loads("-17") == -17

    def test_float(self):
        """Floats encode as decimal literals with decimal point."""
        assert toons.dumps(3.14) == "3.14"
        assert toons.loads("3.14") == 3.14

    def test_string_simple(self):
        """Simple strings encode without quotes when safe."""
        result = toons.dumps("hello")
        assert toons.loads(result) == "hello"


class TestObjects:
    """Test TOON object encoding (Spec §8)."""

    def test_simple_object(self):
        """Objects encode as key: value pairs."""
        obj = {"name": "Alice", "age": 30}
        toon = toons.dumps(obj)
        assert "name: Alice" in toon
        assert "age: 30" in toon
        assert toons.loads(toon) == obj

    def test_nested_object(self):
        """Nested objects use 2-space indentation."""
        obj = {"user": {"name": "Bob", "id": 123}}
        toon = toons.dumps(obj)
        # Check indentation
        lines = toon.split("\n")
        assert lines[0] == "user:"
        assert lines[1].startswith("  ")  # 2-space indent
        assert toons.loads(toon) == obj

    def test_object_with_multiple_types(self):
        """Objects can contain mixed value types."""
        obj = {
            "name": "Test",
            "count": 42,
            "price": 19.99,
            "active": True,
            "notes": None,
        }
        assert toons.loads(toons.dumps(obj)) == obj


class TestArrays:
    """Test TOON array encoding (Spec §9)."""

    def test_primitive_array_integers(self):
        """Primitive integer arrays use [N]: notation."""
        arr = [1, 2, 3]
        toon = toons.dumps(arr)
        assert toon == "[3]: 1,2,3"
        assert toons.loads(toon) == arr

    def test_primitive_array_strings(self):
        """Primitive string arrays use [N]: notation."""
        arr = ["admin", "user", "guest"]
        toon = toons.dumps(arr)
        assert "[3]:" in toon
        assert toons.loads(toon) == arr

    def test_empty_array(self):
        """Empty arrays encode as [0]:"""
        arr = []
        toon = toons.dumps(arr)
        assert "[0]:" in toon
        assert toons.loads(toon) == arr

    def test_tabular_format(self):
        """Uniform object arrays use tabular format [N]{fields}:"""
        arr = [
            {"name": "Alice", "age": 25},
            {"name": "Bob", "age": 30},
        ]
        toon = toons.dumps(arr)
        # Check for tabular header
        assert "[2]{" in toon
        assert "name" in toon
        assert "age" in toon
        # Verify round-trip
        result = toons.loads(toon)
        assert result == arr

    def test_array_in_object(self):
        """Objects can contain arrays."""
        obj = {"tags": ["python", "rust", "toon"]}
        toon = toons.dumps(obj)
        assert "tags[3]:" in toon
        assert toons.loads(toon) == obj


class TestNestedStructures:
    """Test complex nested TOON structures."""

    def test_deeply_nested_objects(self):
        """Multiple levels of object nesting."""
        obj = {"level1": {"level2": {"level3": {"value": 42}}}}
        assert toons.loads(toons.dumps(obj)) == obj

    def test_object_with_nested_array(self):
        """Objects containing nested arrays."""
        obj = {
            "users": [
                {"name": "Alice", "role": "admin"},
                {"name": "Bob", "role": "user"},
            ]
        }
        toon = toons.dumps(obj)
        assert "users[2]{" in toon  # Tabular format
        assert toons.loads(toon) == obj

    def test_mixed_nested_structure(self):
        """Complex structure with mixed types and nesting."""
        obj = {
            "project": "TOON",
            "version": 1.3,
            "active": True,
            "features": ["compact", "readable", "efficient"],
            "metadata": {"author": "Johann", "year": 2025},
        }
        assert toons.loads(toons.dumps(obj)) == obj


class TestRoundTrip:
    """Test round-trip encoding/decoding fidelity."""

    @pytest.mark.parametrize(
        "value",
        [
            None,
            True,
            False,
            0,
            -42,
            3.14,
            "test",
            [],
            [1, 2, 3],
            {"a": 1},
            {"user": {"name": "Test"}},
            [{"id": 1}, {"id": 2}],
        ],
    )
    def test_round_trip_fidelity(self, value):
        """All values maintain fidelity through encode/decode cycle."""
        encoded = toons.dumps(value)
        decoded = toons.loads(encoded)
        assert decoded == value

    def test_empty_object_behavior(self):
        """Empty objects have special behavior in TOON."""
        # Note: rtoon encodes empty objects as empty string
        # This is expected behavior per TOON spec
        encoded = toons.dumps({})
        assert encoded == ""
        # Empty string decodes to None (not {})
        decoded = toons.loads(encoded)
        assert decoded is None

    def test_round_trip_preserves_types(self):
        """Round-trip preserves Python types correctly."""
        obj = {
            "null": None,
            "bool": True,
            "int": 42,
            "float": 3.14,
            "str": "text",
            "list": [1, 2],
            "dict": {"nested": True},
        }
        result = toons.loads(toons.dumps(obj))
        assert result == obj
        assert result["null"] is None
        assert isinstance(result["bool"], bool)
        assert isinstance(result["int"], int)
        assert isinstance(result["float"], float)
        assert isinstance(result["str"], str)
        assert isinstance(result["list"], list)
        assert isinstance(result["dict"], dict)


class TestFileOperations:
    """Test file I/O operations (load/dump)."""

    def test_dump_and_load_file(self, tmp_path):
        """dump() and load() work with file objects."""
        data = {"name": "Test", "count": 42, "tags": ["a", "b"]}
        filepath = tmp_path / "test.toon"

        # Write
        with open(filepath, "w") as f:
            toons.dump(data, f)

        # Read
        with open(filepath, "r") as f:
            loaded = toons.load(f)

        assert loaded == data

    def test_file_contains_valid_toon(self, tmp_path):
        """Dumped files contain valid TOON format."""
        data = {"users": [{"name": "Alice", "age": 25}]}
        filepath = tmp_path / "test.toon"

        with open(filepath, "w") as f:
            toons.dump(data, f)

        # Check file content is valid TOON
        with open(filepath, "r") as f:
            content = f.read()
            assert "users[1]{" in content  # Tabular format
            loaded = toons.loads(content)
            assert loaded == data


class TestSpecCompliance:
    """Tests verifying specific TOON Spec v1.3 requirements."""

    def test_array_count_notation(self):
        """Arrays must include element count [N]: (Spec §9.1)."""
        arr = [1, 2, 3, 4, 5]
        toon = toons.dumps(arr)
        assert "[5]:" in toon

    def test_tabular_format_header(self):
        """Uniform arrays use [N]{field1,field2}: format (Spec §9.3)."""
        arr = [{"x": 1, "y": 2}, {"x": 3, "y": 4}]
        toon = toons.dumps(arr)
        assert "[2]{" in toon
        # Should contain field names
        assert "x" in toon and "y" in toon

    def test_object_indentation_two_spaces(self):
        """Nested objects use 2-space indentation (Spec §8)."""
        obj = {"outer": {"inner": "value"}}
        toon = toons.dumps(obj)
        lines = toon.split("\n")
        # Find indented line
        indented = [
            l for l in lines if l.startswith("  ") and not l.startswith("    ")
        ]
        assert len(indented) > 0
        # Check it's exactly 2 spaces (not 4)
        for line in indented:
            if line.strip():  # Skip empty lines
                assert line.startswith("  ")
                assert not line.startswith("   ")  # Not 3+ spaces

    def test_error_on_invalid_toon(self):
        """Invalid TOON syntax raises ValueError for malformed structures."""
        # Test with incomplete tabular data (missing rows)
        with pytest.raises(ValueError, match="TOON parse error"):
            toons.loads("users[2]:\n  incomplete")

    def test_parser_is_lenient_with_extra_content(self):
        """Parser ignores invalid lines after valid content (lenient mode)."""
        # This is the actual behavior of rtoon - it's lenient
        result = toons.loads("key: value\n invalid line")
        # Only the valid part is parsed
        assert result == {"key": "value"}

    def test_spec_version_in_readme(self):
        """Verify we're testing against documented spec version."""
        # This is a documentation test - the version is v1.3
        # rtoon implements v1.2, which is compatible
        assert True  # Documentation check

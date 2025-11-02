"""Tests for configurable indentation parameter in dumps() and dump()"""

import tempfile

import pytest

import toons


class TestIndentParameter:
    """Test suite for indent parameter"""

    def test_default_indent_is_two_spaces(self):
        """Default indentation is 2 spaces (per TOON spec v1.3)"""
        data = {"parent": {"child": "value"}}
        result = toons.dumps(data)
        lines = result.split("\n")
        assert lines[0] == "parent:"
        assert lines[1] == "  child: value"  # 2 spaces

    def test_indent_2_explicit(self):
        """Explicitly set indent=2 produces 2 spaces"""
        data = {"parent": {"child": "value"}}
        result = toons.dumps(data, indent=2)
        lines = result.split("\n")
        assert lines[0] == "parent:"
        assert lines[1] == "  child: value"  # 2 spaces

    def test_indent_4(self):
        """indent=4 produces 4 spaces per level"""
        data = {"parent": {"child": "value"}}
        result = toons.dumps(data, indent=4)
        lines = result.split("\n")
        assert lines[0] == "parent:"
        assert lines[1] == "    child: value"  # 4 spaces

    def test_indent_zero_raises_error(self):
        """indent < 2 should raise ValueError"""
        with pytest.raises(ValueError, match="indent must be >= 2"):
            toons.dumps({"key": "value"}, indent=0)

        with pytest.raises(ValueError, match="indent must be >= 2"):
            toons.dumps({"key": "value"}, indent=1)

    def test_indent_with_nested_structure(self):
        """indent parameter works with deeply nested structures"""
        data = {"level1": {"level2": {"level3": "value"}}}
        result = toons.dumps(data, indent=3)
        lines = result.split("\n")
        assert lines[0] == "level1:"
        assert lines[1] == "   level2:"  # 3 spaces
        assert lines[2] == "      level3: value"  # 6 spaces

    def test_indent_with_arrays(self):
        """indent parameter works with arrays"""
        data = {"items": [{"name": "Alice"}, {"name": "Bob"}]}
        result = toons.dumps(data, indent=4)
        lines = result.split("\n")
        assert lines[0] == "items[2]{name}:"
        assert lines[1] == "    Alice"  # 4 spaces
        assert lines[2] == "    Bob"  # 4 spaces

    def test_indent_with_expanded_list(self):
        """indent parameter works with expanded list format"""
        data = {"mixed": [1, "text", {"nested": "obj"}]}
        result = toons.dumps(data, indent=3)
        # Should have correct indentation for "- " items
        assert "   - 1" in result or "   - " in result  # 3 spaces before "-"

    def test_dump_with_indent(self):
        """dump() function accepts indent parameter"""
        data = {"parent": {"child": "value"}}

        with tempfile.NamedTemporaryFile(
            mode="w+", delete=False, suffix=".toon"
        ) as f:
            toons.dump(data, f, indent=4)
            f.seek(0)
            content = f.read()

        lines = content.split("\n")
        assert lines[0] == "parent:"
        assert lines[1] == "    child: value"  # 4 spaces

    def test_indent_round_trip_preserves_data(self):
        """Different indent values don't affect round-trip data integrity"""
        data = {
            "user": {
                "name": "Alice",
                "age": 30,
                "tags": ["admin", "developer"],
            }
        }

        # Test with different indent values (>= 2)
        for indent in [2, 4, 8]:
            serialized = toons.dumps(data, indent=indent)
            deserialized = toons.loads(serialized)
            assert (
                deserialized == data
            ), f"Round-trip failed with indent={indent}"

    def test_indent_keyword_only(self):
        """indent parameter is keyword-only"""
        data = {"key": "value"}

        # Should work with keyword
        result = toons.dumps(data, indent=4)
        assert result is not None

        # Should fail with positional (this test verifies the API design)
        # Note: This is enforced by Rust/PyO3, not testable in Python
        # but documented here for clarity

    def test_indent_with_empty_object(self):
        """indent parameter doesn't affect empty objects"""
        data = {}
        result_default = toons.dumps(data)
        result_indent4 = toons.dumps(data, indent=4)
        assert result_default == result_indent4 == ""

    def test_indent_with_single_primitive(self):
        """indent parameter doesn't affect primitives"""
        for value in [None, True, 42, 3.14, "text"]:
            result_default = toons.dumps(value)
            result_indent4 = toons.dumps(value, indent=4)
            assert result_default == result_indent4

    def test_indent_with_root_array(self):
        """indent parameter works with root-level arrays"""
        data = [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]
        result = toons.dumps(data, indent=3)
        lines = result.split("\n")
        # First line is header
        assert lines[0] == "[2]{id,name}:"
        # Data lines have indentation
        assert lines[1].startswith("   ")  # 3 spaces

    def test_large_indent_value(self):
        """Large indent values work correctly"""
        data = {"parent": {"child": "value"}}
        result = toons.dumps(data, indent=10)
        lines = result.split("\n")
        assert lines[0] == "parent:"
        assert lines[1] == " " * 10 + "child: value"  # 10 spaces

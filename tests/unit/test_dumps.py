"""
Test suite specifically for toons.dumps() function.

Tests verify the exact TOON output format for common data structures.
All tests use pytest.mark.parametrize for concise, readable test cases.
"""

import pytest
import toons


class TestDumpsObjects:
    """Test dumps() with various object structures."""

    @pytest.mark.parametrize(
        "obj,expected",
        [
            # Simple flat objects
            (
                {"name": "Alice"},
                "name: Alice",
            ),
            (
                {"age": 30},
                "age: 30",
            ),
            (
                {"active": True},
                "active: true",
            ),
            (
                {"price": 19.99},
                "price: 19.99",
            ),
            (
                {"value": None},
                "value: null",
            ),
            # Multi-field objects (keys are sorted)
            (
                {"name": "Bob", "age": 25},
                "name: Bob\nage: 25",
            ),
            (
                {"active": True, "count": 42, "name": "Test"},
                "active: true\ncount: 42\nname: Test",
            ),
        ],
    )
    def test_flat_objects(self, obj, expected):
        """Flat objects encode as key: value pairs."""
        result = toons.dumps(obj)
        assert result == expected

    @pytest.mark.parametrize(
        "obj,expected_lines",
        [
            # Nested objects with 2-space indentation
            (
                {"user": {"name": "Alice"}},
                ["user:", "  name: Alice"],
            ),
            (
                {"user": {"name": "Bob", "id": 123}},
                ["user:", "  name: Bob", "  id: 123"],
            ),
            (
                {"config": {"enabled": True, "port": 8080}},
                ["config:", "  enabled: true", "  port: 8080"],
            ),
        ],
    )
    def test_nested_objects(self, obj, expected_lines):
        """Nested objects use 2-space indentation."""
        result = toons.dumps(obj)
        lines = result.split("\n")
        assert lines == expected_lines

    @pytest.mark.parametrize(
        "obj,expected_lines",
        [
            # Deeply nested structures
            (
                {"level1": {"level2": {"value": 42}}},
                ["level1:", "  level2:", "    value: 42"],
            ),
            (
                {"app": {"db": {"host": "localhost", "port": 5432}}},
                ["app:", "  db:", "    host: localhost", "    port: 5432"],
            ),
        ],
    )
    def test_deeply_nested_objects(self, obj, expected_lines):
        """Multiple nesting levels maintain consistent indentation."""
        result = toons.dumps(obj)
        lines = result.split("\n")
        assert lines == expected_lines


class TestDumpsLists:
    """Test dumps() with various list structures."""

    @pytest.mark.parametrize(
        "arr,expected",
        [
            # Empty array
            ([], "[0]:"),
            # Single element arrays
            ([1], "[1]: 1"),
            (["a"], "[1]: a"),
            ([True], "[1]: true"),
            # Multiple element arrays
            ([1, 2, 3], "[3]: 1,2,3"),
            ([10, 20, 30, 40, 50], "[5]: 10,20,30,40,50"),
            (["admin", "user", "guest"], "[3]: admin,user,guest"),
            ([True, False, True], "[3]: true,false,true"),
            # Mixed primitive types
            ([1, "text", True], "[3]: 1,text,true"),
        ],
    )
    def test_primitive_arrays(self, arr, expected):
        """Primitive arrays use [N]: element1,element2,... format."""
        result = toons.dumps(arr)
        assert result == expected

    def test_empty_array(self):
        """Empty arrays encode as [0]:"""
        result = toons.dumps([])
        assert result == "[0]:"

    @pytest.mark.parametrize(
        "arr,expected_pattern",
        [
            # Arrays in objects
            (
                {"tags": ["python", "rust"]},
                "tags[2]: python,rust",
            ),
            (
                {"numbers": [1, 2, 3, 4]},
                "numbers[4]: 1,2,3,4",
            ),
            (
                {"flags": [True, False]},
                "flags[2]: true,false",
            ),
        ],
    )
    def test_arrays_in_objects(self, arr, expected_pattern):
        """Arrays within objects maintain [N]: notation."""
        result = toons.dumps(arr)
        assert expected_pattern in result


class TestDumpsTabular:
    """Test dumps() with tabular format for uniform object arrays."""

    @pytest.mark.parametrize(
        "arr,expected",
        [
            # Simple tabular data (2 fields)
            (
                [{"name": "Alice", "age": 25}, {"name": "Bob", "age": 30}],
                "[2]{name,age}:\n  Alice,25\n  Bob,30",
            ),
            # More rows
            (
                [
                    {"id": 1, "active": True},
                    {"id": 2, "active": False},
                    {"id": 3, "active": True},
                ],
                "[3]{id,active}:\n  1,true\n  2,false\n  3,true",
            ),
            # Different field order (fields are sorted in header)
            (
                [{"x": 10, "y": 20}, {"x": 30, "y": 40}],
                "[2]{x,y}:\n  10,20\n  30,40",
            ),
        ],
    )
    def test_uniform_object_arrays(self, arr, expected):
        """Uniform object arrays use tabular [N]{field1,field2}: format."""
        result = toons.dumps(arr)
        assert result == expected

    def test_single_row_tabular(self):
        """Single-row tabular format."""
        arr = [{"name": "Alice", "role": "admin"}]
        result = toons.dumps(arr)
        assert result == "[1]{name,role}:\n  Alice,admin"

    @pytest.mark.parametrize(
        "arr,expected_header,expected_rows",
        [
            # Verify header and row format separately
            (
                [{"user": "alice", "score": 95}, {"user": "bob", "score": 87}],
                "[2]{user,score}:",
                ["  alice,95", "  bob,87"],
            ),
            (
                [
                    {"product": "Widget", "price": 9.99},
                    {"product": "Gadget", "price": 14.5},
                ],
                "[2]{product,price}:",
                ["  Widget,9.99", "  Gadget,14.5"],
            ),
        ],
    )
    def test_tabular_structure(self, arr, expected_header, expected_rows):
        """Tabular format has correct header and row structure."""
        result = toons.dumps(arr)
        lines = result.split("\n")
        assert lines[0] == expected_header
        assert lines[1:] == expected_rows


class TestDumpsComplexStructures:
    """Test dumps() with complex nested structures."""

    def test_object_with_array(self):
        """Object containing an array."""
        obj = {"user": "Alice", "tags": ["python", "rust", "go"]}
        result = toons.dumps(obj)
        assert "user: Alice" in result
        assert "tags[3]: python,rust,go" in result

    def test_object_with_nested_tabular_array(self):
        """Object containing tabular array."""
        obj = {
            "project": "TOON",
            "contributors": [
                {"name": "Alice", "commits": 50},
                {"name": "Bob", "commits": 30},
            ],
        }
        result = toons.dumps(obj)
        lines = result.split("\n")

        # Check structure
        assert "project: TOON" in result
        assert "contributors[2]{name,commits}:" in result
        assert "  Alice,50" in result
        assert "  Bob,30" in result

    @pytest.mark.parametrize(
        "obj,expected_lines",
        [
            # Nested object with arrays (note: arrays in nested objects get extra indent)
            (
                {
                    "server": {
                        "host": "localhost",
                        "ports": [8080, 8443],
                    }
                },
                ["server:", "  host: localhost", "    ports[2]: 8080,8443"],
            ),
            # Multiple nested levels
            (
                {
                    "app": {
                        "name": "MyApp",
                        "config": {"debug": True, "timeout": 30},
                    }
                },
                [
                    "app:",
                    "  name: MyApp",
                    "  config:",
                    "    debug: true",
                    "    timeout: 30",
                ],
            ),
        ],
    )
    def test_mixed_nested_structures(self, obj, expected_lines):
        """Complex nested structures with objects and arrays."""
        result = toons.dumps(obj)
        lines = result.split("\n")
        assert lines == expected_lines


class TestDumpsPrimitives:
    """Test dumps() with primitive values at root level."""

    @pytest.mark.parametrize(
        "value,expected",
        [
            (None, "null"),
            (True, "true"),
            (False, "false"),
            (0, "0"),
            (42, "42"),
            (-17, "-17"),
            (3.14, "3.14"),
            (-99.99, "-99.99"),
            ("hello", "hello"),
            ("world", "world"),
        ],
    )
    def test_primitive_values(self, value, expected):
        """Primitive values encode correctly at root level."""
        result = toons.dumps(value)
        assert result == expected

    def test_string_with_spaces_quoted(self):
        """Strings with spaces are quoted."""
        result = toons.dumps("test string")
        assert result == '"test string"'


class TestDumpsEdgeCases:
    """Test dumps() with edge cases and special values."""

    def test_empty_object(self):
        """Empty objects encode as empty string."""
        result = toons.dumps({})
        assert result == ""

    def test_zero_values(self):
        """Zero values are preserved."""
        obj = {"count": 0, "price": 0.0}
        result = toons.dumps(obj)
        assert "count: 0" in result
        assert "price: 0" in result or "price: 0.0" in result

    def test_negative_numbers(self):
        """Negative numbers are handled correctly."""
        obj = {"balance": -100, "temperature": -5.5}
        result = toons.dumps(obj)
        assert "balance: -100" in result
        assert "temperature: -5.5" in result

    @pytest.mark.parametrize(
        "obj",
        [
            {"a": 1, "b": 2, "c": 3},
            {"users": [{"id": 1}, {"id": 2}]},
            {"nested": {"deep": {"value": 42}}},
        ],
    )
    def test_round_trip_exact_match(self, obj):
        """Dumped TOON can be loaded back to original object."""
        toon = toons.dumps(obj)
        loaded = toons.loads(toon)
        assert loaded == obj

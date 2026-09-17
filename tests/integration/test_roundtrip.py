"""Round-trip tests for structures that stress the encoder and decoder."""

import pytest

import toons


class TestNestedArrays:
    """Arrays nested inside arrays survive a round trip."""

    @pytest.mark.parametrize(
        "data,expected_toon",
        [
            (
                {"k": [[{"a": 1}, {"a": 2}]]},
                "k[1]:\n  - [2]{a}:\n    1\n    2",
            ),
            (
                [[{"a": 1}, {"a": 2}]],
                "[1]:\n  - [2]{a}:\n    1\n    2",
            ),
            (
                {"k": [[[1, 2], [3]]]},
                "k[1]:\n  - [2]:\n    - [2]: 1,2\n    - [1]: 3",
            ),
            (
                {"k": [{"a": 1, "b": [{"c": 1}, {"c": 2}]}]},
                "k[1]:\n  - a: 1\n    b[2]{c}:\n      1\n      2",
            ),
        ],
    )
    def test_nested_array_roundtrip(self, data, expected_toon):
        """Nested arrays encode to the expected text and decode back."""
        encoded = toons.dumps(data)
        assert encoded == expected_toon
        assert toons.loads(encoded) == data


class TestLargeIntegers:
    """Integers outside the 64-bit range keep their exact value."""

    @pytest.mark.parametrize(
        "value",
        [2**63, 2**70, -(2**80), 10**40],
    )
    def test_large_integer_roundtrip(self, value):
        """Large integers encode as digits and decode back as int."""
        encoded = toons.dumps({"value": value})
        assert encoded == f"value: {value}"

        decoded = toons.loads(encoded)["value"]
        assert decoded == value
        assert isinstance(decoded, int)


class TestTuples:
    """Tuples encode as arrays, like the json module does."""

    @pytest.mark.parametrize(
        "data,expected_toon",
        [
            ({"a": (1, 2)}, "a[2]: 1,2"),
            ({"a": ({"x": 1}, {"x": 2})}, "a[2]{x}:\n  1\n  2"),
            ((1, 2, 3), "[3]: 1,2,3"),
        ],
    )
    def test_tuple_encodes_as_array(self, data, expected_toon):
        """A tuple produces the same output as the equivalent list."""
        assert toons.dumps(data) == expected_toon


class TestPathExpansion:
    """Dotted keys expand into nested objects according to the mode."""

    @pytest.mark.parametrize(
        "toon_text,mode,expected",
        [
            ("user.name: Alice", None, {"user.name": "Alice"}),
            ("user.name: Alice", "off", {"user.name": "Alice"}),
            ("user.name: Alice", "safe", {"user": {"name": "Alice"}}),
            ("user.name: Alice", "always", {"user": {"name": "Alice"}}),
            ('"user.name": Alice', "safe", {"user.name": "Alice"}),
            ('"user.name": Alice', "always", {"user": {"name": "Alice"}}),
            ("a.b[2]: 1,2", "safe", {"a": {"b": [1, 2]}}),
            ('"a.b"[2]: 1,2', "safe", {"a.b": [1, 2]}),
            ('"a.b"[2]: 1,2', "always", {"a": {"b": [1, 2]}}),
        ],
    )
    def test_expand_paths_modes(self, toon_text, mode, expected):
        """A quoted key is expanded only in "always" mode."""
        assert toons.loads(toon_text, expand_paths=mode) == expected

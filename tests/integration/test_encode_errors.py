"""Tests for the errors raised by dumps() and dump() on invalid input."""

import io

import pytest

import toons


class TestUnsupportedTypes:
    """Values that have no TOON representation raise TypeError."""

    @pytest.mark.parametrize(
        "value",
        [
            {1, 2, 3},
            frozenset({1}),
            object(),
            (i for i in range(3)),
            print,
        ],
    )
    def test_unsupported_value_raises_type_error(self, value):
        """dumps() rejects values it cannot encode."""
        with pytest.raises(TypeError, match="not TOON serializable"):
            toons.dumps({"value": value})

    @pytest.mark.parametrize("key", [1, 2.5, None, (1, 2)])
    def test_non_string_key_raises_type_error(self, key):
        """dumps() rejects object keys that are not strings."""
        with pytest.raises(TypeError, match="keys must be strings"):
            toons.dumps({key: "value"})

    def test_dump_rejects_unsupported_value(self):
        """dump() applies the same rules as dumps()."""
        with pytest.raises(TypeError, match="not TOON serializable"):
            toons.dump({"value": {1, 2}}, io.StringIO())


class TestRecursionLimits:
    """Self-referencing and very deep structures are rejected, not crashed."""

    def test_self_referencing_dict(self):
        """A dict that contains itself raises ValueError."""
        data = {}
        data["self"] = data
        with pytest.raises(ValueError, match="Circular reference detected"):
            toons.dumps(data)

    def test_self_referencing_list(self):
        """A list that contains itself raises ValueError."""
        data = []
        data.append(data)
        with pytest.raises(ValueError, match="Circular reference detected"):
            toons.dumps(data)

    def test_shared_reference_is_not_circular(self):
        """The same object used twice side by side is not a cycle."""
        shared = {"a": 1}
        assert toons.dumps({"x": shared, "y": shared}) == (
            "[2:]{a}:\n  x: 1\n  y: 1"
        )

    def test_deeply_nested_uniform_columns(self):
        """A deep uniform column is bounded during form detection too."""
        value = {"x": 1}
        for _ in range(1500):
            value = {"g": value}
        with pytest.raises(ValueError, match="Maximum nesting depth"):
            toons.dumps({"a": [value, value]})

    def test_deeply_nested_structure(self):
        """Nesting beyond the encoder limit raises ValueError."""
        data = 1
        for _ in range(1500):
            data = {"nested": data}
        with pytest.raises(ValueError, match="Maximum nesting depth"):
            toons.dumps(data)


class TestOptionValidation:
    """Invalid option values raise ValueError instead of being ignored."""

    @pytest.mark.parametrize("delimiter", ["", ";", "||", "  ", "\n"])
    def test_invalid_delimiter(self, delimiter):
        """dumps() rejects delimiters outside the spec."""
        with pytest.raises(ValueError, match="delimiter must be"):
            toons.dumps([1, 2], delimiter=delimiter)

    @pytest.mark.parametrize("delimiter", [",", "\t", "|"])
    def test_valid_delimiter(self, delimiter):
        """The three delimiters of the spec are accepted."""
        assert toons.dumps([1, 2], delimiter=delimiter).endswith(
            f"1{delimiter}2"
        )

    @pytest.mark.parametrize("indent_size", [0, 1])
    def test_invalid_indent_size(self, indent_size):
        """dumps() requires at least two spaces of indentation."""
        with pytest.raises(ValueError, match="indent_size must be >= 2"):
            toons.dumps({"a": {"b": 1}}, indent_size=indent_size)

    def test_indent_alias_must_agree_with_indent_size(self):
        """Passing both spellings with different values is rejected."""
        with pytest.raises(ValueError, match="disagree"):
            toons.dumps({"a": {"b": 1}}, indent_size=2, indent=4)
        with pytest.raises(ValueError, match="disagree"):
            toons.loads("a: 1", indent_size=2, indent=4)

    def test_invalid_decode_indent_size(self):
        """loads() rejects an indent size of zero."""
        with pytest.raises(ValueError, match="indent_size must be >= 1"):
            toons.loads("a: 1", indent_size=0)

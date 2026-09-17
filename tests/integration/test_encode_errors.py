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
        assert (
            toons.dumps({"x": shared, "y": shared}) == "x:\n  a: 1\ny:\n  a: 1"
        )

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

    def test_invalid_key_folding(self):
        """dumps() rejects unknown key_folding modes."""
        with pytest.raises(ValueError, match="key_folding must be"):
            toons.dumps({"a": {"b": 1}}, key_folding="maybe")

    @pytest.mark.parametrize("indent", [0, 1])
    def test_invalid_indent(self, indent):
        """dumps() requires at least two spaces of indentation."""
        with pytest.raises(ValueError, match="indent must be >= 2"):
            toons.dumps({"a": {"b": 1}}, indent=indent)

    def test_invalid_expand_paths(self):
        """loads() rejects unknown expand_paths modes."""
        with pytest.raises(ValueError, match="expand_paths must be"):
            toons.loads("a.b: 1", expand_paths="maybe")

    def test_invalid_decode_indent(self):
        """loads() rejects an indent hint of zero."""
        with pytest.raises(ValueError, match="indent must be >= 1"):
            toons.loads("a: 1", indent=0)

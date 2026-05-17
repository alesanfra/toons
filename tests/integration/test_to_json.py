import json

import pytest

import toons


class TestToJson:
    """Integration tests for to_json() function."""

    def test_to_json_uses_default_indent(self):
        """to_json() converts TOON to pretty JSON by default."""
        toon_str = "name: Alice\nage: 30\ntags[2]: admin,user"
        result = toons.to_json(toon_str)

        assert result == json.dumps(
            {"name": "Alice", "age": 30, "tags": ["admin", "user"]},
            indent=2,
        )

    def test_to_json_accepts_indent_none(self):
        """to_json() passes indent=None through to json.dumps()."""
        toon_str = "name: Alice\nage: 30"
        result = toons.to_json(toon_str, indent=None)

        assert result == json.dumps({"name": "Alice", "age": 30}, indent=None)

    def test_to_json_respects_strict_flag(self):
        """to_json() forwards strict=False to the TOON parser."""
        toon_str = "[3]:\n  - 1\n\n  - 2\n  - 3"
        result = toons.to_json(toon_str, strict=False)

        assert result == json.dumps([1, 2, 3], indent=2)

    def test_to_json_raises_decode_error(self):
        """to_json() raises ToonDecodeError for malformed TOON."""
        with pytest.raises(toons.ToonDecodeError):
            toons.to_json("a:\n  b:\n     c: 1\n")

    def test_to_json_respects_expand_paths(self):
        """to_json() forwards expand_paths to the TOON parser."""
        result = toons.to_json("user.name: Alice", expand_paths="safe")

        assert result == json.dumps({"user": {"name": "Alice"}}, indent=2)

import io
import tempfile

import pytest

import toons


class TestSmokeLoads:
    """Minimal smoke test for loads() function."""

    def test_loads_basic_object(self):
        """loads() deserializes basic object."""
        toon_str = "name: Alice\nage: 30"
        result = toons.loads(toon_str)
        assert result == {"name": "Alice", "age": 30}

    def test_loads_nested_object(self):
        """loads() deserializes nested object."""
        toon_str = "user:\n  name: Bob\n  age: 25"
        result = toons.loads(toon_str)
        assert result == {"user": {"name": "Bob", "age": 25}}

    def test_loads_array(self):
        """loads() deserializes array."""
        toon_str = "[3]: 1,2,3"
        result = toons.loads(toon_str)
        assert result == [1, 2, 3]

    def test_loads_tabular(self):
        """loads() deserializes tabular format."""
        toon_str = "[2]{id,name}:\n  1,Alice\n  2,Bob"
        result = toons.loads(toon_str)
        assert result == [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]


class TestSmokeDumps:
    """Minimal smoke test for dumps() function."""

    def test_dumps_basic_object(self):
        """dumps() serializes basic object."""
        data = {"name": "Alice", "age": 30}
        result = toons.dumps(data)
        assert "name: Alice" in result
        assert "age: 30" in result

    def test_dumps_array(self):
        """dumps() serializes array."""
        data = [1, 2, 3]
        result = toons.dumps(data)
        assert result == "[3]: 1,2,3"

    def test_dumps_tabular(self):
        """dumps() serializes uniform object arrays."""
        data = [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]
        result = toons.dumps(data)
        assert "[2]{" in result
        assert "Alice" in result
        assert "Bob" in result

    def test_dumps_with_indent(self):
        """dumps() respects indent parameter."""
        data = {"parent": {"child": "value"}}
        result = toons.dumps(data, indent=4)
        lines = result.split("\n")
        assert lines[0] == "parent:"
        assert lines[1] == "    child: value"

    def test_dumps_with_delimiter(self):
        """dumps() respects delimiter parameter."""
        data = [1, 2, 3]
        result = toons.dumps(data, delimiter="|")
        assert result == "[3|]: 1|2|3"


class TestSmokeDump:
    """Minimal smoke test for dump() function."""

    def test_dump_writes_to_file(self):
        """dump() writes to file."""
        data = {"key": "value"}
        fp = io.StringIO()
        toons.dump(data, fp)
        result = fp.getvalue()
        assert result == "key: value"

    def test_dump_with_indent(self):
        """dump() respects indent parameter."""
        data = {"parent": {"child": "value"}}
        fp = io.StringIO()
        toons.dump(data, fp, indent=4)
        result = fp.getvalue()
        assert "    child: value" in result

    def test_dump_with_delimiter(self):
        """dump() respects delimiter parameter."""
        data = [1, 2, 3]
        fp = io.StringIO()
        toons.dump(data, fp, delimiter="|")
        assert "1|2|3" in fp.getvalue()


class TestSmokeLoad:
    """Minimal smoke test for load() function."""

    def test_load_from_file(self):
        """load() reads from file."""
        with tempfile.NamedTemporaryFile(mode="w+", suffix=".toon") as f:
            f.write("key: value")
            f.seek(0)
            result = toons.load(f)
            assert result == {"key": "value"}

    def test_load_array(self):
        """load() deserializes array from file."""
        with tempfile.NamedTemporaryFile(mode="w+", suffix=".toon") as f:
            f.write("[3]: 1,2,3")
            f.seek(0)
            result = toons.load(f)
            assert result == [1, 2, 3]


class TestSmokeStrictFlag:
    """Minimal smoke test for strict parameter."""

    def test_strict_mode_default(self):
        """Strict mode is default in loads()."""
        toon_bad = "[3]:\n  - 1\n\n  - 2\n  - 3"
        with pytest.raises(ValueError, match="Blank line inside array"):
            toons.loads(toon_bad)

    def test_non_strict_mode(self):
        """Non-strict mode allows blank lines."""
        toon_bad = "[3]:\n  - 1\n\n  - 2\n  - 3"
        result = toons.loads(toon_bad, strict=False)
        assert result == [1, 2, 3]

    def test_strict_mode_load(self):
        """Strict mode applies to load()."""
        with tempfile.NamedTemporaryFile(mode="w+", suffix=".toon") as f:
            f.write("[3]:\n  - 1\n\n  - 2\n  - 3")
            f.seek(0)
            with pytest.raises(ValueError, match="Blank line inside array"):
                toons.load(f)


class TestSmokeUnicode:
    """Minimal smoke test for Unicode handling."""

    def test_unicode_in_dataset(self):
        d = {"Test®": [{"name": "a", "age": 2}]}
        s = toons.dumps(d)
        t = toons.loads(s)
        assert t == d

    def test_unicode_in_field(self):
        d = {"Test": [{"name®": "a", "age": 2}]}
        s = toons.dumps(d)
        t = toons.loads(s)
        assert t == d

    def test_unicode_in_value(self):
        d = {"Test": [{"name": "a®", "age": 2}]}
        s = toons.dumps(d)
        t = toons.loads(s)
        assert t == d

    def test_unicode_in_array(self):
        d = ["a®", "b", "c"]
        s = toons.dumps(d)
        t = toons.loads(s)
        assert t == d

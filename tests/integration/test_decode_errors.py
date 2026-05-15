"""Integration tests for ToonDecodeError and its structured attributes."""

import pytest

import toons


class TestToonDecodeErrorAttributes:
    """Confirm ToonDecodeError carries `.line` and `.source` for every
    parse-error path that has access to the line-oriented parser state."""

    def test_count_mismatch_reports_header_line(self):
        """Array length mismatch points at the header line, not the line after.

        The header on line 5 declares 3 inline pipe-delimited values but the
        following lines provide them in YAML-block form (a common LLM mistake).
        The exception must point at the header.
        """
        content = (
            "user_list:\n"
            "  users[1|]:\n"
            "    - name: Alice\n"
            "      role: editor\n"
            "      tags[3|]:\n"
            '        "admin"\n'
            '        "active"\n'
            '        "premium"\n'
        )
        with pytest.raises(toons.ToonDecodeError) as excinfo:
            toons.loads(content)
        exc = excinfo.value
        assert exc.line == 5
        assert exc.source == "      tags[3|]:"
        assert "TOON parse error at line 5" in str(exc)
        assert "Array declared length 3 but found 0 elements" in str(exc)

    def test_indentation_error_attaches_offending_line(self):
        """Strict-mode indentation errors include the offending line verbatim."""
        content = "a:\n  b:\n     c: 1\n"
        with pytest.raises(toons.ToonDecodeError) as excinfo:
            toons.loads(content)
        exc = excinfo.value
        assert exc.line == 3
        assert exc.source == "     c: 1"
        assert "Indentation 5 is not a multiple of indent size 2" in str(exc)

    def test_missing_colon_attaches_offending_line(self):
        """Object lines without a colon report the offending line."""
        content = "a: 1\nb\nc: 3\n"
        with pytest.raises(toons.ToonDecodeError) as excinfo:
            toons.loads(content)
        exc = excinfo.value
        assert exc.line == 2
        assert "b" in str(exc.source)


class TestToonDecodeErrorClassHierarchy:
    """ToonDecodeError MUST stay a subclass of ValueError for back-compat."""

    def test_is_subclass_of_value_error(self):
        assert issubclass(toons.ToonDecodeError, ValueError)

    def test_caught_as_value_error(self):
        """`except ValueError` still catches a ToonDecodeError."""
        with pytest.raises(ValueError):
            toons.loads("a:\n  b:\n     c: 1\n")

    def test_attributes_always_present(self):
        """`.line` and `.source` must exist on every raised instance,
        even if both are None."""
        try:
            toons.loads("a:\n  b:\n     c: 1\n")
        except toons.ToonDecodeError as exc:
            assert hasattr(exc, "line")
            assert hasattr(exc, "source")


class TestPathExpansionErrorClass:
    """Path expansion conflicts must also raise ToonDecodeError, not a
    bare ValueError — they are parse-time decode errors."""

    def test_path_expansion_conflict_raises_toon_decode_error(self):
        """Same key reached via plain and via path expansion conflicts when
        strict expand_paths='safe' detects the type mismatch."""
        # `parent: scalar` then a dotted key `parent.child: 1` would try to
        # turn `parent` into a dict — a type conflict in strict mode.
        content = "parent: scalar\nparent.child: 1\n"
        with pytest.raises(toons.ToonDecodeError):
            toons.loads(content, expand_paths="safe")

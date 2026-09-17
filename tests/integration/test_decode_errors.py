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
        assert "Array declared length 3 but found 0 items" in str(exc)

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


class TestNestingLimit:
    """Deeply nested input is rejected, not crashed: the parser recurses per
    level, so the documented depth limit of Section 15 keeps it off the
    stack limit."""

    @pytest.mark.parametrize(
        "toon_str",
        [
            pytest.param(
                "\n".join("  " * i + f"k{i}:" for i in range(1500)),
                id="objects",
            ),
            pytest.param(
                "a[1]:\n"
                + "".join("  " * (i + 1) + "- [1]:\n" for i in range(1500)),
                id="arrays",
            ),
            pytest.param(
                "a[1]{" + "g{" * 1500 + "x" + "}" * 1500 + "}:\n  1",
                id="field-groups",
            ),
        ],
    )
    def test_nesting_beyond_the_limit_raises(self, toon_str):
        """Nesting deeper than 1000 containers raises ToonDecodeError."""
        with pytest.raises(
            toons.ToonDecodeError, match="Maximum nesting depth"
        ):
            toons.loads(toon_str)


class TestDuplicateKeyErrorClass:
    """Duplicate sibling keys are a strict-mode decode error (Section 14.3),
    resolved last-write-wins when strict is off."""

    def test_duplicate_key_raises_toon_decode_error(self):
        """Two sibling fields with the same key are rejected in strict mode."""
        with pytest.raises(toons.ToonDecodeError):
            toons.loads("name: Ada\nname: Bob\n")

    def test_duplicate_key_last_write_wins(self):
        """Non-strict mode keeps the last value, silently."""
        assert toons.loads("name: Ada\nname: Bob\n", strict=False) == {
            "name": "Bob"
        }

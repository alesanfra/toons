from textwrap import dedent

import pytest

import toons


def test_strict_flag_exposed():
    """Test that the strict flag is exposed in loads and load."""

    # Case 1: Strict mode (default) - should fail on blank lines in arrays
    toon_with_blank_lines = dedent("""
    [3]:
      - 1

      - 2
      - 3
    """).strip()

    with pytest.raises(ValueError, match="Blank line inside array"):
        toons.loads(toon_with_blank_lines)

    with pytest.raises(ValueError, match="Blank line inside array"):
        toons.loads(toon_with_blank_lines, strict=True)

    # Case 2: Non-strict mode - should succeed
    data = toons.loads(toon_with_blank_lines, strict=False)
    assert data == [1, 2, 3]


def test_strict_flag_load(tmp_path):
    """Test that the strict flag is exposed in load."""
    toon_file = tmp_path / "test.toon"
    toon_with_blank_lines = dedent("""
    [3]:
      - 1

      - 2
      - 3
    """).strip()

    toon_file.write_text(toon_with_blank_lines)

    # Strict mode (default)
    with open(toon_file, "r") as f:
        with pytest.raises(ValueError, match="Blank line inside array"):
            toons.load(f)

    # Non-strict mode
    with open(toon_file, "r") as f:
        data = toons.load(f, strict=False)
        assert data == [1, 2, 3]


def test_strict_flag_indentation():
    """Test that strict=False allows indentation mismatches."""
    # Mixed indentation: first indented line sets indent_size=2
    # Second indented line has 3 spaces -> error in strict mode
    toon_bad_indent = dedent("""
    root:
      child1: value
       child2: value
    """).strip()

    # Strict mode (default) - should fail
    with pytest.raises(
        ValueError, match="Indentation .* is not a multiple of indent size"
    ):
        toons.loads(toon_bad_indent)

    # Non-strict mode - should succeed
    data = toons.loads(toon_bad_indent, strict=False)
    assert data["root"]["child1"] == "value"
    assert data["root"]["child2"] == "value"

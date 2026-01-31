"""
Integration tests using official TOON specification fixtures.

This test suite validates the implementation against the comprehensive
language-agnostic JSON test fixtures defined in the TOON specification.
It covers all specification requirements for both encoding and decoding.
"""

import json
from pathlib import Path
from typing import Any

import pytest

import toons

# Fixture directories
FIXTURES_DIR = Path(__file__).parent / "fixtures"
ENCODE_FIXTURES_DIR = FIXTURES_DIR / "encode"
DECODE_FIXTURES_DIR = FIXTURES_DIR / "decode"


def load_fixture_file(fixture_path: Path) -> dict[str, Any]:
    """Load a JSON fixture file."""
    with open(fixture_path, "r") as f:
        return json.load(f)


def camel_to_snake(name: str) -> str:
    """Convert camelCase to snake_case."""
    result = []
    for i, char in enumerate(name):
        if char.isupper() and i > 0:
            result.append("_")
            result.append(char.lower())
        else:
            result.append(char.lower())
    return "".join(result)


def convert_options_to_snake_case(options: dict[str, Any]) -> dict[str, Any]:
    """Convert option keys from camelCase to snake_case for Python functions."""
    return {camel_to_snake(key): value for key, value in options.items()}


def collect_encode_fixtures() -> list[tuple]:
    """Collect all encode test cases from fixture files."""
    test_cases = []

    for fixture_file in sorted(ENCODE_FIXTURES_DIR.glob("*.json")):
        fixture = load_fixture_file(fixture_file)
        fixture_name = fixture_file.stem

        for test in fixture["tests"]:
            test_id = f"{fixture_name}::{test['name']}"
            test_cases.append(
                (
                    test_id,
                    test["input"],
                    test["expected"],
                    test.get("options", {}),
                    test.get("shouldError", False),
                    test.get("note", ""),
                )
            )

    return test_cases


def collect_decode_fixtures() -> list[tuple]:
    """Collect all decode test cases from fixture files."""
    test_cases = []

    for fixture_file in sorted(DECODE_FIXTURES_DIR.glob("*.json")):
        fixture = load_fixture_file(fixture_file)
        fixture_name = fixture_file.stem

        for test in fixture["tests"]:
            test_id = f"{fixture_name}::{test['name']}"
            test_cases.append(
                (
                    test_id,
                    test["input"],
                    test["expected"],
                    test.get("options", {}),
                    test.get("shouldError", False),
                    test.get("note", ""),
                )
            )

    return test_cases


@pytest.mark.parametrize(
    "test_id,input_data,expected,options,should_error,note",
    collect_encode_fixtures(),
    ids=lambda x: x if isinstance(x, str) else "",
)
def test_integration_encode(
    test_id: str,
    input_data: Any,
    expected: str,
    options: dict[str, Any],
    should_error: bool,
    note: str,
):
    """Test encoding from JSON to TOON format."""
    # Convert camelCase options to snake_case for Python functions
    python_options = convert_options_to_snake_case(options)

    if should_error:
        # Test expects an error to be raised
        with pytest.raises(Exception):
            toons.dumps(input_data, **python_options)
    else:
        # Test expects successful encoding
        result = toons.dumps(input_data, **python_options)
        assert result == expected, f"Failed: {test_id}\nNote: {note}"


@pytest.mark.parametrize(
    "test_id,input_toon,expected,options,should_error,note",
    collect_decode_fixtures(),
    ids=lambda x: x if isinstance(x, str) else "",
)
def test_integration_decode(
    test_id: str,
    input_toon: str,
    expected: Any,
    options: dict[str, Any],
    should_error: bool,
    note: str,
):
    """Test decoding from TOON to JSON format."""
    # Convert camelCase options to snake_case for Python functions
    python_options = convert_options_to_snake_case(options)

    if should_error:
        # Test expects an error to be raised
        with pytest.raises(Exception):
            toons.loads(input_toon, **python_options)
    else:
        # Test expects successful decoding
        result = toons.loads(input_toon, **python_options)
        assert result == expected, f"Failed: {test_id}\nNote: {note}"

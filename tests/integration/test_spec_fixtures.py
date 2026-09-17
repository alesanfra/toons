"""Run the official TOON specification fixtures.

The fixtures are language-agnostic JSON files copied from
https://github.com/toon-format/spec/tree/main/tests and cover encoding and
decoding for every section of the specification. Each case is run through
all four entry points: dumps, dump, loads, and load.
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
                    convert_options_to_snake_case(test.get("options", {})),
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
                    convert_options_to_snake_case(test.get("options", {})),
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
def test_integration_dumps(
    test_id: str,
    input_data: Any,
    expected: str,
    options: dict[str, Any],
    should_error: bool,
    note: str,
):
    """Encode a fixture case with dumps()."""

    if should_error:
        with pytest.raises(Exception):
            toons.dumps(input_data, **options)
    else:
        result = toons.dumps(input_data, **options)
        assert result == expected, f"Failed: {test_id}\nNote: {note}"


@pytest.mark.parametrize(
    "test_id,input_data,expected,options,should_error,note",
    collect_encode_fixtures(),
    ids=lambda x: x if isinstance(x, str) else "",
)
def test_integration_dump(
    test_id: str,
    input_data: Any,
    expected: str,
    options: dict[str, Any],
    should_error: bool,
    note: str,
    tmp_path: Path,
):
    """Encode a fixture case with dump()."""
    with open(tmp_path / "temp.toon", "w+t") as f:
        if should_error:
            with pytest.raises(Exception):
                toons.dump(input_data, f, **options)
        else:
            toons.dump(input_data, f, **options)
            f.seek(0)
            result = f.read()
            assert result == expected, f"Failed: {test_id}\nNote: {note}"


@pytest.mark.parametrize(
    "test_id,input_toon,expected,options,should_error,note",
    collect_decode_fixtures(),
    ids=lambda x: x if isinstance(x, str) else "",
)
def test_integration_loads(
    test_id: str,
    input_toon: str,
    expected: Any,
    options: dict[str, Any],
    should_error: bool,
    note: str,
):
    """Decode a fixture case with loads()."""

    if should_error:
        with pytest.raises(Exception):
            toons.loads(input_toon, **options)
    else:
        result = toons.loads(input_toon, **options)
        assert result == expected, f"Failed: {test_id}\nNote: {note}"


@pytest.mark.parametrize(
    "test_id,input_toon,expected,options,should_error,note",
    collect_decode_fixtures(),
    ids=lambda x: x if isinstance(x, str) else "",
)
def test_integration_load(
    test_id: str,
    input_toon: str,
    expected: Any,
    options: dict[str, Any],
    should_error: bool,
    note: str,
    tmp_path: Path,
):
    """Decode a fixture case with load()."""

    with open(tmp_path / "temp.toon", "w+t") as f:
        f.write(input_toon)
        f.seek(0)

        if should_error:
            with pytest.raises(Exception):
                toons.load(f, **options)
        else:
            result = toons.load(f, **options)
            assert result == expected, f"Failed: {test_id}\nNote: {note}"

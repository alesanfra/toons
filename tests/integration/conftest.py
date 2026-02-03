import json
import pathlib

import pytest

FIXTURES_DIR = pathlib.Path(__file__).parent / "fixtures"
ENCODE_FIXTURES_DIR = FIXTURES_DIR / "encode"
DECODE_FIXTURES_DIR = FIXTURES_DIR / "decode"


@pytest.fixture(scope="module")
def encode_fixtures():
    """Fixture to provide encode test cases."""
    test_cases = []

    for fixture_file in sorted(ENCODE_FIXTURES_DIR.glob("*.json")):
        with open(fixture_file, "r") as f:
            fixture = json.load(f)

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


@pytest.fixture(scope="module")
def decode_fixtures():
    """Fixture to provide decode test cases."""
    test_cases = []

    for fixture_file in sorted(DECODE_FIXTURES_DIR.glob("*.json")):
        with open(fixture_file, "r") as f:
            fixture = json.load(f)

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

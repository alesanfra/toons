"""Regression test for complex data structures.

This test validates that TOON can handle a comprehensive set of data types
and organizational patterns (tabular and free-form) with perfect round-trip fidelity.

Test data includes:
- All primitive types (int, float, string, bool, null)
- Collections (arrays, objects, empty containers)
- Nested structures (deep nesting, arrays of objects)
- Tabular data (uniform arrays of objects)
- Edge cases (Unicode, special characters, scientific notation)
- Numeric edge cases (very large/small numbers, negative zero)
"""

import json
import pytest
import toons

import io


@pytest.fixture
def json_data(data_dir):
    """Load complex test data from JSON file"""
    json_path = data_dir / "complex_test.json"
    with open(json_path, "r", encoding="utf-8") as f:
        return json.load(f)


@pytest.fixture
def expected_toon(data_dir):
    """Load expected TOON serialization"""
    toon_path = data_dir / "complex_test.toon"
    with open(toon_path, "r", encoding="utf-8") as f:
        return f.read().strip()


class TestComplexRegression:
    """Test round-trip conversion for complex data structures"""

    def test_loads_from_toon_equals_json(self, json_data, expected_toon):
        """Loading TOON file produces the same data as JSON"""
        toon_data = toons.loads(expected_toon)
        assert toon_data == json_data

    def test_dumps_to_toon_equals_expected(self, json_data, expected_toon):
        """Dumping JSON data produces the expected TOON serialization"""
        toon_output = toons.dumps(json_data)
        assert toon_output == expected_toon

    def test_full_round_trip_preserves_data(self, json_data):
        """Complete round-trip (dict -> TOON -> dict) preserves all data"""
        # Serialize to TOON
        toon_str = toons.dumps(json_data)

        # Deserialize back to dict
        reconstructed = toons.loads(toon_str)

        # Verify exact match
        assert reconstructed == json_data

    def test_dump_equals_dumps(self, json_data, expected_toon):
        """Ensure dump to file matches dumps output"""
        # Serialize using dumps
        toon_str = toons.dumps(json_data)

        # Serialize using dump to a StringIO buffer
        buffer = io.StringIO()
        toons.dump(json_data, buffer)
        buffer.seek(0)
        dumped_str = buffer.read()

        # Verify both methods produce the same TOON string
        assert dumped_str == toon_str == expected_toon

    def test_load_equals_loads(self, expected_toon):
        """Ensure load from file matches loads output"""
        # Deserialize using loads
        toon_data = toons.loads(expected_toon)

        # Deserialize using load from a StringIO buffer
        buffer = io.StringIO(expected_toon)
        loaded_data = toons.load(buffer)

        # Verify both methods produce the same data
        assert loaded_data == toon_data

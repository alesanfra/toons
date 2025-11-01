# TOON Specification v1.3 Test Suite

This test suite verifies compliance with the TOON Specification v1.3 (2025-10-31).

## Test Framework Requirements

**IMPORTANT**:
- ✅ **Use pytest only** - No unittest allowed
- ✅ **Use `@pytest.mark.parametrize`** for parameterized tests
- ✅ **Write readable, human-friendly test cases**
- ✅ **Make assertions on complete output** when testing serialization

## Implementation Details

- **Spec Version**: TOON v1.3
- **Backend**: `rtoon` v0.1.3 (implements TOON Spec v1.2, compatible with v1.3)
- **Test Files**:
  - `test_spec_compliance.py` - General spec compliance (40 tests)
  - `test_dumps.py` - Specific dumps() output testing (50+ tests)

## Test Coverage

### test_spec_compliance.py

#### TestPrimitives (7 tests)
- Null, booleans, integers, floats, strings
- Verifies basic data type encoding/decoding

#### TestObjects (3 tests)
- Simple objects, nested objects, mixed types
- Verifies key-value pair format with 2-space indentation

#### TestArrays (5 tests)
- Primitive arrays with `[N]:` notation
- Tabular format for uniform object arrays `[N]{fields}:`
- Empty arrays

#### TestNestedStructures (3 tests)
- Deeply nested objects
- Objects with arrays
- Complex mixed structures

#### TestRoundTrip (12 tests)
- Round-trip fidelity for all data types
- Type preservation
- Special cases (empty objects)

#### TestFileOperations (2 tests)
- File I/O with `load()` and `dump()`
- TOON file format validation

#### TestSpecCompliance (8 tests)
- Array count notation verification
- Tabular format header verification
- 2-space indentation verification
- Error handling for invalid TOON
- Parser leniency behavior

### test_dumps.py

#### TestDumpsObjects (3 test methods, 15+ cases)
- Flat objects with single/multiple fields
- Nested objects with 2-space indentation
- Deeply nested structures (3+ levels)
- Assertions on complete TOON output

#### TestDumpsLists (3 test methods, 15+ cases)
- Empty arrays
- Single and multiple element arrays
- Primitive type arrays (int, string, bool)
- Mixed type arrays
- Arrays within objects

#### TestDumpsTabular (3 test methods, 10+ cases)
- Uniform object arrays in tabular format
- Header structure `[N]{field1,field2}:`
- Row format with 2-space indentation
- Single and multiple row tables

#### TestDumpsComplexStructures (3 test methods, 5+ cases)
- Objects containing arrays
- Objects with nested tabular arrays
- Multi-level nesting with mixed types

#### TestDumpsPrimitives (1 test method, 10 cases)
- All primitive types at root level
- Positive and negative numbers
- String values

#### TestDumpsEdgeCases (3 test methods, 5+ cases)
- Empty objects
- Zero values
- Negative numbers
- Round-trip verification

## Running Tests

```bash
# Run all tests
pytest tests/unit/

# Run specific test file
pytest tests/unit/test_spec_compliance.py
pytest tests/unit/test_dumps.py

# Run with verbose output
pytest tests/unit/ -v

# Run specific test class
pytest tests/unit/test_dumps.py::TestDumpsObjects

# Run specific test method
pytest tests/unit/test_dumps.py::TestDumpsObjects::test_flat_objects

# Run specific parameterized case (by parameter id)
pytest tests/unit/test_dumps.py::TestDumpsPrimitives::test_primitive_values[42-42]

# Run with output (-s shows print statements)
pytest tests/unit/ -v -s
```

## Writing New Tests

### DO ✅

```python
import pytest
import toons

class TestFeature:
    """Test a specific feature."""

    @pytest.mark.parametrize(
        "input,expected",
        [
            ({"name": "Alice"}, "name: Alice"),
            ({"age": 30}, "age: 30"),
        ],
    )
    def test_something(self, input, expected):
        """Test description."""
        result = toons.dumps(input)
        assert result == expected  # Assert on complete output
```

### DON'T ❌

```python
import unittest  # ❌ No unittest

class TestFeature(unittest.TestCase):  # ❌ Don't use unittest.TestCase
    def test_something(self):
        for case in cases:  # ❌ Use @pytest.mark.parametrize instead
            result = toons.dumps(case)
            assert "something" in result  # ❌ Assert on complete output, not partial
```

## Key Findings

1. **Empty Objects**: Encode as empty string `""`, decode to `None`
2. **Tabular Format**: Automatically used for uniform object arrays
3. **Parser Leniency**: Invalid lines after valid content are ignored
4. **Indentation**: Strict 2-space indentation for nested objects
5. **Array Notation**: All arrays include element count `[N]:`

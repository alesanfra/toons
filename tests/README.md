# Test Conventions for TOONS Project

This document outlines the testing conventions and structure for the TOONS Python library.

## Test Structure

### Directory Organization
```
tests/
├── unit/
│   ├── __init__.py
│   ├── test_loads.py      # Tests for toons.loads() function
│   ├── test_dumps.py      # Tests for toons.dumps() function
│   └── test_roundtrip.py  # Round-trip serialization tests
└── integration/           # Future integration tests
```

### File Naming Conventions
- `test_<function_name>.py` - One file per main function
- Use descriptive test class names: `TestLoads`, `TestDumps`, etc.
- Use descriptive test method names following pattern: `test_<function>_<scenario>`

### Test Implementation Guidelines

#### Use pytest.mark.parametrize for Multiple Test Cases
Instead of writing multiple similar test functions, use parametrization:

```python
@pytest.mark.parametrize("input_data,expected", [
    ("42", 42),
    ("3.14", 3.14),
    ('"hello"', "hello"),
])
def test_loads_primitives(input_data, expected):
    result = toons.loads(input_data)
    assert result == expected
```

#### Test Categories per Function

**For `loads()` function:**
- Primitive types (string, number, boolean, null)
- Collections (arrays, objects)
- TOON-specific features (unquoted keys, tabular format)
- Error handling (malformed input)
- Edge cases (empty containers, special characters)

**For `dumps()` function:**
- Primitive types serialization
- Collections serialization
- Pretty printing options
- Error handling (non-serializable objects)
- Token efficiency verification

**For round-trip tests:**
- Data integrity across serialize/deserialize cycles
- Type preservation
- Complex nested structures

## TOON Specification Compliance

### Test against TOON Spec 1.3
All tests must verify compliance with the official TOON specification v1.3:
https://github.com/johannschopplich/toon/blob/main/SPEC.md

Key areas to test:
1. **Basic Types**: null, boolean, number, string
2. **Collections**: arrays, objects
3. **Tabular Format**: homogeneous object arrays
4. **Syntax Features**: unquoted keys, minimal punctuation
5. **Token Efficiency**: verify compression vs JSON

### Specification Test Data
Create test data that specifically validates:
- All examples from the official spec
- Edge cases mentioned in the spec
- Error conditions defined in the spec

## Test Data Organization

### Use Constants for Test Data
```python
# Good: Define reusable test data
BASIC_OBJECTS = [
    ('{"name": "John"}', {"name": "John"}),
    ('{name: "John"}', {"name": "John"}),  # unquoted key
]

COMPLEX_STRUCTURES = [
    # Define complex test cases here
]
```

### Parametrized Test Patterns
```python
@pytest.mark.parametrize("toon_input,expected_output", BASIC_OBJECTS)
def test_loads_objects(toon_input, expected_output):
    assert toons.loads(toon_input) == expected_output

@pytest.mark.parametrize("python_input", [
    {"simple": "object"},
    [1, 2, 3],
    {"nested": {"deep": ["array"]}},
])
def test_dumps_roundtrip(python_input):
    serialized = toons.dumps(python_input)
    deserialized = toons.loads(serialized)
    assert deserialized == python_input
```

## Error Testing Conventions

### Expected Exceptions
```python
@pytest.mark.parametrize("invalid_input,expected_error", [
    ('{"unclosed": ', ValueError),
    ('[1, 2,', ValueError),
])
def test_loads_invalid_syntax(invalid_input, expected_error):
    with pytest.raises(expected_error):
        toons.loads(invalid_input)
```

## Performance Testing Guidelines

### Token Efficiency Tests
- Compare TOON output length vs JSON equivalent
- Verify compression ratios meet spec expectations (30-60% savings)
- Use parametrized tests for multiple data structures

### Benchmark Structure
```python
@pytest.mark.parametrize("test_data", [
    {"users": [{"name": "Alice"}, {"name": "Bob"}]},
    # More test cases...
])
def test_token_efficiency(test_data):
    import json
    json_str = json.dumps(test_data)
    toon_str = toons.dumps(test_data)

    compression_ratio = len(toon_str) / len(json_str)
    assert compression_ratio < 1.0  # TOON should be more compact
```

## Documentation Requirements

### Test Docstrings
Every test function should have a clear docstring explaining:
- What is being tested
- Expected behavior
- Any special conditions or edge cases

```python
def test_loads_tabular_format(self):
    """Test parsing of TOON tabular format for homogeneous arrays.

    Verifies that arrays of similar objects are correctly parsed
    from the compact tabular representation defined in TOON spec 1.3.
    """
```

### Comments for Complex Test Cases
Use inline comments to explain non-obvious test logic or data:

```python
# Test data from TOON spec section 3.2 - Tabular Arrays
tabular_input = '[2]{name,age}: Alice,25 Bob,30'
```

## Continuous Integration Considerations

### Test Markers
Use pytest markers to categorize tests:
```python
@pytest.mark.unit
@pytest.mark.slow
@pytest.mark.spec_compliance
```

### Coverage Requirements
- Aim for >95% code coverage
- Ensure all error paths are tested
- Test both success and failure scenarios

## Migration Notes

When converting existing tests:
1. Group similar tests into parametrized functions
2. Extract test data into constants
3. Add spec compliance verification
4. Ensure proper file organization
5. Update docstrings to match new conventions

This convention ensures maintainable, comprehensive, and specification-compliant tests for the TOONS library.

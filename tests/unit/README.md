# Unit Tests

Unit tests for TOONS core functionality.

## Test Files

- `test_loads.py` - Tests for `toons.loads()`
- `test_dumps.py` - Tests for `toons.dumps()`
- `test_spec_compliance.py` - TOON Specification v2.0 compliance
- `test_complex_regression.py` - Regression tests
- `test_indent.py` - Indentation handling

## Running Tests

```bash
# Run all unit tests
pytest tests/unit/

# Run specific test file
pytest tests/unit/test_loads.py

# Run with verbose output
pytest tests/unit/ -v
```

## Documentation

See the [Testing Guide](../../docs/testing.md) for complete testing conventions and guidelines.

# Tests

Test suite for the TOONS library.

## Running Tests

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=toons

# Run specific test file
pytest tests/unit/test_loads.py
```

## Documentation

For complete testing guidelines and conventions, see:

- **[Testing Guide](../docs/testing.md)** - Complete testing conventions and best practices

## Test Structure

```
tests/
├── conftest.py              # pytest configuration
├── data/                    # Test data files
│   ├── complex_test.toon
│   └── complex_test.json
└── unit/                    # Unit tests
    ├── test_loads.py
    ├── test_dumps.py
    ├── test_spec_compliance.py
    └── ...
```

## Quick Reference

- Use **pytest** exclusively (no unittest)
- Use `@pytest.mark.parametrize` for multiple test cases
- Write descriptive test names: `test_<function>_<scenario>`
- Assert on complete output when testing serialization

See the [Testing Guide](../docs/testing.md) for detailed conventions.

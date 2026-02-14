# Testing guide

Comprehensive testing conventions for the TOONS project.

## Overview

TOONS uses **pytest** exclusively for testing. All tests must follow the conventions outlined in this guide to ensure consistency and maintainability.

## Test framework

### Requirements

- ✅ **Use pytest only** - No unittest allowed
- ✅ **Use `@pytest.mark.parametrize`** for parameterized tests
- ✅ **Write readable, human-friendly test cases**
- ✅ **Make assertions on complete output** when testing serialization

## Writing tests

### Basic test structure

```python
import pytest
import toons

class TestFeature:
	"""Test a specific feature or function."""

	def test_simple_case(self):
		"""Test description."""
		result = toons.loads("name: Alice")
		assert result == {"name": "Alice"}
```

### Parametrized tests

Use `@pytest.mark.parametrize` for multiple test cases:

```python
import pytest
import toons

class TestDumps:
	"""Test the dumps() function."""

	@pytest.mark.parametrize(
		"input_data,expected",
		[
			({"name": "Alice"}, "name: Alice"),
			({"age": 30}, "age: 30"),
			({"active": True}, "active: true"),
		],
	)
	def test_simple_values(self, input_data, expected):
		"""Test dumps with simple values."""
		result = toons.dumps(input_data)
		assert result == expected
```

### Naming conventions

**Test classes:**
- `TestFunctionName` or `TestFeatureName`
- Example: `TestLoads`, `TestDumpsObjects`

**Test methods:**
- `test_<function>_<scenario>`
- Be descriptive
- Example: `test_loads_nested_objects`, `test_dumps_empty_array`

### Docstrings

Every test should have a clear docstring:

```python
def test_loads_tabular_format(self):
	"""Test parsing of TOON tabular format for homogeneous arrays.

	Verifies that arrays of similar objects are correctly parsed
	from the compact tabular representation defined in TOON spec 1.3.
	"""
	toon_str = """
	users[2]{name,age}:
	  Alice,30
	  Bob,25
	"""
	result = toons.loads(toon_str)
	assert result == {
		"users": [
			{"name": "Alice", "age": 30},
			{"name": "Bob", "age": 25}
		]
	}
```

## Specification compliance tests

TOONS is validated against the [official TOON specification test fixtures](https://github.com/toon-format/spec/tree/main/tests), a comprehensive suite of language-agnostic JSON fixtures maintained alongside the spec itself. The fixtures cover both **encoding** (Python → TOON) and **decoding** (TOON → Python) across every area of the specification: primitives, objects, arrays (inline, tabular, nested, mixed), delimiters, whitespace, key folding, path expansion, root forms, and error handling. Each test case carries the input, the expected output, optional encoder/decoder options, and a reference to the relevant spec section.

Our integration test (`tests/integration/test_spec_fixtures.py`) loads every fixture file, iterates through all test cases, and asserts that TOONS produces the exact expected output — or raises an error when `shouldError` is set. This ensures **100% conformance** with the TOON specification and catches regressions automatically on every CI run.

For the full fixture format, directory layout, and how to contribute new test cases, see the [official spec repository](https://github.com/toon-format/spec).

## Running tests

### Basic commands

```bash
# Run all tests
pytest

# Run tests matching pattern
pytest -k "tabular"

# Run with verbose output
pytest -v

# Run with output capture disabled (see print statements)
pytest -v -s
```

## Best practices

### DO ✅

```python
import pytest
import toons

class TestFeature:
	"""Test a specific feature."""

	@pytest.mark.parametrize(
		"input_data,expected",
		[
			({"name": "Alice"}, "name: Alice"),
			({"age": 30}, "age: 30"),
		],
	)
	def test_something(self, input_data, expected):
		"""Clear docstring explaining the test."""
		result = toons.dumps(input_data)
		assert result == expected  # Assert on complete output
```

### DON'T ❌

```python
import unittest  # ❌ Don't use unittest

class TestFeature(unittest.TestCase):  # ❌ Don't inherit from TestCase
	def test_something(self):
		cases = [...]  # ❌ Don't use loops, use parametrize
		for case in cases:
			result = toons.dumps(case)
			assert "something" in result  # ❌ Don't assert on partial strings
```

## Continuous integration

Tests run automatically on:

- **Pull Requests** - All tests must pass
- **Commits to main** - Verify no regressions
- **Multiple Python versions** - 3.7, 3.8, 3.9, 3.10, 3.11

Ensure your tests:

- Run quickly (< 1 second each when possible)
- Are deterministic (no random failures)
- Clean up resources (files, connections, etc.)
- Don't depend on external services

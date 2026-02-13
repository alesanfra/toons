# Contributing

Thanks for helping improve TOONS.

## Quick steps

1. Fork the repo and create a branch.
2. Set up the dev environment.
3. Make changes with tests.
4. Run checks and open a PR.

## Prerequisites

- Python 3.7+
- Rust (latest stable)
- Git
- maturin

## Setup

```bash
pip install -r requirements-dev.txt
maturin develop
pre-commit install
```

## Build

```bash
# Debug build (fast)
maturin develop

# Release build (optimized)
maturin develop --release
```

## Tests

```bash
pytest
pytest -v
```

## Code quality

```bash
pre-commit run -a
ruff check .
ruff format .
cargo fmt
cargo clippy
```

## PR notes

- Keep changes focused and documented.
- Add tests for new behavior or bug fixes.
- Update docs for user-facing changes.

## Questions?

If you have questions about contributing, feel free to:

- Open a GitHub issue
- Start a GitHub discussion
- Review existing issues and PRs for examples

Thank you for contributing to TOONS! 🎉

## See Also
- [Code of Conduct](https://www.contributor-covenant.org/) - Community guidelines

## Testing guide

Comprehensive testing conventions for the TOONS project.

### Overview

TOONS uses **pytest** exclusively for testing. All tests must follow the conventions outlined in this guide to ensure consistency and maintainability.

### Test framework

#### Requirements

- ✅ **Use pytest only** - No unittest allowed
- ✅ **Use `@pytest.mark.parametrize`** for parameterized tests
- ✅ **Write readable, human-friendly test cases**
- ✅ **Make assertions on complete output** when testing serialization

#### Test structure

```
tests/
├── conftest.py              # pytest configuration
├── data/                    # Test data files
│   ├── complex_test.toon
│   └── complex_test.json
└── unit/                    # Unit tests
	├── test_loads.py        # Tests for loads()
	├── test_dumps.py        # Tests for dumps()
	├── test_spec_compliance.py
	└── test_*.py
```

### Writing tests

#### Basic test structure

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

#### Parametrized tests

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

#### Naming conventions

**Test files:**
- `test_<function_name>.py` - One file per main function
- Example: `test_loads.py`, `test_dumps.py`

**Test classes:**
- `TestFunctionName` or `TestFeatureName`
- Example: `TestLoads`, `TestDumpsObjects`

**Test methods:**
- `test_<function>_<scenario>`
- Be descriptive
- Example: `test_loads_nested_objects`, `test_dumps_empty_array`

#### Docstrings

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

### Test categories

#### Unit tests - loads()

Test the `loads()` function:

```python
class TestLoads:
	"""Test toons.loads() function."""

	@pytest.mark.parametrize(
		"toon_input,expected",
		[
			("name: Alice", {"name": "Alice"}),
			("age: 30", {"age": 30}),
			("active: true", {"active": True}),
			("value: null", {"value": None}),
		],
	)
	def test_primitives(self, toon_input, expected):
		"""Test parsing primitive values."""
		result = toons.loads(toon_input)
		assert result == expected

	def test_nested_objects(self):
		"""Test parsing nested objects."""
		toon_str = """
		user:
		  name: Alice
		  age: 30
		"""
		result = toons.loads(toon_str)
		assert result == {"user": {"name": "Alice", "age": 30}}

	@pytest.mark.parametrize(
		"invalid_input",
		[
			"invalid: [unclosed",
			"missing colon",
			'bad: "unterminated',
		],
	)
	def test_invalid_syntax(self, invalid_input):
		"""Test that invalid TOON raises ValueError."""
		with pytest.raises(ValueError):
			toons.loads(invalid_input)
```

#### Unit tests - dumps()

Test the `dumps()` function:

```python
class TestDumps:
	"""Test toons.dumps() function."""

	@pytest.mark.parametrize(
		"input_data,expected",
		[
			({"name": "Alice"}, "name: Alice"),
			({"tags": ["a", "b"]}, "tags[2]: a,b"),
		],
	)
	def test_serialization(self, input_data, expected):
		"""Test basic serialization."""
		result = toons.dumps(input_data)
		assert result == expected

	def test_tabular_format(self):
		"""Test tabular format for uniform object arrays."""
		data = {
			"users": [
				{"name": "Alice", "age": 30},
				{"name": "Bob", "age": 25}
			]
		}
		result = toons.dumps(data)
		expected = "users[2]{name,age}:\n  Alice,30\n  Bob,25"
		assert result == expected
```

#### Round-trip tests

Verify data survives serialization cycles:

```python
class TestRoundTrip:
	"""Test round-trip serialization."""

	@pytest.mark.parametrize(
		"original",
		[
			{"name": "Alice", "age": 30},
			{"tags": ["python", "rust"]},
			{"nested": {"key": "value"}},
			[1, 2, 3, 4, 5],
		],
	)
	def test_roundtrip(self, original):
		"""Test data survives dumps -> loads cycle."""
		toon_str = toons.dumps(original)
		parsed = toons.loads(toon_str)
		assert parsed == original
```

#### Specification compliance tests

Verify TOON Spec v3.0 compliance:

```python
class TestSpecCompliance:
	"""Test TOON Specification v3.0 compliance."""

	def test_array_count_notation(self):
		"""Test that arrays include element count [N]."""
		data = {"tags": ["a", "b", "c"]}
		result = toons.dumps(data)
		assert "[3]:" in result

	def test_tabular_format_header(self):
		"""Test tabular format header structure."""
		data = {
			"users": [
				{"name": "Alice", "age": 30},
				{"name": "Bob", "age": 25}
			]
		}
		result = toons.dumps(data)
		assert "users[2]{name,age}:" in result

	def test_two_space_indentation(self):
		"""Test that nested objects use 2-space indentation."""
		data = {"user": {"name": "Alice"}}
		result = toons.dumps(data)
		lines = result.split("\n")
		assert lines[1].startswith("  ")  # 2 spaces
		assert not lines[1].startswith("   ")  # Not 3 spaces
```

### Test data organization

#### Using constants

```python
BASIC_OBJECTS = [
	({"name": "Alice"}, "name: Alice"),
	({"age": 30}, "age: 30"),
]

COMPLEX_STRUCTURES = [
	{
		"users": [
			{"name": "Alice", "age": 30},
			{"name": "Bob", "age": 25}
		]
	},
	# More complex test cases...
]

class TestDumps:
	@pytest.mark.parametrize("input_data,expected", BASIC_OBJECTS)
	def test_basic_objects(self, input_data, expected):
		"""Test basic object serialization."""
		result = toons.dumps(input_data)
		assert result == expected
```

#### Using fixtures

```python
import pytest

@pytest.fixture
def sample_user():
	"""Provide sample user data."""
	return {"name": "Alice", "age": 30, "role": "admin"}

@pytest.fixture
def sample_users():
	"""Provide sample user array."""
	return [
		{"name": "Alice", "age": 30},
		{"name": "Bob", "age": 25}
	]

class TestFeature:
	def test_with_fixture(self, sample_user):
		"""Test using fixture data."""
		result = toons.dumps(sample_user)
		assert "name: Alice" in result
```

#### Using test files

```python
import pytest
from pathlib import Path

@pytest.fixture
def test_data_dir():
	"""Get test data directory."""
	return Path(__file__).parent.parent / "data"

class TestFileOperations:
	def test_load_file(self, test_data_dir):
		"""Test loading from test data file."""
		file_path = test_data_dir / "complex_test.toon"
		with open(file_path, "r") as f:
			result = toons.load(f)
		assert isinstance(result, dict)
```

### Error testing

#### Expected exceptions

```python
import pytest

class TestErrors:
	"""Test error handling."""

	@pytest.mark.parametrize(
		"invalid_input",
		[
			"invalid: [unclosed",
			"missing colon value",
			'bad: "unterminated',
		],
	)
	def test_parse_errors(self, invalid_input):
		"""Test that invalid TOON raises ValueError."""
		with pytest.raises(ValueError):
			toons.loads(invalid_input)

	def test_error_message(self):
		"""Test that error messages are descriptive."""
		with pytest.raises(ValueError) as exc_info:
			toons.loads("invalid: [syntax")

		error_msg = str(exc_info.value)
		assert len(error_msg) > 0  # Has error message
```

### Running tests

#### Basic commands

```bash
# Run all tests
pytest

# Run specific file
pytest tests/unit/test_loads.py

# Run specific class
pytest tests/unit/test_dumps.py::TestDumpsObjects

# Run specific test
pytest tests/unit/test_loads.py::TestLoads::test_primitives

# Run tests matching pattern
pytest -k "tabular"

# Run with verbose output
pytest -v

# Run with output capture disabled (see print statements)
pytest -v -s
```

#### Coverage

```bash
# Run with coverage
pytest --cov=toons

# Generate HTML report
pytest --cov=toons --cov-report=html

# View coverage report
open htmlcov/index.html

# Show missing lines
pytest --cov=toons --cov-report=term-missing
```

#### Markers

Use markers to categorize tests:

```python
import pytest

@pytest.mark.slow
def test_large_dataset():
	"""Test with large dataset (slow)."""
	pass

@pytest.mark.spec_compliance
def test_spec_feature():
	"""Test TOON spec compliance."""
	pass
```

Run specific markers:

```bash
# Run only slow tests
pytest -m slow

# Run everything except slow tests
pytest -m "not slow"

# Run spec compliance tests
pytest -m spec_compliance
```

### Best practices

#### DO ✅

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

#### DON'T ❌

```python
import unittest  # ❌ Don't use unittest

class TestFeature(unittest.TestCase):  # ❌ Don't inherit from TestCase
	def test_something(self):
		cases = [...]  # ❌ Don't use loops, use parametrize
		for case in cases:
			result = toons.dumps(case)
			assert "something" in result  # ❌ Don't assert on partial strings
```

### Continuous integration

Tests run automatically on:

- **Pull Requests** - All tests must pass
- **Commits to main** - Verify no regressions
- **Multiple Python versions** - 3.7, 3.8, 3.9, 3.10, 3.11

Ensure your tests:

- Run quickly (< 1 second each when possible)
- Are deterministic (no random failures)
- Clean up resources (files, connections, etc.)
- Don't depend on external services

### Coverage requirements

- **Overall coverage**: > 95%
- **New code**: 100% coverage
- **Critical paths**: 100% coverage
- **Error handling**: All error paths tested

### Performance testing

For performance-sensitive code:

```python
import time
import pytest

def test_performance():
	"""Test that dumps() is fast enough."""
	data = {"users": [{"name": f"User{i}", "age": i} for i in range(100)]}

	start = time.time()
	for _ in range(100):
		toons.dumps(data)
	elapsed = time.time() - start

	# Should complete 100 iterations in < 1 second
	assert elapsed < 1.0, f"Too slow: {elapsed:.2f}s"
```

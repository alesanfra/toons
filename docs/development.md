# Development Guide

Guide for developers who want to contribute to TOONS or build from source.

## Development Setup

### Prerequisites

Before you begin, ensure you have:

- **Python 3.7+** - [Download Python](https://www.python.org/downloads/)
- **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
- **maturin** - Rust/Python build tool
- **Git** - Version control

### Clone and Setup

```bash
# Clone the repository
git clone https://github.com/alesanfra/toons.git
cd toons

# Install development dependencies
pip install -r requirements-dev.txt

# Build the Rust extension in development mode
maturin develop

# Verify installation
python -c "import toons; print('✓ TOONS installed')"
```

### Development Dependencies

The `requirements-dev.txt` includes:

- **pytest** - Testing framework
- **pytest-cov** - Coverage reporting
- **maturin** - Build tool for Rust extensions
- **pre-commit** - Git hooks for code quality
- **ruff** - Fast Python linter and formatter

## Project Structure

```
toons/
├── src/                    # Rust source code
│   ├── lib.rs             # PyO3 bindings
│   └── toon.rs            # TOON implementation wrapper
├── tests/                 # Python tests
│   ├── unit/              # Unit tests
│   │   ├── test_loads.py
│   │   ├── test_dumps.py
│   │   ├── test_spec_compliance.py
│   │   └── ...
│   └── conftest.py        # pytest configuration
├── examples/              # Usage examples
│   ├── string_example.py
│   └── file_example.py
├── docs/                  # Documentation (MkDocs)
├── Cargo.toml            # Rust dependencies
├── pyproject.toml        # Python project config
├── requirements-dev.txt  # Dev dependencies
└── README.md
```

## Building

### Development Build

For iterative development with fast compilation:

```bash
# Build in debug mode (faster compilation)
maturin develop

# Build with release optimizations (slower compilation, faster runtime)
maturin develop --release
```

### Production Build

To build distributable wheels:

```bash
# Build wheel for current platform
maturin build --release

# Wheels are created in target/wheels/
ls target/wheels/
# toons-0.1.2-cp37-abi3-macosx_11_0_arm64.whl
```

### Cross-Platform Builds

```bash
# Build for specific Python version
maturin build --release --target x86_64-unknown-linux-gnu

# Build for multiple Python versions
maturin build --release --interpreter python3.7 python3.8 python3.9 python3.10 python3.11
```

## Testing

### Running Tests

```bash
# Run all tests
pytest

# Run with verbose output
pytest -v

# Run specific test file
pytest tests/unit/test_loads.py

# Run specific test class
pytest tests/unit/test_dumps.py::TestDumpsObjects

# Run specific test method
pytest tests/unit/test_loads.py::TestLoads::test_loads_simple_object

# Run tests matching pattern
pytest -k "tabular"
```

### Coverage

```bash
# Run tests with coverage
pytest --cov=toons

# Generate HTML coverage report
pytest --cov=toons --cov-report=html

# Open coverage report
open htmlcov/index.html
```

### Writing Tests

TOONS uses **pytest** exclusively. Follow these conventions:

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
        """Test description."""
        result = toons.dumps(input_data)
        assert result == expected
```

**Guidelines:**

- Use descriptive test names: `test_<function>_<scenario>`
- Use `@pytest.mark.parametrize` for multiple test cases
- Assert on complete output, not partial strings
- Include docstrings explaining what's being tested

See [Testing](testing.md) for detailed testing conventions.

## Code Quality

### Pre-commit Hooks

Install and use pre-commit hooks:

```bash
# Install hooks
pre-commit install

# Run manually on all files
pre-commit run -a

# Run on staged files (automatically runs on git commit)
pre-commit run
```

Hooks include:

- **Ruff** - Linting and formatting
- **Trailing whitespace removal**
- **End-of-file fixing**
- **YAML validation**

### Linting and Formatting

```bash
# Lint with ruff
ruff check .

# Auto-fix issues
ruff check --fix .

# Format code
ruff format .
```

### Rust Code

```bash
# Format Rust code
cargo fmt

# Lint with clippy
cargo clippy

# Run Rust tests
cargo test
```

## Conventional Commits

All commit messages MUST follow [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting, etc.)
- `refactor` - Code refactoring
- `test` - Test additions or changes
- `chore` - Build process or auxiliary tool changes
- `perf` - Performance improvements
- `ci` - CI/CD changes
- `build` - Build system changes

### Examples

```bash
# Feature
git commit -m "feat(parser): add support for pipe delimiter"

# Bug fix
git commit -m "fix(serializer): correct indentation for nested arrays"

# Documentation
git commit -m "docs(readme): update installation instructions"

# Test
git commit -m "test(loads): add test for empty object arrays"

# Breaking change
git commit -m "feat(api): change dumps() signature

BREAKING CHANGE: dumps() now requires indent parameter"
```

## Debugging

### Python Debugging

```python
# Add debug output in tests
def test_something():
    data = {"name": "Alice"}
    result = toons.dumps(data)
    print(f"Result: {result}")  # Use pytest -s to see output
    assert result == "name: Alice"
```

### Rust Debugging

Add debug output in Rust code:

```rust
// src/lib.rs
use pyo3::prelude::*;

#[pyfunction]
fn dumps(obj: &PyAny) -> PyResult<String> {
    eprintln!("dumps called with: {:?}", obj);
    // ... rest of function
}
```

Run with:

```bash
# See Rust debug output
maturin develop && python -c "import toons; toons.dumps({'test': 'value'})"
```

## Contributing Workflow

1. **Fork** the repository on GitHub
2. **Clone** your fork locally
3. **Create a branch** for your feature/fix
4. **Make changes** following code quality guidelines
5. **Write tests** for your changes
6. **Run tests** and ensure they pass
7. **Commit** with conventional commit messages
8. **Push** to your fork
9. **Create a Pull Request** to the main repository

```bash
# Example workflow
git checkout -b feat/new-feature
# ... make changes ...
pytest
pre-commit run -a
git add .
git commit -m "feat(api): add new feature"
git push origin feat/new-feature
# ... create PR on GitHub ...
```

## Release Process

Releases are managed by maintainers:

1. **Update version** in `Cargo.toml` and `pyproject.toml`
2. **Update changelog** with release notes
3. **Create git tag** with version number
4. **Build wheels** for all platforms
5. **Publish to PyPI** using `maturin publish`

```bash
# Build and publish (maintainers only)
maturin build --release
maturin publish
```

## Documentation

### Building Documentation

```bash
# Install MkDocs dependencies
pip install mkdocs mkdocs-material

# Serve documentation locally
mkdocs serve

# Open browser to http://127.0.0.1:8000

# Build static site
mkdocs build

# Output in site/ directory
```

### Writing Documentation

- Documentation is in `docs/` directory
- Use Markdown format
- Follow existing structure and style
- Include code examples with expected output
- Test all code examples

## Performance Profiling

### Python Profiling

```python
import toons
import cProfile
import pstats

def profile_toons():
    data = {
        "users": [
            {"name": f"User{i}", "age": 20 + i}
            for i in range(1000)
        ]
    }

    for _ in range(100):
        toon_str = toons.dumps(data)
        toons.loads(toon_str)

# Profile
cProfile.run('profile_toons()', 'profile_stats')

# Analyze results
p = pstats.Stats('profile_stats')
p.strip_dirs().sort_stats('cumulative').print_stats(10)
```

### Rust Profiling

```bash
# Profile with cargo flamegraph
cargo install flamegraph
sudo cargo flamegraph --test test_name

# Profile with perf (Linux)
cargo build --release
perf record --call-graph dwarf ./target/release/toons
perf report
```

## Troubleshooting

### Build Issues

**Problem:** `maturin develop` fails with compiler errors

**Solution:**
```bash
# Update Rust
rustup update

# Clean build artifacts
cargo clean

# Rebuild
maturin develop
```

**Problem:** Python can't find toons module

**Solution:**
```bash
# Ensure you're in the right virtual environment
which python

# Reinstall
pip uninstall toons
maturin develop
```

### Test Issues

**Problem:** Tests fail with import errors

**Solution:**
```bash
# Install in editable mode
maturin develop

# Verify installation
python -c "import toons; print(toons.__file__)"
```

**Problem:** Tests pass locally but fail in CI

**Solution:**
- Check Python version compatibility
- Ensure all dependencies are in `requirements-dev.txt`
- Run tests with same Python version as CI

## Resources

- **Rust Book**: https://doc.rust-lang.org/book/
- **PyO3 Guide**: https://pyo3.rs/
- **Maturin Docs**: https://www.maturin.rs/
- **pytest Docs**: https://docs.pytest.org/
- **TOON Spec**: https://github.com/johannschopplich/toon

## See Also

- [Contributing Guidelines](contributing.md) - Contribution requirements
- [Testing Guide](testing.md) - Testing conventions
- [API Reference](api-reference.md) - API documentation

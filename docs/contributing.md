# Contributing to TOONS

Thank you for your interest in contributing to TOONS! This guide will help you get started.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## Getting Started

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/toons.git
cd toons
```

### 2. Set Up Development Environment

```bash
# Install development dependencies
pip install -r requirements-dev.txt

# Build the extension in development mode
maturin develop

# Install pre-commit hooks
pre-commit install

# Verify everything works
pytest
```

### 3. Create a Branch

```bash
# Create a feature branch
git checkout -b feat/your-feature-name

# Or a fix branch
git checkout -b fix/issue-description
```

## Development Workflow

### Making Changes

1. **Write your code** following the project's style guidelines
2. **Add tests** for your changes
3. **Update documentation** if needed
4. **Run tests** to ensure everything passes
5. **Run linters** to check code quality

```bash
# Run tests
pytest

# Run linters
pre-commit run -a

# Or manually
ruff check .
ruff format .
```

### Commit Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/) for all commit messages:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

#### Commit Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that don't affect code meaning (formatting, etc.)
- **refactor**: Code change that neither fixes a bug nor adds a feature
- **test**: Adding missing tests or correcting existing tests
- **chore**: Changes to build process or auxiliary tools
- **perf**: Performance improvement
- **ci**: CI/CD configuration changes
- **build**: Changes affecting build system or dependencies

#### Commit Examples

```bash
# Feature
git commit -m "feat(parser): add support for custom delimiters"

# Bug fix
git commit -m "fix(serializer): handle empty objects correctly"

# Documentation
git commit -m "docs(api): add examples for dumps() function"

# Test
git commit -m "test(loads): add edge case for nested arrays"

# Breaking change
git commit -m "feat(api)!: change return type of loads()

BREAKING CHANGE: loads() now returns None for empty input instead of empty dict"
```

#### Scope Examples

- `parser` - Changes to parsing logic
- `serializer` - Changes to serialization logic
- `api` - Changes to public API
- `tests` - Test-related changes
- `docs` - Documentation changes
- `build` - Build system changes

### Running Tests

```bash
# Run all tests
pytest

# Run specific test file
pytest tests/unit/test_loads.py

# Run with coverage
pytest --cov=toons

# Run with verbose output
pytest -v
```

**Important:** All tests must pass before submitting a PR.

### Code Quality

```bash
# Run all pre-commit hooks
pre-commit run -a

# Or run individual tools
ruff check .        # Linting
ruff format .       # Formatting
cargo fmt          # Rust formatting
cargo clippy       # Rust linting
```

## Contribution Guidelines

### Code Style

**Python:**
- Follow PEP 8 style guide
- Use type hints where appropriate
- Maximum line length: 79 characters
- Use descriptive variable names

**Rust:**
- Follow Rust standard style guide
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting

### Testing Requirements

All contributions must include appropriate tests:

1. **Unit tests** for new functions/features
2. **Integration tests** for complex interactions
3. **Regression tests** for bug fixes

**Test Guidelines:**
- Use pytest exclusively (no unittest)
- Use `@pytest.mark.parametrize` for multiple test cases
- Write descriptive test names: `test_<function>_<scenario>`
- Include docstrings explaining what's being tested
- Assert on complete output, not partial strings

Example:

```python
import pytest
import toons

class TestNewFeature:
    """Test the new feature."""

    @pytest.mark.parametrize(
        "input_data,expected",
        [
            ({"key": "value"}, "key: value"),
            ({"number": 42}, "number: 42"),
        ],
    )
    def test_new_feature(self, input_data, expected):
        """Test new feature with various inputs."""
        result = toons.dumps(input_data)
        assert result == expected
```

### Documentation

Update documentation for any user-facing changes:

- **API changes**: Update `docs/api-reference.md`
- **New features**: Add examples to `docs/examples.md`
- **Usage changes**: Update `docs/getting-started.md`
- **README**: Keep the main README concise, link to full docs

### Pull Request Process

1. **Ensure all tests pass**
   ```bash
   pytest
   pre-commit run -a
   ```

2. **Update documentation** if needed

3. **Push your branch**
   ```bash
   git push origin feat/your-feature-name
   ```

4. **Create a Pull Request** on GitHub
   - Use a descriptive title following conventional commits
   - Provide a clear description of changes
   - Reference any related issues
   - Include screenshots for UI changes (if applicable)

5. **Respond to review feedback**
   - Address reviewer comments
   - Make requested changes
   - Push additional commits to your branch

6. **Wait for approval**
   - At least one maintainer approval required
   - All CI checks must pass

### Pull Request Template

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Related Issues
Fixes #<issue_number>

## Testing
- [ ] Tests pass locally
- [ ] New tests added for new functionality
- [ ] Documentation updated

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] No new warnings generated
- [ ] Tests added and passing
- [ ] Conventional commit format used
```

## Types of Contributions

### Bug Reports

Found a bug? Please create an issue with:

- **Clear title** describing the problem
- **Steps to reproduce** the issue
- **Expected behavior** vs actual behavior
- **Code example** demonstrating the bug
- **Environment details** (Python version, OS, TOONS version)

Example:

````markdown
## Bug: loads() fails on empty tabular arrays

### Steps to Reproduce
```python
import toons
data = toons.loads("users[0]{name,age}:")
```

### Expected Behavior
Should return `{'users': []}`

### Actual Behavior
Raises `ValueError: Invalid TOON format`

### Environment
- TOONS version: 0.1.2
- Python version: 3.10.5
- OS: macOS 14.0
````

### Feature Requests

Have an idea? Create an issue with:

- **Clear description** of the proposed feature
- **Use case** explaining why it's needed
- **Example usage** showing how it would work
- **Alternatives considered**

### Documentation Improvements

Documentation contributions are always welcome:

- Fix typos or unclear explanations
- Add more examples
- Improve API documentation
- Add tutorials or guides

### Performance Improvements

For performance-related contributions:

- Include **benchmarks** showing improvement
- Explain **trade-offs** if any
- Ensure **backward compatibility**

## Review Process

### For Contributors

- Be responsive to feedback
- Be patient - reviews take time
- Ask questions if feedback is unclear
- Be open to suggestions

### For Reviewers

- Be respectful and constructive
- Explain reasoning behind suggestions
- Acknowledge good work
- Focus on the code, not the person

## Community

### Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Documentation**: Check the [docs](https://github.com/alesanfra/toons/tree/main/docs) first

### Recognition

Contributors are recognized in:

- GitHub contributors page
- Release notes (for significant contributions)
- Project documentation (for major features)

## License

By contributing to TOONS, you agree that your contributions will be licensed under the same license as the project (Apache License 2.0).

## Questions?

If you have questions about contributing, feel free to:

- Open a GitHub issue
- Start a GitHub discussion
- Review existing issues and PRs for examples

Thank you for contributing to TOONS! 🎉

## See Also

- [Development Guide](development.md) - Development setup and workflow
- [Testing Guide](testing.md) - Testing conventions
- [Code of Conduct](https://www.contributor-covenant.org/) - Community guidelines

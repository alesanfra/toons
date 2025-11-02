# TOONS Documentation

Complete documentation for the TOONS library.

## Building Documentation

### Install Dependencies

```bash
pip install -r requirements-dev.txt
```

### Serve Locally

```bash
# Serve documentation with live reload
mkdocs serve

# Open browser to http://127.0.0.1:8000
```

### Build Static Site

```bash
# Build documentation to site/
mkdocs build

# Output will be in site/ directory
```

## Documentation Structure

```
docs/
├── index.md              # Home page
├── getting-started.md    # Installation and setup
├── quick-start.md        # Quick start guide
├── examples.md           # Practical examples
├── api-reference.md      # Complete API documentation
├── specification.md      # TOON format specification
├── data-types.md         # Type mapping guide
├── development.md        # Development guide
├── contributing.md       # Contributing guidelines
└── testing.md            # Testing conventions
```

## Documentation Pages

### User Documentation

- **[Home](index.md)** - Introduction and overview
- **[Getting Started](getting-started.md)** - Installation and first steps
- **[Quick Start](quick-start.md)** - Get up and running quickly
- **[Examples](examples.md)** - Practical usage examples
- **[API Reference](api-reference.md)** - Complete API documentation
- **[Specification](specification.md)** - TOON format specification v1.3
- **[Data Types](data-types.md)** - Type mapping and conversion

### Developer Documentation

- **[Development](development.md)** - Development setup and workflow
- **[Contributing](contributing.md)** - Contributing guidelines
- **[Testing](testing.md)** - Testing conventions and best practices

## Writing Documentation

### Markdown Guidelines

- Use clear, concise language
- Include code examples with expected output
- Use admonitions for important notes
- Link to related pages

### Code Examples

````markdown
```python
import toons

# Example code
data = toons.loads("name: Alice")
print(data)  # {'name': 'Alice'}
```
````

### Admonitions

```markdown
!!! note "Title"
    This is a note admonition.

!!! warning "Important"
    This is a warning.

!!! tip "Pro Tip"
    This is a helpful tip.
```

## MkDocs Configuration

The site is configured in `mkdocs.yml`:

- **Theme**: Material for MkDocs
- **Features**: Navigation, search, code highlighting
- **Extensions**: Admonitions, code blocks, tables

## Publishing

Documentation can be published using:

```bash
# Build and deploy to GitHub Pages
mkdocs gh-deploy

# Or build manually and deploy to your hosting
mkdocs build
# Upload site/ directory to your host
```

## Contributing to Documentation

When adding or updating documentation:

1. **Test locally** - Run `mkdocs serve` and verify changes
2. **Check links** - Ensure all internal links work
3. **Test code examples** - Verify all code examples run correctly
4. **Follow style** - Match existing documentation style
5. **Update navigation** - Add new pages to `mkdocs.yml` if needed

## Resources

- **MkDocs**: https://www.mkdocs.org/
- **Material for MkDocs**: https://squidfunk.github.io/mkdocs-material/
- **Markdown Guide**: https://www.markdownguide.org/

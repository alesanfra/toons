# Complex examples

Examples that use additional arguments for parsing and serialization.

## Custom delimiters

```python
import toons

data = {"users": [{"id": 1, "name": "A"}, {"id": 2, "name": "B"}]}
print(toons.dumps(data, delimiter="|"))
# users[2|]{id|name}:
#   1|A
#   2|B
```

## Key folding (flatten nested keys)

```python
import toons

payload = {
    "user": {
        "profile": {"name": "Alice", "role": "admin"},
        "prefs": {"theme": "dark"},
    }
}

print(toons.dumps(payload, key_folding="safe"))
# user.profile.name: Alice
# user.profile.role: admin
# user.prefs.theme: dark
```

## Key folding with depth limit

```python
import toons

payload = {
    "a": {"b": {"c": {"d": 1}}}
}

print(toons.dumps(payload, key_folding="on", flatten_depth=2))
# a.b:
#   c:
#     d: 1
```

## Parsing with relaxed rules

```python
import toons

toon_str = """
items[2]:
  - 1

  - 2
"""

data = toons.loads(toon_str, strict=False)
print(data)  # {'items': [1, 2]}
```

## Expanding paths while parsing

```python
import toons

toon_str = "path: ~/data/input.txt"
data = toons.loads(toon_str, expand_paths="safe")
print(data["path"])
```

## Custom indentation for output

```python
import toons

data = {"config": {"host": "localhost", "port": 5432}}
print(toons.dumps(data, indent=4))
# config:
#     host: localhost
#     port: 5432

#   1,Alice,alice@example.com
#   2,Bob,bob@example.com
#   3,Charlie,charlie@example.com

# Compare with JSON (146 characters, ~26 tokens)
import json
json_output = json.dumps({"users": users})
print(json_output)
# {"users":[{"id":1,"name":"Alice","email":"alice@example.com"},...]}
```

### Data Export/Import

```python
import toons

# Export database query results
def export_query_results(results, filename):
    """Export query results to TOON format."""
    data = {"results": results, "count": len(results)}
    with open(filename, "w") as f:
        toons.dump(data, f)

# Import and process
def import_and_process(filename):
    """Import TOON data and process."""
    with open(filename, "r") as f:
        data = toons.load(f)

    print(f"Processing {data['count']} results...")
    for result in data['results']:
        # Process each result
        pass

# Usage
query_results = [
    {"product": "Widget", "sales": 100, "revenue": 999.0},
    {"product": "Gadget", "sales": 150, "revenue": 2175.0}
]
export_query_results(query_results, "sales.toon")
import_and_process("sales.toon")
```

### LLM Prompt Context

```python
import toons

# Prepare data for LLM prompt
def prepare_llm_context(user_data, transaction_history):
    """Prepare compact context for LLM."""
    context = {
        "user": user_data,
        "recent_transactions": transaction_history[-5:]  # Last 5
    }
    return toons.dumps(context)

user = {
    "id": 123,
    "name": "Alice",
    "tier": "premium"
}

transactions = [
    {"date": "2025-01-01", "amount": 99.99, "status": "completed"},
    {"date": "2025-01-05", "amount": 149.50, "status": "completed"},
    {"date": "2025-01-10", "amount": 75.00, "status": "pending"}
]

llm_context = prepare_llm_context(user, transactions)
print("LLM Context (TOON format):")
print(llm_context)
# user:
#   id: 123
#   name: Alice
#   tier: premium
# recent_transactions[3]{date,amount,status}:
#   2025-01-01,99.99,completed
#   2025-01-05,149.5,completed
#   2025-01-10,75.0,pending
```

## Error Handling

### Handling Parse Errors

```python
import toons

def safe_parse_toon(toon_str):
    """Parse TOON with error handling."""
    try:
        return toons.loads(toon_str)
    except ValueError as e:
        print(f"Parse error: {e}")
        return None

# Valid TOON
result = safe_parse_toon("name: Alice\nage: 30")
print(result)  # {'name': 'Alice', 'age': 30}

# Invalid TOON
result = safe_parse_toon("invalid: [syntax")
print(result)  # None (with error message)
```

### Strict Mode

By default, TOONS enforces strict compliance with the v3.0 specification. You can disable this for lenient parsing of slightly malformed data (e.g., blank lines in arrays).

```python
import toons

# Malformed TOON (blank line in array is invalid in strict mode)
toon_str = """
items[2]:
  - 1

  - 2
"""

# Strict mode (default) - raises ValueError
try:
    toons.loads(toon_str)
except ValueError as e:
    print(f"Strict error: {e}")

# Non-strict mode - allows the blank line
data = toons.loads(toon_str, strict=False)
print(data)  # {'items': [1, 2]}
```

### Handling Serialization Errors

```python
import toons

def safe_serialize(obj):
    """Serialize with error handling."""
    try:
        return toons.dumps(obj)
    except ValueError as e:
        print(f"Serialization error: {e}")
        return None

# Valid object
result = safe_serialize({"name": "Alice"})
print(result)  # name: Alice

# Invalid object (functions can't be serialized)
result = safe_serialize({"func": lambda x: x})
print(result)  # None (with error message)
```

## Round-Trip Verification

```python
import toons

def verify_roundtrip(original_data):
    """Verify data survives serialization round-trip."""
    # Serialize
    toon_str = toons.dumps(original_data)

    # Parse back
    parsed_data = toons.loads(toon_str)

    # Compare
    if parsed_data == original_data:
        print("✓ Round-trip successful")
        return True
    else:
        print("✗ Round-trip failed")
        print(f"Original: {original_data}")
        print(f"Parsed:   {parsed_data}")
        return False

# Test with complex data
test_data = {
    "users": [
        {"name": "Alice", "age": 30, "active": True},
        {"name": "Bob", "age": 25, "active": False}
    ],
    "metadata": {
        "version": "1.0",
        "timestamp": "2025-01-01T00:00:00Z"
    },
    "tags": ["production", "verified"]
}

verify_roundtrip(test_data)
```

## Advanced Parameters

### Key Folding (Flattening Nested Objects)

Flatten nested object keys into dot-notation paths:

```python
import toons

data = {
    "config": {
        "database": {
            "host": "localhost",
            "port": 5432
        },
        "api": {
            "debug": True
        }
    }
}

# Without key folding (default)
print(toons.dumps(data))
# config:
#   database:
#     host: localhost
#     port: 5432
#   api:
#     debug: true

# With key folding
print(toons.dumps(data, key_folding="safe"))
# config.database.host: localhost
# config.database.port: 5432
# config.api.debug: true
```

### Flatten Depth (Limit Nesting Depth)

Control maximum nesting depth before flattening:

```python
import toons

data = {
    "app": {
        "settings": {
            "ui": {
                "theme": "dark",
                "language": "en"
            }
        }
    }
}

# Only flatten up to depth 2
result = toons.dumps(data, key_folding="safe", flatten_depth=2)
# app.settings:
#   ui:
#     theme: dark
#     language: en
```

### Path Expansion

Expand environment variables and home directory paths (for deserialization):

```python
import toons
import os

os.environ["DATA_DIR"] = "/var/data"

toon_str = """
paths[2]: ~/documents,$DATA_DIR
home: ~
"""

# Safe expansion (expand ~ and env vars)
data = toons.loads(toon_str, expand_paths="safe")
print(data)
# {
#   'paths': ['/home/user/documents', '/var/data'],
#   'home': '/home/user'
# }

# No expansion (default)
data = toons.loads(toon_str)
print(data)
# {
#   'paths': ['~/documents', '$DATA_DIR'],
#   'home': '~'
# }
```

### Custom Delimiters

Use different delimiters for array and tabular data:

```python
import toons

data = {"items": [1, 2, 3], "users": [{"name": "Alice", "age": 30}]}

# Tab delimiter
print(toons.dumps(data, delimiter="\t"))
# items[3	]: 1	2	3
# users[1{name	age}]:
#   Alice	30

# Pipe delimiter
print(toons.dumps(data, delimiter="|"))
# items[3|]: 1|2|3
# users[1|{name|age}]:
#   Alice|30
```

### Custom Indentation

Control indentation levels:

```python
import toons

data = {"nested": {"deeply": {"value": 42}}}

# 4-space indentation
print(toons.dumps(data, indent=4))
# nested:
#     deeply:
#         value: 42

# 1-space indentation
print(toons.dumps(data, indent=1))
# nested:
#  deeply:
#   value: 42
```

### Strict Mode

Enforce strict TOON v3.0 compliance during parsing:

```python
import toons

# Valid TOON but lax (blank line in array)
toon_str = """
items[2]:
  - 1

  - 2
"""

# Strict mode (default) - raises error
try:
    data = toons.loads(toon_str)
except ValueError as e:
    print(f"Strict mode error: {e}")

# Non-strict mode - allows leniency
data = toons.loads(toon_str, strict=False)
print(data)  # {'items': [1, 2]}
```

## Performance Comparison

```python
import toons
import json
import time

def compare_formats(data, iterations=1000):
    """Compare TOON vs JSON performance and size."""

    # JSON
    json_start = time.time()
    for _ in range(iterations):
        json_str = json.dumps(data)
        json.loads(json_str)
    json_time = time.time() - json_start
    json_size = len(json_str)

    # TOON
    toon_start = time.time()
    for _ in range(iterations):
        toon_str = toons.dumps(data)
        toons.loads(toon_str)
    toon_time = time.time() - toon_start
    toon_size = len(toon_str)

    print(f"JSON: {json_size} chars, {json_time:.3f}s")
    print(f"TOON: {toon_size} chars, {toon_time:.3f}s")
    print(f"Size reduction: {(1 - toon_size/json_size) * 100:.1f}%")

# Test with tabular data
users = [
    {"name": f"User{i}", "age": 20 + i, "active": True}
    for i in range(10)
]
compare_formats({"users": users})
```

## See Also

- [API Reference](api-reference.md) - Complete API documentation
- [Data Types](data-types.md) - Supported data types
- [Development](development.md) - Contributing examples

# Examples

Practical examples demonstrating TOONS usage in various scenarios.

## Basic Examples

### Parsing Simple Objects

```python
import toons

# Single-line TOON
data = toons.loads("name: Alice")
print(data)  # {'name': 'Alice'}

# Multi-line object
toon_str = """
name: Alice
age: 30
active: true
"""
data = toons.loads(toon_str)
print(data)
# {'name': 'Alice', 'age': 30, 'active': True}
```

### Serializing Objects

```python
import toons

# Simple object
user = {"name": "Bob", "age": 25, "active": False}
print(toons.dumps(user))
# Output:
# name: Bob
# age: 25
# active: false
```

## Working with Arrays

### Primitive Arrays

```python
import toons

# Parse array
data = toons.loads("tags[3]: python,rust,javascript")
print(data)  # {'tags': ['python', 'rust', 'javascript']}

# Serialize array
tags = {"tags": ["admin", "developer", "ops"]}
print(toons.dumps(tags))
# Output: tags[3]: admin,developer,ops
```

### Arrays of Objects (Tabular Format)

The tabular format is automatically used for uniform object arrays, providing significant token savings:

```python
import toons

# Serialize uniform objects
users = {
    "users": [
        {"name": "Alice", "age": 30, "role": "admin"},
        {"name": "Bob", "age": 25, "role": "user"},
        {"name": "Charlie", "age": 35, "role": "moderator"}
    ]
}

print(toons.dumps(users))
# Output:
# users[3]{name,age,role}:
#   Alice,30,admin
#   Bob,25,user
#   Charlie,35,moderator

# Parse tabular format
toon_str = """
users[2]{name,age}:
  Alice,30
  Bob,25
"""
data = toons.loads(toon_str)
print(data)
# {'users': [{'name': 'Alice', 'age': 30}, {'name': 'Bob', 'age': 25}]}
```

### Mixed Arrays

When arrays contain different types or non-uniform objects:

```python
import toons

# Mixed array with primitives and objects
data = {
    "items": [
        42,
        "hello",
        {"name": "Alice", "age": 30},
        True
    ]
}

print(toons.dumps(data))
# Output:
# items[4]:
#   - 42
#   - hello
#   - name: Alice
#     age: 30
#   - true
```

## Nested Structures

### Nested Objects

```python
import toons

# Deep nesting
data = {
    "company": {
        "name": "TechCorp",
        "location": {
            "city": "San Francisco",
            "country": "USA",
            "coordinates": {
                "lat": 37.7749,
                "lon": -122.4194
            }
        }
    }
}

print(toons.dumps(data))
# Output:
# company:
#   name: TechCorp
#   location:
#     city: San Francisco
#     country: USA
#     coordinates:
#       lat: 37.7749
#       lon: -122.4194
```

### Objects with Arrays

```python
import toons

# Object containing arrays
project = {
    "name": "TOONS",
    "version": "0.1.2",
    "languages": ["Python", "Rust"],
    "maintainers": [
        {"name": "Alice", "role": "lead"},
        {"name": "Bob", "role": "contributor"}
    ]
}

print(toons.dumps(project))
# Output:
# name: TOONS
# version: 0.1.2
# languages[2]: Python,Rust
# maintainers[2]{name,role}:
#   Alice,lead
#   Bob,contributor
```

## File Operations

### Reading and Writing Files

```python
import toons

# Write data to file
data = {
    "config": {
        "database": "postgresql",
        "host": "localhost",
        "port": 5432
    }
}

with open("config.toon", "w") as f:
    toons.dump(data, f)
    print("✓ Saved to config.toon")

# Read data from file
with open("config.toon", "r") as f:
    loaded = toons.load(f)
    print("✓ Loaded from config.toon")
    print(loaded)
```

### Configuration Files

TOON is excellent for configuration files due to its readability:

**config.toon:**
```
server:
  host: 0.0.0.0
  port: 8080
  debug: false
database:
  engine: postgresql
  host: localhost
  port: 5432
  credentials:
    user: admin
    password: secret
allowed_ips[3]: 192.168.1.1,192.168.1.2,192.168.1.3
```

**Loading configuration:**
```python
import toons

def load_config(filename):
    with open(filename, "r") as f:
        return toons.load(f)

config = load_config("config.toon")
print(f"Server running on {config['server']['host']}:{config['server']['port']}")
```

## Real-World Use Cases

### API Response Serialization

```python
import toons

# Serialize API response for LLM context
def serialize_users_for_llm(users):
    """Serialize user data in token-efficient TOON format."""
    data = {"users": users}
    return toons.dumps(data)

users = [
    {"id": 1, "name": "Alice", "email": "alice@example.com"},
    {"id": 2, "name": "Bob", "email": "bob@example.com"},
    {"id": 3, "name": "Charlie", "email": "charlie@example.com"}
]

# TOON format (47 characters, ~16 tokens)
toon_output = serialize_users_for_llm(users)
print(toon_output)
# users[3]{id,name,email}:
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
- [Quick Start](quick-start.md) - Get started quickly
- [Data Types](data-types.md) - Supported data types
- [Development](development.md) - Contributing examples

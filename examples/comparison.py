import difflib
import pprint
import time

import toons

try:
    import toon
except ImportError as e:
    print(e)
    print("Install it with: 'pip install python-toon'")
    exit(1)


DATA = {
    "metadata": {
        "version": "1.0.0",
        "created": "2025-11-02T10:30:00Z",
        "author": None,
        "tags": ["test", "complex", "comprehensive"],
        "verified": True,
        "score": 98.5,
    },
    "primitives": {
        "integer_positive": 42,
        "integer_negative": -17,
        "integer_zero": 0,
        "float_positive": 3.14159,
        "float_negative": -99.99,
        "float_scientific": 2.5e-4,
        "float_small": 0.0000001,
        "float_large": 1.5e10,
        "string_simple": "hello",
        "string_empty": "",
        "string_unicode": "Hello 世界 🌍",
        "string_with_quotes": 'She said "hello"',
        "string_with_escapes": "Line1\nLine2\tTabbed",
        "boolean_true": True,
        "boolean_false": False,
        "null_value": None,
    },
    "arrays": {
        "empty": [],
        "integers": [1, 2, 3, 4, 5],
        "floats": [1.1, 2.2, 3.3],
        "strings": ["alpha", "beta", "gamma"],
        "booleans": [True, False, True],
        "mixed": [1, "two", 3.0, True, None],
        "nested": [[1, 2], [3, 4], [5, 6]],
    },
    "objects": {
        "empty": {},
        "simple": {"key1": "value1", "key2": "value2"},
        "nested": {"level1": {"level2": {"level3": "deep"}}},
        "mixed_values": {
            "string": "text",
            "number": 123,
            "bool": True,
            "null": None,
            "array": [1, 2, 3],
            "object": {"nested": "value"},
        },
    },
    "tabular_data": [
        {
            "id": 1,
            "name": "Alice",
            "age": 30,
            "active": True,
            "salary": 75000.50,
        },
        {
            "id": 2,
            "name": "Bob",
            "age": 25,
            "active": False,
            "salary": 65000.0,
        },
        {
            "id": 3,
            "name": "Charlie",
            "age": 35,
            "active": True,
            "salary": 85000.75,
        },
        {
            "id": 4,
            "name": "Diana",
            "age": 28,
            "active": True,
            "salary": 70000.0,
        },
    ],
    "edge_cases": {
        "very_long_string": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
        "special_chars": "!@#$%^&*()_+-={}[]|:;<>?,./",
        "numbers_as_strings": ["123", "45.67", "1e10"],
        "empty_nested": {
            "level1": {"level2": {"empty_array": [], "empty_object": {}}}
        },
        "unicode_variants": {
            "emoji": "🎉🚀✨💻🐍",
            "chinese": "你好世界",
            "japanese": "こんにちは世界",
            "arabic": "مرحبا بالعالم",
            "russian": "Привет мир",
        },
    },
    "nested_arrays_of_objects": [
        {
            "category": "fruits",
            "items": [
                {"name": "apple", "color": "red", "price": 1.2},
                {"name": "banana", "color": "yellow", "price": 0.8},
            ],
        },
        {
            "category": "vegetables",
            "items": [
                {"name": "carrot", "color": "orange", "price": 0.9},
                {"name": "broccoli", "color": "green", "price": 1.5},
            ],
        },
    ],
    "deeply_nested": {
        "a": {"b": {"c": {"d": {"e": {"f": {"value": "deep", "level": 6}}}}}}
    },
    "numeric_edge_cases": {
        "min_int": -2147483648,
        "max_int": 2147483647,
        "very_small": 1e-10,
        "very_large": 1e15,
        "negative_zero": 0,
        "pi": 3.141592653589793,
    },
}

ITERATIONS = 1000

start = time.perf_counter()
for _ in range(ITERATIONS):
    official_result = toon.encode(DATA)
print(f"Official toon.encode: {time.perf_counter() - start:.4f} seconds")

start = time.perf_counter()
for _ in range(ITERATIONS):
    toons_result = toons.dumps(DATA)
print(f"toons.dumps: {time.perf_counter() - start:.4f} seconds")

print(
    "\n".join(
        difflib.ndiff(
            pprint.pformat(toons_result).splitlines(),
            pprint.pformat(official_result).splitlines(),
        )
    )
)

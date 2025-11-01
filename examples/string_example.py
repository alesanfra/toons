"""
Simple example using TOON with strings (loads/dumps)
"""

import toons

# Serialize Python dict to TOON string
data = {"name": "Alice", "age": 30, "tags": ["python", "rust", "toon"]}

toon_string = toons.dumps(data)
print("TOON output:")
print(toon_string)
print()

# Parse TOON string back to Python dict
parsed = toons.loads(toon_string)
print("Parsed data:")
print(parsed)
print()

# Verify round-trip
print(f"Round-trip successful: {data == parsed}")

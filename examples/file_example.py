"""
Simple example using TOON with files (load/dump)
"""

import toons

# Data to save
data = {
    "users": [
        {"name": "Alice", "role": "admin"},
        {"name": "Bob", "role": "user"},
    ],
    "active": True,
}

# Write TOON data to file
with open("data.toon", "w") as f:
    toons.dump(data, f)
    print("✓ Data saved to data.toon")

# Read TOON data from file
with open("data.toon", "r") as f:
    loaded = toons.load(f)
    print("\n✓ Data loaded from data.toon:")
    print(loaded)

print(f"\n✓ Round-trip successful: {data == loaded}")

# Show the TOON file content
print("\nTOON file content:")
print("-" * 40)
with open("data.toon", "r") as f:
    print(f.read())

"""Encode and decode TOON strings with dumps() and loads()."""

import toons

data = {"name": "Alice", "age": 30, "tags": ["python", "rust", "toon"]}

toon_string = toons.dumps(data)
print("TOON output:")
print(toon_string)

parsed = toons.loads(toon_string)
print("\nParsed data:")
print(parsed)

print("\nRound-trip successful:", data == parsed)

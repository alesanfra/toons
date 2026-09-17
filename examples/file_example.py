"""Read and write TOON files with load() and dump()."""

import tempfile
from pathlib import Path

import toons

data = {
    "users": [
        {"name": "Alice", "role": "admin"},
        {"name": "Bob", "role": "user"},
    ],
    "active": True,
}

with tempfile.TemporaryDirectory() as tmp_dir:
    path = Path(tmp_dir) / "data.toon"

    with path.open("w") as f:
        toons.dump(data, f)

    print("File content:")
    print(path.read_text())

    with path.open() as f:
        loaded = toons.load(f)

print("\nLoaded:", loaded)
print("Round-trip successful:", data == loaded)

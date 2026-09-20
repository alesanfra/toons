"""Time toons.dumps against json.dumps on a few payload shapes.

json.dumps is the yardstick because it is a C encoder that walks the same
Python objects once, so the ratio is stable across machines in a way that
absolute milliseconds are not. Run it against a release build:

    uv run maturin develop --uv --release
    uv run --no-sync python benchmarks/encode_vs_json.py
"""

import json
import timeit

import toons

ROWS = [
    {
        "id": i,
        "name": f"user{i}",
        "email": f"user{i}@example.com",
        "score": (i * 7919) % 1000 / 10,
        "active": i % 2 == 0,
    }
    for i in range(2000)
]

NESTED = [
    {
        "file": f"src/module_{i}.py",
        "line": i * 3,
        "rule": "layer-violation",
        "detail": {
            "source": "api",
            "target": "db",
            "note": "text, with comma",
        },
    }
    for i in range(1000)
]

FLAT = {f"key_{i}": f"value number {i}" for i in range(3000)}

KEYED = {
    f"row{i}": {"a": i, "b": f"s{i}", "c": i % 3 == 0} for i in range(2000)
}

PAYLOADS = [
    ("tabular rows", ROWS),
    ("nested objects", NESTED),
    ("flat object", FLAT),
    ("keyed tabular", KEYED),
]


def best_ms(fn, number=20, repeat=7):
    """Best of `repeat` runs of `number` calls, in milliseconds per call."""
    return min(timeit.repeat(fn, number=number, repeat=repeat)) / number * 1e3


def main():
    print(f"toons {toons.__version__}")
    print(f"{'payload':16} {'dumps ms':>9} {'json ms':>8} {'ratio':>6}")
    for label, obj in PAYLOADS:
        toon_ms = best_ms(lambda: toons.dumps(obj))
        json_ms = best_ms(lambda: json.dumps(obj))
        print(
            f"{label:16} {toon_ms:9.3f} {json_ms:8.3f} {toon_ms / json_ms:6.2f}"
        )


if __name__ == "__main__":
    main()

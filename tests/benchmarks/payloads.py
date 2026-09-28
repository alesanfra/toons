"""Payload shapes timed by the encoder benchmarks.

tests/integration/test_benchmark_payloads.py round-trips them in the default
run, so a benchmark never times a document the encoder gets wrong.
"""

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

PAYLOADS = [ROWS, NESTED, FLAT, KEYED]

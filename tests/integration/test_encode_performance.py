"""Encoder throughput relative to the standard library's json encoder.

json.dumps is a C encoder that walks the same Python objects once, so the
ratio between the two is a machine-independent yardstick: both are timed in
the same process moments apart, and the best of several repeats discards
scheduler noise.

The bounds are 1.5x the ratios measured on a release build (macOS arm64,
Python 3.14). The encoder of 0.8.0 built a fresh str per table cell for its
dict lookups, copied every key into a Vec per row, and probed primitive
types through failing extractions; it measured 2.3x, 3.2x, 4.4x, and 2.2x
json.dumps on these payloads and fails every bound below.
"""

import json
import timeit

import pytest

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


def best_seconds(fn, number=20, repeat=7):
    """Best of `repeat` runs of `number` calls, in seconds per call."""
    return min(timeit.repeat(fn, number=number, repeat=repeat)) / number


class TestEncodeThroughput:
    """dumps() stays within a fixed factor of json.dumps() per payload shape."""

    @pytest.mark.parametrize(
        ("payload", "max_ratio"),
        [
            pytest.param(ROWS, 1.0, id="tabular-rows"),
            pytest.param(NESTED, 1.4, id="nested-objects"),
            pytest.param(FLAT, 3.0, id="flat-object"),
            pytest.param(KEYED, 1.5, id="keyed-tabular"),
        ],
    )
    def test_dumps_within_factor_of_json(self, payload, max_ratio):
        """Encoding is at most `max_ratio` times slower than json.dumps."""
        toon_seconds = best_seconds(lambda: toons.dumps(payload))
        json_seconds = best_seconds(lambda: json.dumps(payload))

        ratio = toon_seconds / json_seconds
        assert ratio <= max_ratio, (
            f"toons.dumps took {ratio:.2f}x json.dumps "
            f"({toon_seconds * 1e3:.3f} ms vs {json_seconds * 1e3:.3f} ms)"
        )

    def test_payloads_still_round_trip(self):
        """The timed payloads are real documents, not encoder shortcuts."""
        for payload in (ROWS, NESTED, FLAT, KEYED):
            assert toons.loads(toons.dumps(payload)) == payload

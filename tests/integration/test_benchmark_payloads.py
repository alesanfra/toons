"""The benchmark payloads are real documents, not encoder shortcuts."""

import pytest
from payloads import PAYLOADS

import toons


@pytest.mark.parametrize(
    "payload",
    PAYLOADS,
    ids=["tabular-rows", "nested-objects", "flat-object", "keyed-tabular"],
)
def test_payload_round_trips(payload):
    """Each timed payload decodes back to the object that was encoded."""
    assert toons.loads(toons.dumps(payload)) == payload

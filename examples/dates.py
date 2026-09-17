"""Date and time objects encode as ISO 8601 strings."""

from datetime import datetime

import toons

now = datetime(2025, 2, 7, 14, 30, 45)

data = {
    "values": [
        {"kind": "datetime", "value": now},
        {"kind": "time", "value": now.time()},
        {"kind": "date", "value": now.date()},
        {"kind": "preformatted", "value": now.strftime("%Y-%m-%d")},
    ]
}

print(toons.dumps(data))

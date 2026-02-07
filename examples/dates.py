from datetime import datetime

import toons

now = datetime.now()

d = {
    "my_data": [
        {"my_date": now},
        {"my_date": now.time()},
        {"my_date": now.date()},
        {"my_date": now.isoformat()},
        {"my_date": now.strftime("%Y-%m-%d")},
        {"my_date": now.time().isoformat()},
        {"my_date": now.time().strftime("%H:%M:%S")},
    ]
}

print(toons.dumps(d))

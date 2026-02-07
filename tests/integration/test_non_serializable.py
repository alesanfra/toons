"""
Tests for serialization of non-JSON-serializable Python objects.

Validates that datetime, time, date, and Decimal objects
serialize to their string representations.
"""

import io
from datetime import date, datetime, time
from decimal import Decimal

import pytest

import toons


class TestDateTimeSerialization:
    """Test serialization of datetime objects to strings."""

    def test_datetime_object_serializes_to_string(self):
        """datetime.datetime serializes to ISO format string."""
        data = {"timestamp": datetime(2025, 2, 7, 14, 30, 45)}
        assert 'timestamp: "2025-02-07T14:30:45"' == toons.dumps(data)

    def test_datetime_in_array(self):
        """datetime in array serializes to string."""
        data = {
            "times": [
                datetime(2025, 1, 1, 0, 0, 0),
                datetime(2025, 2, 7, 14, 30, 45),
            ]
        }
        assert (
            'times[2]: "2025-01-01T00:00:00","2025-02-07T14:30:45"'
            == toons.dumps(data)
        )

    def test_datetime_nested_object(self):
        """datetime in nested objects serializes to string."""
        data = {
            "events": [
                {"name": "Event A", "when": datetime(2025, 2, 7, 10, 0, 0)},
                {"name": "Event B", "when": datetime(2025, 2, 7, 15, 30, 0)},
            ]
        }
        assert (
            'events[2]{name,when}:\n  Event A,"2025-02-07T10:00:00"\n  Event B,"2025-02-07T15:30:00"'
            == toons.dumps(data)
        )


class TestDateSerialization:
    """Test serialization of date objects to strings."""

    def test_date_object_serializes_to_string(self):
        """datetime.date serializes to ISO format string."""
        data = {"day": date(2025, 2, 7)}
        assert "day: 2025-02-07" == toons.dumps(data)

    def test_date_in_array(self):
        """date in array serializes to string."""
        data = {"dates": [date(2025, 1, 1), date(2025, 2, 7)]}
        assert "dates[2]: 2025-01-01,2025-02-07" == toons.dumps(data)

    def test_date_in_objects(self):
        """date in object array serializes to string."""
        data = [
            {"name": "Birth", "date": date(2000, 1, 15)},
            {"name": "Event", "date": date(2025, 2, 7)},
        ]
        assert (
            "[2]{name,date}:\n  Birth,2000-01-15\n  Event,2025-02-07"
            == toons.dumps(data)
        )


class TestTimeSerialization:
    """Test serialization of time objects to strings."""

    def test_time_object_serializes_to_string(self):
        """datetime.time serializes to ISO format string."""
        data = {"clock": time(14, 30, 45)}
        assert 'clock: "14:30:45"' == toons.dumps(data)

    def test_time_in_array(self):
        """time in array serializes to string."""
        data = {"business_hours": [time(9, 0, 0), time(17, 30, 0)]}
        assert 'business_hours[2]: "09:00:00","17:30:00"' == toons.dumps(data)

    def test_time_in_objects(self):
        """time in object array serializes to string."""
        data = [
            {"activity": "breakfast", "time": time(8, 0, 0)},
            {"activity": "lunch", "time": time(12, 30, 0)},
        ]
        assert (
            '[2]{activity,time}:\n  breakfast,"08:00:00"\n  lunch,"12:30:00"'
            == toons.dumps(data)
        )


class TestDecimalSerialization:
    """Test serialization of Decimal objects to strings."""

    def test_decimal_object_serializes_to_string(self):
        """Decimal serializes to string representation."""
        data = {"price": Decimal("19.99")}
        assert "price: 19.99" == toons.dumps(data)

    @pytest.mark.xfail(
        reason="Decimal trailing zeros not preserved in serialization"
    )
    def test_decimal_in_array(self):
        """Decimal in array serializes to string."""
        data = {"prices": [Decimal("10.50"), Decimal("25.99")]}
        assert "prices[2]: 10.50,25.99" == toons.dumps(data)

    def test_decimal_precision_preserved(self):
        """Decimal precision is preserved in serialization."""
        data = {"value": Decimal("10.123456789")}
        assert "value: 10.123456789" == toons.dumps(data)

    """Test serialization of mixed non-serializable types."""

    def test_mixed_datetime_and_date(self):
        """Mixed datetime and date objects serialize correctly."""
        data = {
            "events": [
                {
                    "name": "Conference",
                    "start": datetime(2025, 2, 7, 9, 0, 0),
                    "day": date(2025, 2, 7),
                },
                {
                    "name": "Meetup",
                    "start": datetime(2025, 2, 15, 18, 0, 0),
                    "day": date(2025, 2, 15),
                },
            ]
        }
        assert (
            'events[2]{name,start,day}:\n  Conference,"2025-02-07T09:00:00",2025-02-07\n  Meetup,"2025-02-15T18:00:00",2025-02-15'
            == toons.dumps(data)
        )

    def test_mixed_uuid_and_datetime(self):
        """Mixed datetime objects serialize correctly."""
        data = {
            "logs": [
                {
                    "timestamp": datetime(2025, 2, 7, 14, 30, 0),
                },
                {
                    "timestamp": datetime(2025, 2, 7, 15, 0, 0),
                },
            ]
        }
        assert (
            'logs[2]{timestamp}:\n  "2025-02-07T14:30:00"\n  "2025-02-07T15:00:00"'
            == toons.dumps(data)
        )

    @pytest.mark.xfail(
        reason="Decimal trailing zeros not preserved in serialization"
    )
    def test_mixed_decimal_and_date(self):
        """Mixed Decimal and date objects serialize correctly."""
        data = {
            "transactions": [
                {"date": date(2025, 2, 7), "amount": Decimal("100.50")},
                {"date": date(2025, 2, 8), "amount": Decimal("250.99")},
            ]
        }
        assert (
            "transactions[2]{amount,date}:\n  100.50,2025-02-07\n  250.99,2025-02-08"
            == toons.dumps(data)
        )

    def test_all_types_mixed(self):
        """All non-serializable types together serialize correctly."""
        data = {
            "entry": {
                "created": datetime(2025, 2, 7, 10, 30, 45),
                "date_only": date(2025, 2, 7),
                "time_only": time(10, 30, 45),
                "price": Decimal("99.99"),
            }
        }
        assert (
            'entry:\n  created: "2025-02-07T10:30:45"\n  date_only: 2025-02-07\n  time_only: "10:30:45"\n  price: 99.99'
            == toons.dumps(data)
        )


class TestNonSerializableWithDump:
    """Test that dump() also handles non-serializable objects."""

    def test_dump_with_datetime(self):
        """dump() serializes datetime objects."""
        data = {"created": datetime(2025, 2, 7, 14, 30, 0)}
        fp = io.StringIO()
        toons.dump(data, fp)
        assert 'created: "2025-02-07T14:30:00"' == fp.getvalue()

    def test_dump_with_decimal(self):
        """dump() serializes Decimal objects."""
        data = {"price": Decimal("19.99")}
        fp = io.StringIO()
        toons.dump(data, fp)
        assert "price: 19.99" == fp.getvalue()

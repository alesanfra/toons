"""Tests for the TOON v4 forms and rules, beyond the spec fixtures.

The fixtures cover the specification; these cases pin this
implementation's contract for the same features: comment stripping, keyed
tabular form, nested field groups, the decoder number grammar, and the
canonical empty-array form.
"""

import pytest

import toons


class TestComments:
    """Comment lines are removed before any other rule (Section 5.1)."""

    @pytest.mark.parametrize(
        "toon_str,expected",
        [
            ("# note\na: 1", {"a": 1}),
            ("a: 1\n   # outdented, unaligned\nb: 2", {"a": 1, "b": 2}),
            (
                "items[2]{id}:\n  1\n  # gap\n  2",
                {"items": [{"id": 1}, {"id": 2}]},
            ),
            (
                "m[2:]{v}:\n  a: 1\n# gap\n  b: 2",
                {"m": {"a": {"v": 1}, "b": {"v": 2}}},
            ),
            ("# only a comment", {}),
        ],
    )
    def test_comment_lines_are_stripped(self, toon_str, expected):
        """A comment never ends a scope nor counts as a row."""
        assert toons.loads(toon_str) == expected

    def test_hash_is_only_structural_at_line_start(self):
        """A `#` after a colon or mid-value is data."""
        assert toons.loads("note: #x\nmsg: hello # world") == {
            "note": "#x",
            "msg": "hello # world",
        }

    def test_encoder_quotes_hash_leading_strings(self):
        """Encoder output never contains a line that reads as a comment."""
        assert toons.dumps({"note": "#x", "tags": ["#a", "b"]}) == (
            'note: "#x"\ntags[2]: "#a",b'
        )


class TestKeyedTabular:
    """Objects of uniform objects use the keyed tabular form (Section 9.5)."""

    @pytest.mark.parametrize(
        "data,expected_toon",
        [
            (
                {"m": {"a": {"x": 1}, "b": {"x": 2}}},
                "m[2:]{x}:\n  a: 1\n  b: 2",
            ),
            (
                {"a": {"x": 1, "y": 2}, "b": {"x": 3, "y": 4}},
                "[2:]{x,y}:\n  a: 1,2\n  b: 3,4",
            ),
            (
                {"m": {"a": {"x": {"y": 1}}, "b": {"x": {"y": 2}}}},
                "m[2:]{x{y}}:\n  a: 1\n  b: 2",
            ),
        ],
    )
    def test_keyed_roundtrip(self, data, expected_toon):
        """Keyed output is canonical and decodes back to the same object."""
        assert toons.dumps(data) == expected_toon
        assert toons.loads(expected_toon) == data

    @pytest.mark.parametrize(
        "data,expected_toon",
        [
            # A single entry is below the two-entry threshold.
            ({"m": {"only": {"x": 1}}}, "m:\n  only:\n    x: 1"),
            # Differing key sets disqualify the object.
            (
                {"m": {"a": {"x": 1}, "b": {"y": 2}}},
                "m:\n  a:\n    x: 1\n  b:\n    y: 2",
            ),
            # A primitive entry value disqualifies the object.
            ({"m": {"a": {"x": 1}, "b": 2}}, "m:\n  a:\n    x: 1\n  b: 2"),
        ],
    )
    def test_ineligible_objects_stay_nested(self, data, expected_toon):
        """Objects that fail detection keep the plain nested form."""
        assert toons.dumps(data) == expected_toon
        assert toons.loads(expected_toon) == data

    def test_entry_keys_are_sibling_keys(self):
        """Duplicate entry keys are a strict-mode error, LWW otherwise."""
        toon_str = "m[2:]{v}:\n  a: 1\n  a: 2"
        with pytest.raises(toons.ToonDecodeError):
            toons.loads(toon_str)
        assert toons.loads(toon_str, strict=False) == {"m": {"a": {"v": 2}}}


class TestNestedFieldGroups:
    """Uniform nested object columns collapse into field groups (§9.3)."""

    @pytest.mark.parametrize(
        "data,expected_toon",
        [
            (
                {
                    "o": [
                        {"id": 1, "c": {"n": "Ada"}},
                        {"id": 2, "c": {"n": "Bob"}},
                    ]
                },
                "o[2]{id,c{n}}:\n  1,Ada\n  2,Bob",
            ),
            (
                {"o": [{"g": {"p": {"lat": 1.5, "lon": 2.5}}}]},
                "o[1]{g{p{lat,lon}}}:\n  1.5,2.5",
            ),
        ],
    )
    def test_nested_group_roundtrip(self, data, expected_toon):
        """Rows stay flat and decode back into nested objects."""
        assert toons.dumps(data) == expected_toon
        assert toons.loads(expected_toon) == data

    def test_row_width_counts_leaf_fields(self):
        """A row must carry one cell per leaf field, not per field entry."""
        with pytest.raises(toons.ToonDecodeError):
            toons.loads("o[1]{id,c{n,k}}:\n  1,Ada")


class TestNumberGrammar:
    """Only the Section 4 grammar decodes as a number."""

    @pytest.mark.parametrize(
        "token",
        [".5", "1.", "+5", "05", "-007", "Infinity", "NaN", "0x10", "1_000"],
    )
    def test_non_conforming_tokens_are_strings(self, token):
        """Tokens outside the grammar stay strings, in every position."""
        assert toons.loads(f"v: {token}") == {"v": token}
        assert toons.loads(f"v[1]: {token}") == {"v": [token]}

    @pytest.mark.parametrize(
        "token,expected",
        [
            ("42", 42),
            ("-0", 0),
            ("-0.0", 0.0),
            ("1e-6", 1e-6),
            ("-1E+03", -1000.0),
        ],
    )
    def test_conforming_tokens_are_numbers(self, token, expected):
        """Decimal and exponent forms decode as numbers, -0 as zero."""
        decoded = toons.loads(f"v: {token}")["v"]
        assert decoded == expected
        assert repr(decoded) == repr(expected)


class TestDocumentedPolicies:
    """The choices the spec leaves to the implementation (Sections 2, 4, 12)."""

    def test_out_of_range_float_is_rejected_in_strict_mode(self):
        """A magnitude beyond double precision errors instead of inf."""
        with pytest.raises(toons.ToonDecodeError, match="out of range"):
            toons.loads("a: 1e400")
        assert toons.loads("a: 1e400", strict=False)["a"] == float("inf")

    def test_unpaired_surrogate_is_rejected(self):
        """A str with an unpaired surrogate has no TOON representation."""
        with pytest.raises(ValueError, match="unpaired surrogate"):
            toons.dumps({"a": "\ud800"})

    def test_tab_indentation_counts_as_one_level(self):
        """Tabs error in strict mode and count as one level otherwise."""
        with pytest.raises(toons.ToonDecodeError, match="Tabs"):
            toons.loads("a:\n\tb: 1")
        assert toons.loads("a:\n\tb: 1", strict=False) == {"a": {"b": 1}}


class TestEmptyArrays:
    """Empty arrays use the canonical value form (Section 9.1)."""

    def test_encoder_emits_canonical_form(self):
        """Field and root positions use `key: []` and `[]`."""
        assert toons.dumps({"a": []}) == "a: []"
        assert toons.dumps([]) == "[]"

    @pytest.mark.parametrize(
        "toon_str,expected",
        [
            ("a: []", {"a": []}),
            ("a[0]:", {"a": []}),
            ("[]", []),
            ("[0]:", []),
        ],
    )
    def test_decoder_accepts_both_forms(self, toon_str, expected):
        """The legacy header form is still decoded."""
        assert toons.loads(toon_str) == expected

    def test_bare_key_is_an_empty_object(self):
        """`key:` with no children is an object, never an array."""
        assert toons.loads("a:") == {"a": {}}

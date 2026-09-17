"""TOONS: parse and serialize the TOON format."""

from typing import IO, Any, Optional

__version__: str
"""Version of this library."""

__toon_spec__: str
"""Version of the TOON specification implemented by this library."""

class ToonDecodeError(ValueError):
    """Raised by the decoder when input cannot be parsed.

    Subclasses ``ValueError``, so existing ``except ValueError`` handlers
    keep catching parse failures.

    Attributes:
        line: 1-based line number where the error was detected, or ``None``
            when the location is unknown (for example, empty input).
        source: Raw source line, including its original indentation, or
            ``None`` when unknown.

    The message reads ``"TOON parse error at line N: <detail>"`` when a line
    number is available.

    Example:
        >>> try:
        ...     toons.loads("items[3]: a,b")
        ... except toons.ToonDecodeError as exc:
        ...     print(exc.line, exc.source, str(exc))
    """

    line: Optional[int]
    source: Optional[str]

def load(
    fp: IO[str],
    *,
    strict: bool = True,
    expand_paths: Optional[str] = None,
    indent: Optional[int] = None,
) -> Any:
    """Parse TOON read from a text file object.

    Args:
        fp: File-like object with a .read() method.
        strict: Enforce strict TOON v3.0 compliance.
        expand_paths: Expand dotted keys into nested objects: None, "off",
            "safe", or "always".
        indent: Expected spaces per indentation level, or None to detect it
            from the input.

    Returns:
        The parsed Python object.

    Raises:
        ToonDecodeError: If the input is malformed.
        ValueError: If an option value is invalid.
    """
    ...

def loads(
    s: str,
    *,
    strict: bool = True,
    expand_paths: Optional[str] = None,
    indent: Optional[int] = None,
) -> Any:
    """Parse a TOON string.

    Args:
        s: TOON-formatted string.
        strict: Enforce strict TOON v3.0 compliance.
        expand_paths: Expand dotted keys into nested objects: None, "off",
            "safe", or "always".
        indent: Expected spaces per indentation level, or None to detect it
            from the input.

    Returns:
        The parsed Python object.

    Raises:
        ToonDecodeError: If the input is malformed. Subclass of ValueError
            carrying ``.line`` and ``.source``.
        ValueError: If an option value is invalid.
    """
    ...

def to_json(
    s: str,
    *,
    strict: bool = True,
    expand_paths: Optional[str] = None,
    indent: Optional[int] = None,
) -> str:
    """Convert a TOON string to a JSON string.

    Args:
        s: TOON-formatted string.
        strict: Enforce strict TOON v3.0 compliance.
        expand_paths: Expand dotted keys into nested objects: None, "off",
            "safe", or "always".
        indent: Spaces per JSON indentation level, or None for compact JSON.

    Returns:
        JSON-formatted string.

    Raises:
        ToonDecodeError: If the input is malformed.
        ValueError: If an option value is invalid.
    """
    ...

def dump(
    obj: Any,
    fp: IO[str],
    *,
    indent: int = 2,
    delimiter: str = ",",
    key_folding: Optional[str] = None,
    flatten_depth: Optional[int] = None,
) -> None:
    """Serialize an object to TOON and write it to a file object.

    Args:
        obj: Object to serialize.
        fp: File-like object with a .write() method.
        indent: Spaces per indentation level (minimum 2).
        delimiter: Array and tabular delimiter: ",", "\\t", or "|".
        key_folding: Fold single-key object chains into dotted keys: None or
            "off" to disable, "safe" to enable.
        flatten_depth: Maximum number of segments in a folded key.

    Raises:
        TypeError: If a value cannot be encoded, or a key is not a string.
        ValueError: If an option value is invalid, or the object contains a
            reference cycle.
    """
    ...

def dumps(
    obj: Any,
    *,
    indent: int = 2,
    delimiter: str = ",",
    key_folding: Optional[str] = None,
    flatten_depth: Optional[int] = None,
) -> str:
    """Serialize an object to a TOON string.

    Args:
        obj: Object to serialize. dict, list, tuple, str, int, float, bool,
            None, and date/time/datetime objects are supported.
        indent: Spaces per indentation level (minimum 2).
        delimiter: Array and tabular delimiter: ",", "\\t", or "|".
        key_folding: Fold single-key object chains into dotted keys: None or
            "off" to disable, "safe" to enable.
        flatten_depth: Maximum number of segments in a folded key.

    Returns:
        TOON-formatted string.

    Raises:
        TypeError: If a value cannot be encoded, or a key is not a string.
        ValueError: If an option value is invalid, or the object contains a
            reference cycle.
    """
    ...

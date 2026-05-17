"""TOONS Python API for parsing and serializing TOON format."""

from typing import IO, Any, Optional

class ToonDecodeError(ValueError):
    """Exception raised by the TOON decoder when input cannot be parsed.

    Subclasses ``ValueError`` for backward compatibility, so existing
    ``except ValueError`` handlers continue to catch parse failures.

    Attributes:
        line: 1-based line number where the error was detected, or ``None``
            if the location is unknown (e.g. empty input).
        source: The raw source line (including original indentation) where
            the error was detected, or ``None`` if unknown.

    The default message is formatted as ``"Line N: <detail>"`` when a line
    number is available, matching the canonical TypeScript reference
    implementation (``@toon-format/toon``).

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
    """Parse TOON from a text file object.

    Args:
        fp: File-like object with a .read() method.
        strict: Enforce strict TOON v3.0 compliance.
        expand_paths: Path expansion mode: None, "off", "safe", "always".
        indent: Optional indentation hint for parsing.

    Returns:
        The parsed Python object.

    Raises:
        ToonDecodeError: If the input is malformed. Subclass of ValueError.
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
        expand_paths: Path expansion mode: None, "off", "safe", "always".
        indent: Optional indentation hint for parsing.

    Returns:
        The parsed Python object.

    Raises:
        ToonDecodeError: If the input is malformed. Subclass of ValueError;
            carries structured ``.line`` and ``.source`` attributes.
    """
    ...

def to_json(
    s: str,
    *,
    strict: bool = True,
    expand_paths: Optional[str] = None,
    indent: Optional[int] = 2,
) -> str:
    """Convert a TOON string to a JSON string.

    Args:
        s: TOON-formatted string.
        strict: Enforce strict TOON v3.0 compliance.
        expand_paths: Path expansion mode: None, "off", "safe", "always".
        indent: Spaces per JSON indentation level, or None for compact JSON.

    Returns:
        JSON-formatted string.

    Raises:
        ToonDecodeError: If the input is malformed. Subclass of ValueError;
            carries structured ``.line`` and ``.source`` attributes.
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
        obj: Python object to serialize.
        fp: File-like object with a .write() method.
        indent: Spaces per indentation level.
        delimiter: Array/tabular delimiter (",", "\t", or "|").
        key_folding: Flatten nested keys: None, "safe", "on", "always".
        flatten_depth: Maximum depth for key folding.
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
        obj: Python object to serialize.
        indent: Spaces per indentation level.
        delimiter: Array/tabular delimiter (",", "\t", or "|").
        key_folding: Flatten nested keys: None, "safe", "on", "always".
        flatten_depth: Maximum depth for key folding.

    Returns:
        TOON-formatted string.
    """
    ...

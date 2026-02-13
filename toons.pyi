"""TOONS Python API for parsing and serializing TOON format."""

from typing import IO, Any, Optional

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

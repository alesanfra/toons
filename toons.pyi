from typing import IO, Any, Optional

def load(
    fp: IO[str],
    *,
    strict: bool = True,
    expand_paths: Optional[str] = None,
    indent: Optional[int] = None,
) -> Any: ...
def loads(
    s: str,
    *,
    strict: bool = True,
    expand_paths: Optional[str] = None,
    indent: Optional[int] = None,
) -> Any: ...
def dump(
    obj: Any,
    fp: IO[str],
    *,
    indent: int = 2,
    delimiter: str = ",",
    key_folding: Optional[str] = None,
    flatten_depth: Optional[int] = None,
) -> None: ...
def dumps(
    obj: Any,
    *,
    indent: int = 2,
    delimiter: str = ",",
    key_folding: Optional[str] = None,
    flatten_depth: Optional[int] = None,
) -> str: ...

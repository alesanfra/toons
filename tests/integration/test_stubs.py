"""Keep toons.pyi in sync with the compiled module.

The stubs are written by hand because `maturin generate-stubs` drops the
docstrings, the `ToonDecodeError` class, and the file-object types. This test
compares them against the runtime signatures so they cannot drift.
"""

import ast
import inspect
from pathlib import Path

import pytest

import toons

STUB_PATH = Path(__file__).parent.parent.parent / "toons.pyi"
FUNCTIONS = ["load", "loads", "to_json", "dump", "dumps"]


def stub_tree() -> ast.Module:
    return ast.parse(STUB_PATH.read_text())


def stub_functions() -> dict[str, ast.FunctionDef]:
    return {
        node.name: node
        for node in stub_tree().body
        if isinstance(node, ast.FunctionDef)
    }


def stub_signature(node: ast.FunctionDef) -> str:
    """Render a stub signature the way `inspect.signature` renders one."""
    parts = [arg.arg for arg in node.args.args]

    if node.args.kwonlyargs:
        parts.append("*")
        defaults = node.args.kw_defaults
        for arg, default in zip(node.args.kwonlyargs, defaults):
            rendered = ast.literal_eval(default) if default else None
            parts.append(f"{arg.arg}={rendered!r}")

    return f"({', '.join(parts)})"


class TestStubCoverage:
    """Every public name of the module appears in the stub."""

    def test_functions_are_declared(self):
        """The stub declares each public function."""
        assert set(FUNCTIONS) <= set(stub_functions())

    def test_exception_is_declared(self):
        """The stub declares ToonDecodeError with its attributes."""
        classes = {
            node.name: node
            for node in stub_tree().body
            if isinstance(node, ast.ClassDef)
        }
        assert "ToonDecodeError" in classes

        annotated = {
            target.target.id
            for target in classes["ToonDecodeError"].body
            if isinstance(target, ast.AnnAssign)
        }
        assert {"line", "source"} <= annotated

    def test_module_constants_are_declared(self):
        """The stub declares __version__ and __toon_spec__."""
        annotated = {
            node.target.id
            for node in stub_tree().body
            if isinstance(node, ast.AnnAssign)
        }
        assert {"__version__", "__toon_spec__"} <= annotated


class TestStubSignatures:
    """Stub signatures match the compiled module."""

    @pytest.mark.parametrize("name", FUNCTIONS)
    def test_signature_matches_runtime(self, name):
        """Parameter names, order, and defaults agree with the module."""
        runtime = str(inspect.signature(getattr(toons, name)))
        assert stub_signature(stub_functions()[name]) == runtime

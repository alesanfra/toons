import pathlib
import pytest


@pytest.fixture
def test_path() -> pathlib.Path:
    """Path to the current test file"""
    return pathlib.Path(__file__).parent


@pytest.fixture
def data_dir(test_path) -> pathlib.Path:
    """Path to test data directory"""
    return test_path / "data"

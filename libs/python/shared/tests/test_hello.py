"""Hello unit test module."""

from shared.hello import hey


def test_hello():
    """Test the hello function."""
    assert hey() == "Hello python-shared"

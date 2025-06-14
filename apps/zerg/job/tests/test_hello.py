"""Hello unit test module."""

from zerg_job.hello import hello


def test_hello():
    """Test the hello function."""
    assert hello() == "Hello zerg-job!"

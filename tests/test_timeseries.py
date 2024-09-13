import numpy as np
import numpy.testing as npt
import pytest

from two_layer_model.core import TimeAxis


@pytest.fixture()
def timeaxis():
    return TimeAxis.from_values(np.arange(1850.0, 2000.0, 5))


def test_timeaxis_create():
    axis = TimeAxis.from_values(np.asarray([2000.0, 2020.0, 2040.0]))

    npt.assert_allclose(axis.values(), [2000.0, 2020.0, 2040.0])
    npt.assert_allclose(axis.bounds(), [2000.0, 2020.0, 2040.0, 2060.0])
    assert axis.len() == 3

    # This is in rust debug format, but ok for now
    exp = "TimeAxis { bounds: [2000.0, 2020.0, 2040.0, 2060.0], shape=[4], strides=[1], layout=CFcf (0xf), const ndim=1 }"  # noqa: E501
    assert repr(axis) == exp


def test_timeaxis_immutable(timeaxis):
    values = timeaxis.values()

    assert values.flags["OWNDATA"]
    values[0] = 1200

    assert values[0] == 1200.0
    assert timeaxis.values()[0] == 1850.0


def test_timeaxis_create_from_list():
    match = "'list' object cannot be converted to 'PyArray<T, D>'"
    with pytest.raises(TypeError, match=match):
        TimeAxis.from_values([2000.0, 2020.0, 2040.0])


def test_timeseries_at(timeaxis):
    assert timeaxis.at(0) == 1850.0
    with pytest.raises(OverflowError):
        assert timeaxis.at(-1) is None
    assert timeaxis.at(1) == 1855.0
    assert timeaxis.at(10000) is None


def test_timeseries_at_bounds(timeaxis):
    assert timeaxis.at_bounds(0) == (1850.0, 1855.0)
    with pytest.raises(OverflowError):
        assert timeaxis.at_bounds(-1) is None
    assert timeaxis.at_bounds(1) == (1855.0, 1860.0)
    assert timeaxis.at_bounds(10000) is None

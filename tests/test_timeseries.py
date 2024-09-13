import numpy as np
import numpy.testing as npt
import pytest

from two_layer_model.core import TimeAxis


def test_timeaxis_create():
    axis = TimeAxis.from_values(np.asarray([2000.0, 2020.0, 2040.0]))

    npt.assert_allclose(axis.values(), [2000.0, 2020.0, 2040.0])
    npt.assert_allclose(axis.bounds(), [2000.0, 2020.0, 2040.0, 2060.0])


def test_timeaxis_create_from_list():
    match = "'list' object cannot be converted to 'PyArray<T, D>'"
    with pytest.raises(TypeError, match=match):
        TimeAxis.from_values([2000.0, 2020.0, 2040.0])

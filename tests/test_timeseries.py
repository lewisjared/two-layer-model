import numpy as np
import numpy.testing as npt

from two_layer_model.core import TimeAxis


def test_timeaxis_create():
    axis = TimeAxis.from_values(np.asarray([2000.0, 2020.0, 2040.0]))

    npt.assert_allclose(axis.values(), [2000.0, 2020.0, 2040.0])
    npt.assert_allclose(axis.bounds(), [2000.0, 2020.0, 2040.0, 2060.0])

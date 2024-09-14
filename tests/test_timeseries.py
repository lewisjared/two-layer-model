import numpy as np
import numpy.testing as npt
import pytest

from two_layer_model.core import InterpolationStrategy, TimeAxis, Timeseries


@pytest.fixture()
def time_axis():
    return TimeAxis.from_values(np.arange(1850.0, 2000.0, 5))


class TestTimeAxis:
    def test_time_axis_create(self):
        axis = TimeAxis.from_values(np.asarray([2000.0, 2020.0, 2040.0]))

        npt.assert_allclose(axis.values(), [2000.0, 2020.0, 2040.0])
        npt.assert_allclose(axis.bounds(), [2000.0, 2020.0, 2040.0, 2060.0])
        assert axis.len() == 3

        # This is in rust debug format, but ok for now
        exp = "TimeAxis { bounds: [2000.0, 2020.0, 2040.0, 2060.0], shape=[4], strides=[1], layout=CFcf (0xf), const ndim=1 }"  # noqa: E501
        assert repr(axis) == exp

    def test_time_axis_immutable(self, time_axis):
        values = time_axis.values()

        assert values.flags["OWNDATA"]
        values[0] = 1200

        assert values[0] == 1200.0
        assert time_axis.values()[0] == 1850.0

    def test_time_axis_create_from_list(self):
        match = "'list' object cannot be converted to 'PyArray<T, D>'"
        with pytest.raises(TypeError, match=match):
            TimeAxis.from_values([2000.0, 2020.0, 2040.0])

    def test_time_axis_at(self, time_axis):
        assert time_axis.at(0) == 1850.0
        with pytest.raises(OverflowError):
            assert time_axis.at(-1) is None
        assert time_axis.at(1) == 1855.0
        assert time_axis.at(10000) is None

    def test_time_axis_at_bounds(self, time_axis):
        assert time_axis.at_bounds(0) == (1850.0, 1855.0)
        with pytest.raises(OverflowError):
            assert time_axis.at_bounds(-1) is None
        assert time_axis.at_bounds(1) == (1855.0, 1860.0)
        assert time_axis.at_bounds(10000) is None


class TestTimeseries:
    def test_create(self, time_axis):
        ts = Timeseries(
            values=np.sin(np.arange(1850.0, 2000.0, 5) * np.pi / 180.0),
            time_axis=time_axis,
            units="Test",
            interpolation_strategy=InterpolationStrategy.Next,
        )

        ts.values()

    def test_create_invalid(self, time_axis):
        values = np.arange(0.0, 10.0)
        assert len(values) != len(time_axis)

        with pytest.raises(ValueError):
            Timeseries(
                values=values,
                time_axis=time_axis,
                units="Test",
                interpolation_strategy=InterpolationStrategy.Next,
            )

from two_layer_model.core import TimeAxis  # noqa


def test_timeaxis_create():
    TimeAxis.from_values([2000.0, 2020.0, 2040.0])

import numpy as np

from two_layer_model._lib import TwoLayerComponentBuilder
from two_layer_model._lib.core import InterpolationStrategy, Timeseries
from two_layer_model.core import ModelBuilder


def test_model(time_axis):
    component = TwoLayerComponentBuilder.from_parameters(
        dict(
            lambda0=0.0,
            a=0.0,
            efficacy=0.0,
            eta=0.0,
            heat_capacity_deep=0.0,
            heat_capacity_surface=0.0,
        )
    ).build()

    builder = ModelBuilder()
    builder.with_time_axis(time_axis).with_rust_component(component)

    # Doesn't have any ERF data
    # Need pyo3_runtime.PanicException
    # with pytest.raises(Exception):
    #     builder.build()

    erf = Timeseries(
        np.asarray([1.0] * len(time_axis)),
        time_axis,
        "W / m^2",
        InterpolationStrategy.Next,
    )

    model = builder.with_exogenous_variable("Effective Radiative Forcing", erf).build()

    print(model.as_dot())

    model.run()

from two_layer_model._lib import TwoLayerComponentBuilder
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

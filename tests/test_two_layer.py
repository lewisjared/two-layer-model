from two_layer_model import TwoLayerComponentBuilder


def test_create_component():
    component = TwoLayerComponentBuilder.from_parameters(
        dict(
            lambda0=0.3,
            efficacy=31,
            a=12,
            eta=12,
            heat_capacity_deep=12,
            heat_capacity_surface=1,
        )
    ).build()
    res = component.solve(2000, 2010, {"Effective Radiative Forcing": 12})
    assert isinstance(res, dict)

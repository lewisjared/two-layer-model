from two_layer_model import TwoLayerComponent


def test_create_component():
    component = TwoLayerComponent.from_parameters(
        lambda0=0.3,
        efficacy=31,
        a=12,
        eta=12,
        heat_capacity_deep=12,
        heat_capacity_surface=1,
    )
    res = component.solve(2000, 2010, {"Effective Radiative Forcing": 12})
    assert isinstance(res, dict)

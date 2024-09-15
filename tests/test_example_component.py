import pytest

from two_layer_model._lib.core import TestComponent


def test_component_definitions():
    component = TestComponent.from_parameters({"p": 12})

    definitions = component.definitions()
    assert len(definitions) == 2
    assert definitions[0].name == "Emissions|CO2"
    assert definitions[1].name == "Concentrations|CO2"


def test_component_invalid():
    with pytest.raises(ValueError, match="missing field `p`"):
        TestComponent.from_parameters({})

    with pytest.raises(
        ValueError,
        match="unexpected type: 'NoneType' object cannot be converted to 'Mapping'",
    ):
        # noinspection PyTypeChecker
        TestComponent.from_parameters(None)

import pytest

from two_layer_model._lib.core import TestComponent, UserDerivedComponent


class PythonComponent:
    def solve(
        self, time_current: float, time_next: float, input_state: dict[str, float]
    ) -> dict[str, float]:
        return {"output": input_state.get("input") * 3}


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


def test_user_derived():
    py_component = PythonComponent()
    component = UserDerivedComponent(py_component)

    # TODO: resolve later
    component.definitions() == []

    res = component.solve(0, 1, {"input": 35.0})
    res["output"] == 35.0 * 3.0

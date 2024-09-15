import pytest

from two_layer_model._lib.core import TestComponent
from two_layer_model.core import RequirementDefinition, UserDerivedComponent


class PythonComponent:
    def definitions(self) -> list[RequirementDefinition]:
        return []

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


def test_user_derived_create_and_solve():
    py_component = PythonComponent()
    component = UserDerivedComponent(py_component)

    # TODO: resolve later
    assert component.definitions() == []

    res = component.solve(0, 1, {"input": 35.0})
    assert res["output"] == 35.0 * 3.0

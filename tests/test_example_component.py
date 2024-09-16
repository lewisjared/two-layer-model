import pytest

from two_layer_model._lib.core import TestComponentBuilder
from two_layer_model.core import PythonComponent, RequirementDefinition


class ExamplePythonComponent:
    def definitions(self) -> list[RequirementDefinition]:
        return []

    def solve(
        self, time_current: float, time_next: float, input_state: dict[str, float]
    ) -> dict[str, float]:
        return {"output": input_state.get("input") * 3}


def test_component_definitions():
    component = TestComponentBuilder.from_parameters({"p": 12}).build()

    definitions = component.definitions()
    assert len(definitions) == 2
    assert definitions[0].name == "Emissions|CO2"
    assert definitions[1].name == "Concentrations|CO2"


def test_component_invalid():
    with pytest.raises(ValueError, match="missing field `p`"):
        TestComponentBuilder.from_parameters({})

    with pytest.raises(
        ValueError,
        match="unexpected type: 'NoneType' object cannot be converted to 'Mapping'",
    ):
        # noinspection PyTypeChecker
        TestComponentBuilder.from_parameters(None).build()


def test_user_derived_create_and_solve():
    py_component = ExamplePythonComponent()
    component = PythonComponent.build(py_component)

    # TODO: resolve later
    assert component.definitions() == []

    res = component.solve(0, 1, {"input": 35.0})
    assert res["output"] == 35.0 * 3.0

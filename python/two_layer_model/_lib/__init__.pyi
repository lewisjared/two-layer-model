# This .pyi file and the other .pyi files within this module are handwritten.
# See https://github.com/PyO3/pyo3/issues/2454 for more information about the ongoing
# implementation of the automatic generation of .pyi files from rust

class TwoLayerComponent:
    @staticmethod
    def from_parameters(
        lambda0: float,
        a: float,
        efficacy: float,
        eta: float,
        heat_capacity_surface: float,
        heat_capacity_deep: float,
    ): ...
    def solve(
        self, time_start: float, time_end: float, input_state: dict[str, float]
    ) -> dict[str, float]: ...

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

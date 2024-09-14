from enum import Enum, auto

import numpy as np
from numpy.typing import NDArray

# RSCM uses 64bit floats throughout
Arr = NDArray[np.float64]
F = np.float64 | float

class TimeAxis:
    @staticmethod
    def from_values(values: Arr) -> TimeAxis: ...
    @staticmethod
    def from_bounds(values: Arr) -> TimeAxis: ...
    def values(self): ...
    def bounds(self): ...
    def __len__(self) -> int: ...
    def at(self, index: int) -> F: ...
    def at_bounds(self, index: int) -> tuple[F, F]: ...

class InterpolationStrategy(Enum):
    Linear = auto()
    Next = auto()
    Previous = auto()

class Timeseries:
    def __init__(
        self, values: Arr, time_axis: TimeAxis, units: str, interpolation_strategy
    ) -> Timeseries: ...
    def with_interpolation_strategy(self, interpolation_strategy) -> Timeseries: ...
    def __len__(self) -> int: ...
    def set(self, index: int, value: float): ...
    def values(self) -> Arr: ...
    @property
    def latest(self) -> int: ...
    def latest_value(self) -> F | None: ...
    def at_time(self, time: F) -> F:
        """
        Interpolates a value for a given time using the current interpolation strategy.

        Parameters
        ----------
        time
            Time to interpolate (or potentially extrapolate)

        Raises
        ------
        RuntimeError
            Something went wrong during the interpolation.

            See the exception message for more information.

        Returns
        -------
        Interpolated value

        """

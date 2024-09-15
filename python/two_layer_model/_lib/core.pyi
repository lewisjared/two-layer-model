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
    def at(self, index: int) -> F: ...
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

class VariableType(Enum):
    Exogenous = auto()
    Endogenous = auto()

class TimeseriesCollection:
    def __init__(self) -> TimeseriesCollection: ...
    def add_timeseries(
        self, name: str, timeseries: Timeseries, variable_type: VariableType
    ): ...
    def get_timeseries_by_name(self, name: str) -> Timeseries | None:
        """
        Get a timeseries from the collection by name

        Any modifications to the returned timeseries will not be reflected
        in the collection as this function returns a cloned timeseries.

        Parameters
        ----------
        name
            Name of the timeseries to query

        Returns
        -------
        A clone of the timeseries or None if the collection doesn't contain
        a timeseries by that name.
        """

    def timeseries(self) -> list[Timeseries]:
        """
        Get a list of timeseries stored in the collection.

        These are clones of the original timeseries,
        so they can be modified without affecting the original.

        Returns
        -------
        List of timeseries
        """

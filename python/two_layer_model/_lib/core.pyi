from enum import Enum, auto
from typing import Protocol, TypeVar

import numpy as np
from numpy.typing import NDArray

T = TypeVar("T")

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

class RequirementType(Enum):
    Input = auto()
    Output = auto()
    InputAndOutput = auto()

class RequirementDefinition:
    name: str
    units: str
    requirement_type: RequirementType

class ComponentBuilder(Protocol):
    """A component of the model that can be solved"""

    @classmethod
    def from_parameters(cls: type[T], parameters: dict[str, F]) -> T:
        """
        Create a builder object from parameters

        Returns
        -------
        Builder that can create a Component
        """
    def build(self) -> Component:
        """
        Create a concrete component

        Returns
        -------
        Component object that can be solved
        or coupled with other components via a `Model`.
        """

class Component(Protocol):
    """A component of the model that can be solved"""

    def definitions(self) -> list[RequirementDefinition]: ...
    def solve(
        self, t_current: float, t_next: float, input_state: dict[str, float]
    ) -> dict[str, float]: ...

class CustomComponent(Protocol):
    """
    Interface required for registering Python-based component

    See Also
    --------
    UserDefinedComponent
    """

    def definitions(self) -> list[RequirementDefinition]: ...
    def solve(
        self, t_current: float, t_next: float, input_state: dict[str, float]
    ) -> dict[str, float]: ...

class TestComponent(Component): ...

class UserDerivedComponent(Component):
    def __init__(self, component: CustomComponent) -> UserDerivedComponent: ...

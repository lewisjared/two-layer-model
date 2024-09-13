use crate::timeseries::{Time, TimeAxis, Timeseries};
use numpy::{PyArray1, PyArrayMethods, ToPyArray};
use pyo3::prelude::*;

#[pyclass]
#[pyo3(name = "TimeAxis")]
pub struct PyTimeAxis(pub TimeAxis);

#[pymethods]
impl PyTimeAxis {
    #[staticmethod]
    fn from_values(values: Bound<PyArray1<Time>>) -> Self {
        Self(TimeAxis::from_values(values.to_owned_array()))
    }

    #[staticmethod]
    fn from_bounds(bounds: Bound<PyArray1<Time>>) -> Self {
        Self(TimeAxis::from_bounds(bounds.to_owned_array()))
    }

    fn values<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<Time>> {
        self.0.values().to_pyarray_bound(py)
    }
}

#[pyclass]
#[pyo3(name = "Timeseries")]
pub struct PyTimeseries(pub Timeseries<f32>);

#[pymethods]
impl PyTimeseries {
    #[staticmethod]
    fn from_values<'py>(
        values: Bound<'py, PyArray1<f32>>,
        time: Bound<'py, PyArray1<f32>>,
    ) -> Self {
        PyTimeseries(Timeseries::from_values(
            values.to_owned_array(),
            time.to_owned_array(),
        ))
    }
}

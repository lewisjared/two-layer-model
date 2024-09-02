use crate::timeseries::Timeseries;
use numpy::{PyArray1, PyArrayMethods};
use pyo3::prelude::*;

#[pyclass]
pub struct PyTimeseries(pub Timeseries);

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

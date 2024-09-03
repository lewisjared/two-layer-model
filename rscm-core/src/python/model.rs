use crate::model::Model;
use pyo3::prelude::*;

#[pyclass]
pub struct PyModel(pub Model);

#[pymethods]
impl PyModel {
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

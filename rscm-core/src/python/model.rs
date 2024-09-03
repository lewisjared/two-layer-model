use crate::model::Model;
use pyo3::prelude::*;

#[pyclass]
pub struct PyModel(pub Model);

#[pymethods]
impl PyModel {}

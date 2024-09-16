use crate::model::{Model, ModelBuilder};
use crate::python::component::PyPythonComponent;
use crate::python::timeseries::PyTimeAxis;
use crate::python::PyRustComponent;
use pyo3::prelude::*;

#[pyclass]
#[pyo3(name = "ModelBuilder")]
pub struct PyModelBuilder(pub ModelBuilder);

#[pymethods]
impl PyModelBuilder {
    #[new]
    fn new() -> Self {
        Self(ModelBuilder::new())
    }

    /// Add a component that is defined in rust
    fn with_rust_component<'py>(
        mut self_: PyRefMut<'py, Self>,
        component: Bound<'py, PyRustComponent>,
    ) -> PyResult<PyRefMut<'py, Self>> {
        self_.0.with_component(component.borrow().0.clone());
        Ok(self_)
    }

    /// Pass a component that is defined in python (UserDerivedComponent)
    fn with_py_component<'py>(
        mut self_: PyRefMut<'py, Self>,
        component: Bound<'py, PyPythonComponent>,
    ) -> PyResult<PyRefMut<'py, Self>> {
        let user_derived_component = component.borrow().0.clone();
        self_.0.with_component(user_derived_component);
        Ok(self_)
    }

    fn with_time_axis<'py>(
        mut self_: PyRefMut<'py, Self>,
        time_axis: Bound<PyTimeAxis>,
    ) -> PyResult<PyRefMut<'py, Self>> {
        let time_axis = time_axis.borrow().0.clone();

        self_.0.time_axis = time_axis;
        Ok(self_)
    }
}

#[pyclass]
#[pyo3(name = "Model")]
pub struct PyModel(pub Model);

#[pymethods]
impl PyModel {
    // Not exposing initialiser deliberately
}

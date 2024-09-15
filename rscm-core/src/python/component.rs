/// Macros for exposing a component to Python and using python-defined modules in rust
use crate::component::{Component, InputState, OutputState};
use crate::errors::RSCMResult;
use crate::timeseries::{FloatValue, Time};
use pyo3::prelude::*;
use std::collections::HashMap;

// Reexport the Requirement Definition
pub use crate::component::{RequirementDefinition, RequirementType};

/// Enable the creation of components in python
#[macro_export]
macro_rules! impl_component_from_parameters {
    ($py_component:ty, $component:ty, $parameter_type:ty) => {
        #[pymethods]
        impl $py_component {
            #[staticmethod]
            // Technically from_parameters isn't part of the Component trait (due to needing a generic),
            // but it is a common pattern used by all
            fn from_parameters(parameters: Bound<PyAny>) -> PyResult<Self> {
                use pyo3::exceptions::PyValueError;
                use pythonize::depythonize_bound;

                // todo: figure out how to use an attrs class as parameters instead of a dict
                let parameters = depythonize_bound::<$parameter_type>(parameters);
                match parameters {
                    Ok(parameters) => Ok(Self(<$component>::from_parameters(parameters))),
                    Err(e) => Err(PyValueError::new_err(format!("{}", e))),
                }
            }
        }
    };
}

/// Expose component-related functionality to python
#[macro_export]
macro_rules! impl_component {
    ($py_component:ty) => {
        #[pymethods]
        impl $py_component {
            fn definitions(&self) -> Vec<RequirementDefinition> {
                self.0.definitions()
            }

            pub fn solve(
                &mut self,
                t_current: Time,
                t_next: Time,
                input_state: HashMap<String, FloatValue>,
            ) -> PyResult<HashMap<String, FloatValue>> {
                let state = InputState::from_hashmap(input_state);
                let output_state = Component::solve(&self.0, t_current, t_next, &state)?;
                Ok(output_state.to_hashmap())
            }
        }
    };
}

#[pyclass]
#[derive(Debug)]
pub struct UserDerivedComponent {
    component: PyObject,
}

impl Component for UserDerivedComponent {
    fn definitions(&self) -> Vec<RequirementDefinition> {
        vec![]
    }

    fn solve(
        &self,
        t_current: Time,
        t_next: Time,
        input_state: &InputState,
    ) -> RSCMResult<OutputState> {
        Python::with_gil(|py| {
            let py_result = self
                .component
                .bind(py)
                .call_method(
                    "solve",
                    (t_current, t_next, input_state.clone().to_hashmap()),
                    None,
                )
                .unwrap();

            let state = OutputState::from_hashmap(py_result.extract().unwrap());
            Ok(state)
        })
    }
}

#[pyclass]
pub struct PyUserDerivedComponent(UserDerivedComponent);

#[pymethods]
impl PyUserDerivedComponent {
    #[new]
    pub fn new(component: Py<PyAny>) -> Self {
        Self(UserDerivedComponent { component })
    }
}

impl_component!(PyUserDerivedComponent);

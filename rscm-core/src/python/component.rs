/// Macros for exposing a component to Python and using python-defined modules in rust
// Reexport the Requirement Definition
pub use crate::component::{RequirementDefinition, RequirementType};

#[macro_export]
macro_rules! impl_component {
    // This macro takes an argument of designator `ident` and
    // creates a function named `$func_name`.
    // The `ident` designator is used for variable/function names.
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

            fn definitions(&self) -> Vec<RequirementDefinition> {
                self.0.definitions()
            }
        }
    };
}

use crate::errors::RSCMError;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::{pymodule, Bound, PyResult};

mod model;
pub mod timeseries;
mod timeseries_collection;

#[pymodule]
pub fn core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<timeseries::PyTimeAxis>()?;
    m.add_class::<timeseries::PyTimeseries>()?;
    m.add_class::<timeseries::PyInterpolationStrategy>()?;
    m.add_class::<timeseries_collection::PyTimeseriesCollection>()?;
    m.add_class::<timeseries_collection::VariableType>()?;
    Ok(())
}

impl From<RSCMError> for PyErr {
    fn from(e: RSCMError) -> PyErr {
        PyRuntimeError::new_err(e.to_string())
    }
}

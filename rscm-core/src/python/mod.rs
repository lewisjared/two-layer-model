use pyo3::prelude::*;
use pyo3::{pymodule, Bound, PyResult};

mod model;
pub mod timeseries;

#[pymodule]
pub fn core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<timeseries::PyTimeAxis>()?;
    m.add_class::<timeseries::PyTimeseries>()?;
    m.add_class::<timeseries::PyInterpolationStrategy>()?;
    Ok(())
}

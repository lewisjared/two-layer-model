use crate::python::timeseries::PyTimeseries;
use crate::timeseries_collection::TimeseriesCollection;
use pyo3::prelude::*;

pub use crate::timeseries_collection::VariableType;
#[pyclass]
#[pyo3(name = "TimeseriesCollection")]
pub struct PyTimeseriesCollection(pub TimeseriesCollection);

#[pymethods]
impl PyTimeseriesCollection {
    #[new]
    fn new() -> Self {
        Self(TimeseriesCollection::new())
    }

    fn __repr__(&self) -> String {
        let names: Vec<&str> = self.0.iter().map(|x| x.name.as_str()).collect();
        format!("<TimeseriesCollection names={:?}>", names)
    }

    pub fn add_timeseries(
        &mut self,
        name: String,
        timeseries: Bound<PyTimeseries>,
        variable_type: VariableType,
    ) {
        let timeseries = timeseries.borrow().0.clone();
        self.0.add_timeseries(name, timeseries, variable_type);
    }
}

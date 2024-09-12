use crate::timeseries::Timeseries;
use std::vec::IntoIter;

#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Debug)]
#[pyo3::pyclass]
pub enum VariableType {
    /// Values that are defined outside of the model
    Exogenous,
    /// Values that are determined within the model
    Endogenous,
}

#[derive(Debug)]
pub struct TimeseriesItem {
    pub timeseries: Timeseries<f32>,
    pub name: String,
    pub variable_type: VariableType,
}

/// A collection of time series data.
/// Allows for easy access to time series data by name across the whole model
#[derive(Debug)]
pub struct TimeseriesCollection {
    timeseries: Vec<TimeseriesItem>,
}

impl TimeseriesCollection {
    pub fn new() -> Self {
        Self {
            timeseries: Vec::new(),
        }
    }

    /// Add a new timeseries to the collection
    ///
    /// Panics if a timeseries with the same name already exists in the collection
    /// TODO: Revisit if this is the correct way of handling this type of error
    pub fn add_timeseries(
        &mut self,
        name: String,
        timeseries: Timeseries<f32>,
        variable_type: VariableType,
    ) {
        if self.timeseries.iter().any(|x| x.name == name) {
            panic!("timeseries {} already exists", name)
        }
        self.timeseries.push(TimeseriesItem {
            timeseries,
            name,
            variable_type,
        });
    }

    pub fn get_by_name(&self, name: &str) -> Option<&TimeseriesItem> {
        self.timeseries.iter().find(|x| x.name == name)
    }

    pub fn get_timeseries_by_name(&self, name: &str) -> Option<&Timeseries<f32>> {
        self.get_by_name(name).map(|item| &item.timeseries)
    }
    pub fn get_timeseries_by_name_mut(&mut self, name: &str) -> Option<&mut Timeseries<f32>> {
        self.timeseries
            .iter_mut()
            .find(|x| x.name == name)
            .map(|item| &mut item.timeseries)
    }

    pub fn iter(&self) -> impl Iterator<Item = &TimeseriesItem> {
        self.timeseries.iter()
    }

    pub fn into_iter(self) -> IntoIter<TimeseriesItem> {
        self.timeseries.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use numpy::array;
    use numpy::ndarray::Array;

    #[test]
    fn adding() {
        let mut collection = TimeseriesCollection::new();

        let timeseries =
            Timeseries::from_values(array![1.0, 2.0, 3.0], Array::range(2020.0, 2023.0, 1.0));
        collection.add_timeseries(
            "Surface Temperature".to_string(),
            timeseries.clone(),
            VariableType::Exogenous,
        );
        collection.add_timeseries(
            "Emissions|CO2".to_string(),
            timeseries.clone(),
            VariableType::Endogenous,
        );
    }

    #[test]
    #[should_panic]
    fn adding_same_name() {
        let mut collection = TimeseriesCollection::new();

        let timeseries =
            Timeseries::from_values(array![1.0, 2.0, 3.0], Array::range(2020.0, 2023.0, 1.0));
        collection.add_timeseries(
            "test".to_string(),
            timeseries.clone(),
            VariableType::Exogenous,
        );
        collection.add_timeseries(
            "test".to_string(),
            timeseries.clone(),
            VariableType::Endogenous,
        );
    }
}

use crate::timeseries::{Time, TimeAxis};
use crate::timeseries_collection::TimeseriesCollection;

struct Model {
    ts: TimeseriesCollection,
    time_axis: TimeAxis,
    time_index: usize,
}

/// A model represents a collection of components that can be solved together
/// Each component may require information from other components to be solved (endrogenous) or
/// predefined data (exogenous).
impl Model {
    pub fn new(time_axis: TimeAxis) -> Self {
        Self {
            ts: TimeseriesCollection::new(),
            time_axis,
            time_index: 0,
        }
    }

    /// Gets the time value at the current step
    pub fn current_time(&self) -> Time {
        self.time_axis.at(self.time_index).unwrap()
    }

    /// Steps the model forward one time step
    pub fn step(&mut self) {
        assert!(self.time_index < self.time_axis.len());

        self.time_index += 1
    }

    /// Steps the model until the end of the time axis
    pub fn run(&mut self) {
        while self.time_index < self.time_axis.len() {
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use numpy::ndarray::Array;

    #[test]
    fn step() {
        let time_axis = TimeAxis::from_values(Array::range(2020.0, 2025.0, 1.0));
        let mut model = Model::new(time_axis);

        assert_eq!(model.time_index, 0);
        model.step();
        model.step();
        assert_eq!(model.time_index, 2);
        assert_eq!(model.current_time(), 2022.0);
        model.run();
        assert_eq!(model.time_index, 5);
    }
}

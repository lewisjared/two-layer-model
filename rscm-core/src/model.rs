use crate::timeseries::{Time, TimeAxis};
use crate::timeseries_collection::TimeseriesCollection;

struct Model {
    ts: TimeseriesCollection,
    time_axis: TimeAxis,
    time_index: usize,
}

impl Model {
    pub fn new(time_axis: TimeAxis) -> Self {
        Self {
            ts: TimeseriesCollection::new(),
            time_axis,
            time_index: 0,
        }
    }

    pub fn current_time(&self) -> Time {
        self.time_axis.at(self.time_index).unwrap()
    }

    pub fn step() {}
}

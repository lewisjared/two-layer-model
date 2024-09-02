use crate::timeseries_collection::TimeseriesCollection;

struct Model {
    ts: TimeseriesCollection,
}

impl Model {
    pub fn new() -> Self {
        Self {
            ts: TimeseriesCollection::new(),
        }
    }
    pub fn step() {}
}

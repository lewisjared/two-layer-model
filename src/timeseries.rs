use std::cmp::Ordering;

type Time = f32;

pub struct TimeAxis {
    values: Vec<Time>,
    bounds: Vec<Time>,
}

impl TimeAxis {
    pub fn from_values(values: Vec<Time>) -> Self {
        let mut bounds = values.clone();
        bounds.push(values.last().expect("No value") + 1.0);

        Self { values, bounds }
    }

    pub fn from_bounds(bounds: Vec<Time>) -> Self {
        assert!(bounds.len() > 1);
        let mut values = bounds.clone();
        values.pop();

        Self { values, bounds }
    }

    pub fn contains(&self, value: Time) -> bool {
        self.values.contains(&value)
    }

    pub fn values(&self) -> &Vec<Time> {
        &self.values
    }

    pub fn bounds(&self) -> &Vec<Time> {
        &self.bounds
    }
}

pub struct Timeseries<T> {
    // A temporally contiguous set of values
    name: String,
    // TODO: Should be NDArray
    values: Vec<T>,
    // TODO: Should be NDArray
    time_bounds: Vec<Time>,
}

impl<T> Timeseries<T> {
    pub fn new() -> Timeseries<T> {
        Self {
            name: "".to_string(),
            values: vec![],
            time_bounds: vec![],
        }
    }
    pub fn at_time(&self, time: Time) -> &T {
        let nearest = self
            .time_bounds
            .binary_search_by(|probe| match probe.ge(&time) {
                true => Ordering::Less,
                false => Ordering::Greater,
            })
            .expect("Search failed");
        &self.values[nearest]
    }

    pub fn from_values(values: Vec<T>, time: Vec<Time>) -> Self {
        assert_eq!(values.len(), time.len());

        Self {
            name: "".to_string(),
            values,
            time_bounds: time,
        }
    }
}

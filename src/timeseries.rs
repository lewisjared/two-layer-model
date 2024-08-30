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
}

pub struct Timeseries<T> {
    // A temporally contiguous set of values
    name: String,
    // TODO: Should be NDArray
    values: Vec<T>,
    time_axis: TimeAxis,
}

impl<T> Timeseries<T> {
    pub fn at_time(&self, time: Time) -> &T {
        let nearest = self
            .time_axis
            .values
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
            time_axis: TimeAxis::from_values(time),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_value() {
        let result = Timeseries::from_values(
            vec![1, 2, 3, 4, 5],
            vec![2020.0, 2021.0, 2022.0, 2023.0, 2024.0],
        );
        assert_eq!(result.at_time(2020.0), &1);
        assert_eq!(result.at_time(2020.5), &1);
        assert_eq!(result.at_time(2021.0), &2);
    }
}

use ndarray_interp::interp1d::{Interp1DBuilder, Linear};
use ndarray_interp::InterpolateError;
use numpy::ndarray::prelude::*;
use numpy::ndarray::{Array, Array1, OwnedRepr};
use std::iter::zip;
use std::sync::Arc;

pub type Time = f32;

#[derive(Clone, Debug)]
pub struct TimeAxis {
    bounds: Array1<Time>,
}

fn check_monotonic_increasing(arr: &Array1<Time>) -> bool {
    let mut zipped_arr = zip(arr.slice(s![0..arr.len() - 1]), arr.slice(s![1..]));

    // Check that [i + 1] > [i]
    zipped_arr.all(|(&a, &b)| b > a)
}

/// Axis for a time series
///
/// The time values must be monotonically increasing with
/// contiguous bounds (i.e. there cannot be any gaps).
///
/// The convention used here is that the value represents the start of a time step.
/// Each time step has a half-open bound that denotes the time period over which that step is
/// calculated.
///
/// Generally, decimal year values are used throughout
impl TimeAxis {
    pub fn new(bounds: Array1<Time>) -> Self {
        let is_monotonic = check_monotonic_increasing(&bounds);
        assert!(is_monotonic);

        Self { bounds }
    }

    /// Initialise using values
    ///
    /// Assumes that the size of the last time step is equal to the size of the previous time step
    ///
    /// # Example
    ///
    /// ```rust
    /// use numpy::array;
    /// use rscm_core::timeseries::TimeAxis;
    /// let ta = TimeAxis::from_values(array![1.0, 2.0, 3.0]);
    /// assert_eq!(ta.at_bounds(2).unwrap(), (3.0, 4.0));
    /// ```
    pub fn from_values(values: Array1<Time>) -> Self {
        assert!(values.len() > 2);
        let step = values[values.len() - 1] - values[values.len() - 2];

        let mut bounds = Array::zeros(values.len() + 1);
        bounds.slice_mut(s![..values.len()]).assign(&values);

        let last_idx = bounds.len() - 1;

        bounds[last_idx] = bounds[last_idx - 1] + step;
        Self::new(bounds)
    }

    /// Initialise using bounds
    ///
    /// # Example
    ///
    /// ```rust
    /// use numpy::array;
    /// use rscm_core::timeseries::TimeAxis;
    /// let ta = TimeAxis::from_bounds(array![1.0, 2.0, 3.0, 4.0]);
    /// assert_eq!(ta.len(), 3);
    /// ```
    pub fn from_bounds(bounds: Array1<Time>) -> Self {
        assert!(bounds.len() > 1);

        Self { bounds }
    }

    pub fn values(&self) -> ArrayView1<Time> {
        self.bounds.slice(s![0..self.len()])
    }

    /// Get the last time value
    pub fn len(&self) -> usize {
        self.bounds.len() - 1
    }

    /// Get the number of bounds
    ///
    /// This is always 1 larger than the number of values
    pub fn len_bounds(&self) -> usize {
        self.bounds.len()
    }

    /// Get the first time value
    // TODO: Investigate Time vs &Time
    pub fn first(&self) -> &Time {
        self.bounds.first().unwrap()
    }

    /// Get the last time value
    pub fn last(&self) -> &Time {
        self.bounds.get(self.len()).unwrap()
    }

    /// Get the time value for a step
    ///
    /// # Example
    ///
    /// ```rust
    /// use numpy::array;
    /// use rscm_core::timeseries::TimeAxis;
    /// let ta = TimeAxis::from_values(array![1.0, 2.0, 3.0]);
    /// assert_eq!(ta.at(1).unwrap(), 2.0);
    /// assert_eq!(ta.at(27), None);
    /// ```
    pub fn at(&self, index: usize) -> Option<Time> {
        if index < self.len() {
            Option::from(self.bounds[index])
        } else {
            None
        }
    }

    /// Get the bounds for a given index
    pub fn at_bounds(&self, index: usize) -> Option<(Time, Time)> {
        if index < self.len() {
            let bound: (Time, Time) = (self.bounds[index], self.bounds[index + 1]);
            Option::from(bound)
        } else {
            None
        }
    }

    /// Check if the axis contains a given value
    ///
    /// # Example
    ///
    /// ```rust
    /// use numpy::array;
    /// use rscm_core::timeseries::TimeAxis;
    /// let ta = TimeAxis::from_values(array![1.0, 2.0, 3.0]);
    /// assert!(ta.contains(1.0));
    /// assert!(!ta.contains(27.0));
    /// ```
    pub fn contains(&self, value: Time) -> bool {
        let mut found = false;

        for v in self.values().iter() {
            if value == *v {
                found = true;
                break;
            }
        }
        found
    }
}

/// A contiguous set of values
///
#[derive(Clone, Debug)]
pub struct Timeseries {
    units: String,
    // TODO: Make type-agnostic
    values: Array1<f32>,
    // Using a reference counted time axis to avoid having to maintain multiple clones of the
    // time axis.
    time_axis: Arc<TimeAxis>,
}

impl Timeseries {
    /// Get the default interpolator
    ///
    /// Allows the strategy to be overwritten if needed:
    /// ```rust
    /// # use numpy::ndarray::{array, Array};
    /// # use ndarray_interp::interp1d::Linear;
    /// # use crate::rscm_core::timeseries::Timeseries;
    /// let data = array![1.0, 1.5, 2.0];
    /// let years = Array::range(2020.0, 2023.0, 1.0);
    /// let query = 2024.0;
    /// let expected = 3.0;
    ///
    /// let timeseries = Timeseries::from_values(data, years);
    /// let interpolator = timeseries
    ///     .interpolator()
    ///     .strategy(Linear::new().extrapolate(true))
    ///     .build()
    ///     .unwrap();
    /// let result = interpolator.interp_scalar(query).unwrap();
    /// # assert_eq!(result, expected);
    pub fn interpolator(&self) -> Interp1DBuilder<OwnedRepr<f32>, OwnedRepr<Time>, Ix1, Linear> {
        Interp1DBuilder::new(self.values.clone()).x(self.time_axis.values().to_owned())
    }

    /// Get the value at a given time
    ///
    /// Linearly interpolates between values so doesn't currently do anything that is "spline"
    /// aware.
    pub fn at_time(&self, time: Time) -> Result<f32, InterpolateError> {
        let interp = self.interpolator().build().unwrap();

        interp.interp_scalar(time)
    }

    pub fn new(values: Array1<f32>, time_axis: Arc<TimeAxis>, units: String) -> Self {
        assert_eq!(values.len(), time_axis.values().len());

        Self {
            units,
            values,
            time_axis,
        }
    }

    pub fn from_values(values: Array1<f32>, time: Array1<f32>) -> Self {
        assert_eq!(values.len(), time.len());

        Self {
            units: "".to_string(),
            values,
            time_axis: Arc::new(TimeAxis::from_values(time)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn check_monotonic_values() {
        Timeseries::from_values(array![1.0, 2.0, 3.0], array![2020.0, 1.0, 2021.0,]);
    }

    #[test]
    fn get_value() {
        let result = Timeseries::from_values(
            array![1.0, 2.0, 3.0, 4.0, 5.0],
            Array::range(2020.0, 2025.0, 1.0),
        );
        assert_eq!(result.at_time(2020.0).unwrap(), 1.0);
        assert_eq!(result.at_time(2020.5).unwrap(), 1.5);
        assert_eq!(result.at_time(2021.0).unwrap(), 2.0);

        // Linear extrapolate isn't allowed by default
        assert!(result.at_time(2025.0).is_err());
    }

    #[test]
    fn custom_interpolator() {
        let data = array![1.0, 1.5, 2.0];
        let years = Array::range(2020.0, 2023.0, 1.0);
        let query = 2024.0;
        let expected = 3.0;

        let timeseries = Timeseries::from_values(data, years);
        let interpolator = timeseries
            .interpolator()
            .strategy(Linear::new().extrapolate(true))
            .build()
            .unwrap();
        let result = interpolator.interp_scalar(query).unwrap();

        assert_eq!(result, expected);
    }
}

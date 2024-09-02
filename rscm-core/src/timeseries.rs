use ndarray_interp::interp1d::{Interp1DBuilder, Linear};
use ndarray_interp::InterpolateError;
use numpy::ndarray::prelude::*;
use numpy::ndarray::{Array, Array1, OwnedRepr};
use std::iter::zip;

type Time = f32;

#[derive(Clone, Debug)]
pub struct TimeAxis {
    values: Array1<Time>,
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
impl TimeAxis {
    pub fn new(values: Array1<Time>, bounds: Array1<Time>) -> Self {
        assert_eq!(values.len(), bounds.len() - 1);

        let is_monotonic = check_monotonic_increasing(&values);
        assert!(is_monotonic);

        Self { values, bounds }
    }

    pub fn from_values(values: Array1<Time>) -> Self {
        let mut bounds = Array::zeros(values.len() + 1);
        bounds.slice_mut(s![..values.len()]).assign(&values);

        Self::new(values, bounds)
    }

    pub fn from_bounds(bounds: Array1<Time>) -> Self {
        assert!(bounds.len() > 1);
        let values = bounds.slice(s![..bounds.len() - 1]).to_owned();

        Self { values, bounds }
    }

    pub fn contains(&self, value: &Time) -> bool {
        let mut found = false;

        for v in self.values.iter() {
            if value == v {
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
    name: String,
    // TODO: Make type-agnostic
    values: Array1<f32>,
    time_axis: TimeAxis,
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
        Interp1DBuilder::new(self.values.clone()).x(self.time_axis.values.clone())
    }

    /// Get the value at a given time
    ///
    /// Linearly interpolates between values so doesn't currently do anything that is "spline"
    /// aware.
    pub fn at_time(&self, time: Time) -> Result<f32, InterpolateError> {
        let interp = self.interpolator().build().unwrap();

        interp.interp_scalar(time)
    }

    pub fn from_values(values: Array1<f32>, time: Array1<f32>) -> Self {
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

use crate::errors::RSCMResult;
use crate::interpolate::strategies::{InterpolationStrategy, LinearSplineStrategy};
use crate::interpolate::Interp1d;
use nalgebra::max;
use num::{Float, ToPrimitive};
use numpy::ndarray::prelude::*;
use numpy::ndarray::{Array, Array1, ViewRepr};
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
    /// let expected: (f32, f32) = (3.0, 4.0);
    /// assert_eq!(ta.at_bounds(2).unwrap(), expected);
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

    pub fn get_index(&self, time: Time) -> usize {
        self.bounds
            .as_slice()
            .unwrap()
            // Have to use binary_search_by as
            .binary_search_by(|v| v.partial_cmp(&time).expect("Couldn't compare values"))
            .unwrap()
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
pub struct Timeseries<T>
where
    T: Float,
{
    units: String,
    values: Array1<T>,
    // Using a reference counted time axis to avoid having to maintain multiple clones of the
    // time axis.
    time_axis: Arc<TimeAxis>,
    /// Latest value specified
    latest: isize,
    interpolation_strategy: InterpolationStrategy,
}

impl<T> Timeseries<T>
where
    T: Float + From<Time>,
{
    pub fn new(
        values: Array1<T>,
        time_axis: Arc<TimeAxis>,
        units: String,
        interpolation_strategy: InterpolationStrategy,
    ) -> Self {
        assert_eq!(values.len(), time_axis.values().len());

        let latest = values
            .iter()
            .take_while(|x| !x.is_nan())
            .count()
            .to_isize()
            .unwrap();

        Self {
            units,
            values,
            time_axis,
            latest,
            interpolation_strategy,
        }
    }

    /// Create a new timeseries from a set of values and a time axis
    ///
    /// The interpolation strategy for the timeseries defaults to linear with extrapolation.
    pub fn from_values(values: Array1<T>, time: Array1<Time>) -> Self {
        Self::new(
            values,
            Arc::new(TimeAxis::from_values(time)),
            "".to_string(),
            InterpolationStrategy::from(LinearSplineStrategy::new(true)),
        )
    }

    /// Replace the interpolation strategy
    pub fn with_interpolation_strategy(
        &mut self,
        interpolation_strategy: InterpolationStrategy,
    ) -> &Self {
        self.interpolation_strategy = interpolation_strategy;
        self
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Set a value at time_index
    pub fn set(&mut self, time_index: usize, value: T) {
        assert!(time_index < self.len());
        self.values[time_index] = value;

        if !value.is_nan() {
            self.latest = max(self.latest, time_index.to_isize().unwrap())
        }
    }

    /// Get the index of the lastest valid timestep
    ///
    /// Doesn't verify that all prior values are non-nan
    pub fn latest(&self) -> &isize {
        &self.latest
    }

    /// Get the value for the latest valid timestep
    ///
    /// Doesn't verify that all prior values are non-nan
    pub fn latest_value(&self) -> Option<T> {
        match (self.latest < 0) & (self.latest.to_usize().unwrap() < self.len()) {
            true => None,
            false => Option::from(self.values[self.latest.to_usize().unwrap()]),
        }
    }

    pub fn new_empty(
        time_axis: Arc<TimeAxis>,
        units: String,
        interpolation_strategy: InterpolationStrategy,
    ) -> Self {
        let mut arr = Array::zeros(time_axis.len());
        arr.fill(T::nan());

        Self::new(arr, time_axis, units, interpolation_strategy)
    }

    /// Get the interpolator used to interpolate values onto a different timebase
    pub fn interpolator(&self) -> Interp1d<ViewRepr<&Time>, ViewRepr<&T>> {
        Interp1d::new(
            self.time_axis.values(),
            self.values.view(),
            self.interpolation_strategy.clone(),
        )
    }

    /// Get the value at a given time
    ///
    /// Linearly interpolates between values so doesn't currently do anything that is "spline"
    /// aware.
    pub fn at_time(&self, time: Time) -> RSCMResult<T> {
        let interp = self.interpolator();

        interp.interpolate(time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolate::strategies::{InterpolationStrategy, PreviousStrategy};

    #[test]
    #[should_panic]
    fn check_monotonic_values() {
        Timeseries::from_values(array![1.0, 2.0, 3.0], array![2020.0, 1.0, 2021.0,]);
    }

    #[test]
    fn get_value() {
        let mut result = Timeseries::from_values(
            array![1.0, 2.0, 3.0, 4.0, 5.0],
            Array::range(2020.0, 2025.0, 1.0),
        );

        result.with_interpolation_strategy(InterpolationStrategy::from(LinearSplineStrategy::new(
            false,
        )));
        assert_eq!(result.at_time(2020.0).unwrap(), 1.0);
        assert_eq!(result.at_time(2020.5).unwrap(), 1.5);
        assert_eq!(result.at_time(2021.0).unwrap(), 2.0);

        // Linear extrapolate isn't allowed by default
        assert!(result.at_time(2026.0).is_err());
    }

    #[test]
    fn custom_interpolator() {
        let data = array![1.0, 1.5, 2.0];
        let years = Array::range(2020.0, 2023.0, 1.0);
        let query = 2024.0;

        let mut timeseries = Timeseries::from_values(data, years);

        // Default to linear interpolation
        let result = timeseries.at_time(query).unwrap();
        assert_eq!(result, 3.0);

        // Replace interpolation strategy
        timeseries
            .with_interpolation_strategy(InterpolationStrategy::from(PreviousStrategy::new(true)));
        let result = timeseries.at_time(query).unwrap();
        assert_eq!(result, 2.0);
    }
}

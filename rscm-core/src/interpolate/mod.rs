/// 1d interpolation
///
///
/// # Technical implementation
/// Dynamic dispatch was difficult to implement because of the generics associated with
/// the `Interp1DStrategy` trait. These generics make the trait not object-safe which
/// doesn't allow the
///
/// Instead static dispatching was implemented using the enum `InterpolationStrategy` which
/// implements the `Interp1DStrategy` trait.
///
/// Static dispatching decreases the ability for consumers of this library to implement their
/// own custom interpolation strategies. This isn't intended as a generic implementation of
/// interpolation routines so that is a satisfactory tradeoff.
///
///
use crate::errors::{RSCMError, RSCMResult};
use is_close::is_close;
use num::Float;
use numpy::ndarray::Array1;
use std::fmt::{Debug, Formatter};

mod linear_spline;
mod next;
mod previous;

pub use linear_spline::Interp1DLinearSpline;
pub use next::Interp1DNext;
pub use previous::Interp1DPrevious;

#[derive(Clone)]
pub enum InterpolationStrategy {
    Linear(Interp1DLinearSpline),
    Next(Interp1DNext),
    Previous(Interp1DPrevious),
}

impl<T, V> Interp1DStrategy<T, V> for InterpolationStrategy
where
    T: Float + Into<V>,
    V: Float + Into<T>,
{
    fn interpolate(&self, time: &Array1<T>, y: &Array1<V>, time_target: T) -> RSCMResult<V> {
        match self {
            InterpolationStrategy::Linear(strat) => strat.interpolate(time, y, time_target),
            InterpolationStrategy::Next(strat) => strat.interpolate(time, y, time_target),
            InterpolationStrategy::Previous(strat) => strat.interpolate(time, y, time_target),
        }
    }
}

impl From<Interp1DLinearSpline> for InterpolationStrategy {
    fn from(value: Interp1DLinearSpline) -> Self {
        InterpolationStrategy::Linear(value)
    }
}

impl From<Interp1DNext> for InterpolationStrategy {
    fn from(value: Interp1DNext) -> Self {
        InterpolationStrategy::Next(value)
    }
}

impl From<Interp1DPrevious> for InterpolationStrategy {
    fn from(value: Interp1DPrevious) -> Self {
        InterpolationStrategy::Previous(value)
    }
}

impl Debug for InterpolationStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InterpolationStrategy").finish()
    }
}

pub struct Interp1d<'a, T, V>
where
    T: Float,
{
    time: &'a Array1<T>,
    y: &'a Array1<V>,
    strategy: InterpolationStrategy,
}

impl<'a, T, V> Interp1d<'a, T, V>
where
    T: Float,
    V: Float,
{
    pub fn new(time: &'a Array1<T>, y: &'a Array1<V>, strategy: InterpolationStrategy) -> Self {
        Self { time, y, strategy }
    }
    pub fn with_strategy(&mut self, strategy: InterpolationStrategy) -> &mut Self {
        self.strategy = strategy;
        self
    }

    pub fn interpolate(&self, time_target: T) -> RSCMResult<T> {
        self.strategy.interpolate(self.time, self.y, time_target)
    }
}

#[derive(PartialEq)]
pub enum SegmentOptions {
    InSegment,
    ExtrapolateBackward,
    ExtrapolateForward,
    OnBoundary,
}

pub trait Interp1DStrategy<T, V>
where
    T: Float,
{
    fn find_segment(
        &self,
        target: T,
        time_bounds: &Array1<T>,
        extrapolate: bool,
    ) -> RSCMResult<(SegmentOptions, usize)> {
        let end_segment_idx = self.find_segment_index(&target, time_bounds);

        let needs_extrap_forward = end_segment_idx == time_bounds.len();
        let needs_extrap_backward = !needs_extrap_forward & (end_segment_idx == 0);

        // Check if we can fast return because there is an exact match
        if !needs_extrap_forward {
            if is_close!(time_bounds[end_segment_idx], target) {
                return Ok((SegmentOptions::OnBoundary, end_segment_idx));
            }
        }

        let needs_extrap = needs_extrap_backward | needs_extrap_forward;

        if needs_extrap & (!extrapolate) {
            if needs_extrap_backward {
                return Err(RSCMError::ExtrapolationNotAllowed(
                    target.to_f32().unwrap(),
                    "start of".to_string(),
                    time_bounds[0].to_f32().unwrap(),
                ));
            } else {
                return Err(RSCMError::ExtrapolationNotAllowed(
                    target.to_f32().unwrap(),
                    "end of".to_string(),
                    time_bounds[time_bounds.len() - 1].to_f32().unwrap(),
                ));
            }
        }
        if needs_extrap_backward {
            Ok((SegmentOptions::ExtrapolateBackward, 0))
        } else if needs_extrap_forward {
            Ok((SegmentOptions::ExtrapolateForward, time_bounds.len()))
        } else {
            Ok((SegmentOptions::InSegment, end_segment_idx))
        }
    }

    fn find_segment_index(&self, target: &T, time_bounds: &Array1<T>) -> usize {
        let result = time_bounds
            .as_slice()
            .unwrap()
            // Have to use binary_search_by as
            .binary_search_by(|v| v.partial_cmp(&target).expect("Couldn't compare values"));

        result.unwrap_or_else(|res| res)
    }

    fn interpolate(&self, time: &Array1<T>, y: &Array1<V>, time_target: T) -> RSCMResult<V>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use numpy::array;
    use numpy::ndarray::Array;

    #[test]
    fn interpolate() {
        let data = array![1.0, 1.5, 2.0];
        let years = Array::range(2020.0, 2023.0, 1.0);
        let query = 2024.0;
        let expected = 3.0;

        let interpolator = Interp1d::new(
            &years,
            &data,
            InterpolationStrategy::from(Interp1DNext::new(false)),
        );
        let result = interpolator.interpolate(query).unwrap();

        assert_eq!(result, expected);
    }
}

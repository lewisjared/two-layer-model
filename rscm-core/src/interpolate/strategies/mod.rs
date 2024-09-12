pub mod linear_spline;
pub mod next;
pub mod previous;

use crate::errors::{RSCMError, RSCMResult};
use is_close::is_close;
pub use linear_spline::Interp1DLinearSpline;
pub use next::Interp1DNext;
use num::{Float, ToPrimitive};
use numpy::ndarray::{ArrayBase, Data};
use numpy::Ix1;
pub use previous::Interp1DPrevious;
use std::fmt::{Debug, Formatter};

#[derive(PartialEq)]
pub(crate) enum SegmentOptions {
    InSegment,
    ExtrapolateBackward,
    ExtrapolateForward,
    OnBoundary,
}

fn find_segment<T>(
    target: T::Elem,
    time_bounds: &ArrayBase<T, Ix1>,
    extrapolate: bool,
) -> RSCMResult<(SegmentOptions, usize)>
where
    T: Data,
    T::Elem: Float,
{
    let end_segment_idx = find_segment_index(&target, time_bounds);

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

fn find_segment_index<T>(target: &T::Elem, time_bounds: &ArrayBase<T, Ix1>) -> usize
where
    T: Data,
    T::Elem: PartialOrd,
{
    let result = time_bounds
        .as_slice()
        .unwrap()
        // Have to use binary_search_by as
        .binary_search_by(|v| v.partial_cmp(&target).expect("Couldn't compare values"));

    result.unwrap_or_else(|res| res)
}

/// Strategy for interpolating a set of 1D values along a time axis
///
/// A simple climate model often needs to perform mathematical operations on a timeseries
/// that require assumptions about how to convert between discrete and continuous data
/// (e.g. integration and differentiation). These assumptions can be encoded using different
/// interpolation strategies.
pub trait Interp1DStrategy<At, Ay>
where
    At: Data,
    Ay: Data,
{
    /// Interpolate the value at a given time
    /// This is used internally by [`Interp1D`].
    fn interpolate(
        &self,
        time: &ArrayBase<At, Ix1>,
        y: &ArrayBase<Ay, Ix1>,
        time_target: At::Elem,
    ) -> RSCMResult<Ay::Elem>;
}

#[derive(Clone)]
pub enum InterpolationStrategy {
    Linear(Interp1DLinearSpline),
    Next(Interp1DNext),
    Previous(Interp1DPrevious),
}

impl<At, Ay> Interp1DStrategy<At, Ay> for InterpolationStrategy
where
    At: Data,
    At::Elem: Float,
    Ay: Data,
    Ay::Elem: Float + From<At::Elem>,
{
    fn interpolate(
        &self,
        time: &ArrayBase<At, Ix1>,
        y: &ArrayBase<Ay, Ix1>,
        time_target: At::Elem,
    ) -> RSCMResult<Ay::Elem> {
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

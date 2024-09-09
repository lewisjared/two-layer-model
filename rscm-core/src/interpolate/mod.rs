use crate::errors::{RSCMError, RSCMResult};
use is_close::is_close;
use num::Float;
use numpy::ndarray::Array1;
use std::fmt::Display;

mod linear_spline;
mod next;
mod previous;

#[derive(PartialEq)]
pub enum SegmentOptions {
    InSegment,
    ExtrapolateBackward,
    ExtrapolateForward,
    OnBoundary,
}

pub trait Interpolate<T, V>
where
    T: Float + Display,
{
    fn find_segment(
        &self,
        target: T,
        time_bounds: &Array1<T>,
        allow_extrapolation: bool,
    ) -> RSCMResult<(SegmentOptions, usize)> {
        let end_segment_idx = Self::find_segment_index(&target, time_bounds);

        let needs_extrap_forward = end_segment_idx == time_bounds.len();
        let needs_extrap_backward = !needs_extrap_forward & (end_segment_idx == 0);

        // Check if we can fast return because there is an exact match
        if !needs_extrap_forward {
            if is_close!(time_bounds[end_segment_idx], target) {
                return Ok((SegmentOptions::OnBoundary, end_segment_idx));
            }
        }

        let needs_extrap = needs_extrap_backward | needs_extrap_forward;

        if needs_extrap & (!allow_extrapolation) {
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

    fn find_segment_index(target: &T, time_bounds: &Array1<T>) -> usize {
        let result = time_bounds
            .as_slice()
            .unwrap()
            // Have to use binary_search_by as
            .binary_search_by(|v| v.partial_cmp(&target).expect("Couldn't compare values"));

        result.unwrap_or_else(|res| res)
    }

    fn interpolate(&self, time_target: T) -> RSCMResult<V>;
}

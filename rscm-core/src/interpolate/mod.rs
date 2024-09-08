use crate::errors::{RSCMError, RSCMResult};
use is_close::is_close;
use num::Float;
use numpy::ndarray::Array1;
use std::fmt::Display;

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
    ) -> RSCMResult<(SegmentOptions, Option<usize>)> {
        let end_segment_idx = Self::find_segment_index(&target, time_bounds);

        let needs_extrap_forward = end_segment_idx == time_bounds.len();
        let needs_extrap_backward = !needs_extrap_forward & (end_segment_idx == 0);

        // Check if we can fast return because there is an exact match
        if !needs_extrap_forward {
            if is_close!(time_bounds[end_segment_idx], target) {
                return Ok((SegmentOptions::OnBoundary, Option::from(end_segment_idx)));
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
            Ok((SegmentOptions::ExtrapolateBackward, None))
        } else if needs_extrap_forward {
            Ok((SegmentOptions::ExtrapolateForward, None))
        } else {
            Ok((SegmentOptions::InSegment, Option::from(end_segment_idx)))
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

pub struct LinearSpline<'a, T, V> {
    time: &'a Array1<T>,
    y: &'a Array1<V>,
    allow_extrapolation: bool,
}

impl<'a, T, V> LinearSpline<'a, T, V> {
    pub fn new(time: &'a Array1<T>, y: &'a Array1<V>, allow_extrapolation: bool) -> Self {
        assert_eq!(time.len(), y.len() + 1);

        Self {
            time,
            y,
            allow_extrapolation,
        }
    }
}

impl<'a, T, V> Interpolate<T, V> for LinearSpline<'a, T, V>
where
    T: Float + Into<V> + Display,
    V: Float + Into<T>,
{
    fn interpolate(&self, time_target: T) -> RSCMResult<V> {
        let segment_info = self.find_segment(time_target, self.time, self.allow_extrapolation);

        let (segment_options, end_segment_idx) = match segment_info {
            Ok(info) => info,
            Err(e) => return Err(e),
        };

        if segment_options == SegmentOptions::OnBoundary {
            // Fast return
            return Ok(self.y[end_segment_idx.unwrap()]);
        }

        let (time1, time2, y1, y2) = match segment_options {
            SegmentOptions::ExtrapolateBackward => {
                // Use first two points
                let time1 = self.time[0];
                let y1 = self.y[0];

                let time2 = self.time[1];
                let y2 = self.y[1];

                (time1, time2, y1, y2)
            }
            SegmentOptions::ExtrapolateForward => {
                // Use first two points
                let time1 = self.time[0];
                let y1 = self.y[0];

                let time2 = self.time[1];
                let y2 = self.y[1];

                (time1, time2, y1, y2)
            }
            SegmentOptions::InSegment => {
                let end_segment_idx = end_segment_idx.unwrap();

                // Use points surrounding time_target
                let time1 = self.time[end_segment_idx - 1];
                let y1 = self.y[end_segment_idx - 1];

                let time2 = self.time[end_segment_idx];
                let y2 = self.y[end_segment_idx];

                (time1, time2, y1, y2)
            }

            _ => {
                panic!("All other options handled")
            }
        };

        let time1: V = time1.into();
        let time2: V = time2.into();
        let time_target: V = time_target.into();

        let m = (y2 - y1) / (time2 - time1);

        Ok(m * (time_target - time1) + y1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use numpy::array;
    use std::iter::zip;

    #[test]
    fn test_linear() {
        let time = array![0.0, 0.5, 1.0, 1.5];
        let y = array![5.0, 8.0, 9.0];

        let target = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let exps = vec![5.0, 6.5, 8.0, 8.5, 9.0];

        let interpolator = LinearSpline::new(&time, &y, false);

        zip(target.into_iter(), exps.into_iter()).for_each(|(t, e)| {
            println!("target={}, expected={}", t, e);
            assert!(is_close!(interpolator.interpolate(t).unwrap(), e));
        })
    }

    #[test]
    fn test_linear_extrapolation_error() {
        let time = array![0.0, 1.0];
        let y = array![5.0];

        let target = vec![-1.0, -0.01, 1.01, 1.2];

        let interpolator = LinearSpline::new(&time, &y, false);

        target.into_iter().for_each(|t| {
            println!("target={t}");
            let res = interpolator.interpolate(t);
            assert!(res.is_err());

            let err = res.err().unwrap();
            assert!(err.to_string().starts_with("Extrapolation is not allowed"))
        })
    }
}

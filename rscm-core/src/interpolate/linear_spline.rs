use crate::errors::RSCMResult;
use crate::interpolate::{Interpolate, SegmentOptions};
use num::Float;
use numpy::ndarray::Array1;
use std::cmp::min;
use std::fmt::Display;

/// LinearSpline 1D interpolation
///
/// The interpolated value is
/// derived from a linear interpolation of the two points
/// to either side of time_target.
///
/// The resulting curve is therefore only zero-order continuous.
pub struct Interp1dLinearSpline<'a, T, V> {
    time: &'a Array1<T>,
    y: &'a Array1<V>,
    allow_extrapolation: bool,
}

impl<'a, T, V> Interp1dLinearSpline<'a, T, V> {
    pub fn new(time: &'a Array1<T>, y: &'a Array1<V>, allow_extrapolation: bool) -> Self {
        assert_eq!(time.len(), y.len() + 1);

        Self {
            time,
            y,
            allow_extrapolation,
        }
    }
}

impl<'a, T, V> Interpolate<T, V> for Interp1dLinearSpline<'a, T, V>
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
        // Clip the index to exclude the last bound
        let end_segment_idx = min(end_segment_idx, self.y.len() - 1);

        if segment_options == SegmentOptions::OnBoundary {
            // Fast return
            return Ok(self.y[end_segment_idx]);
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
                // Use last two points (excludes the influence of the bound of the last value
                let time1 = self.time[self.y.len() - 2];
                let y1 = self.y[self.y.len() - 2];

                let time2 = self.time[self.y.len() - 1];
                let y2 = self.y[self.y.len() - 1];

                (time1, time2, y1, y2)
            }
            SegmentOptions::InSegment | SegmentOptions::OnBoundary => {
                // Use points surrounding time_target
                let time1 = self.time[end_segment_idx - 1];
                let y1 = self.y[end_segment_idx - 1];

                let time2 = self.time[end_segment_idx];
                let y2 = self.y[end_segment_idx];

                (time1, time2, y1, y2)
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
    use is_close::is_close;
    use numpy::array;
    use std::iter::zip;

    #[test]
    fn test_linear() {
        let time = array![0.0, 0.5, 1.0, 1.5];
        let y = array![5.0, 8.0, 9.0];

        let target = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let exps = vec![5.0, 6.5, 8.0, 8.5, 9.0];

        let interpolator = Interp1dLinearSpline::new(&time, &y, false);

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

        let interpolator = Interp1dLinearSpline::new(&time, &y, false);

        target.into_iter().for_each(|t| {
            println!("target={t}");
            let res = interpolator.interpolate(t);
            assert!(res.is_err());

            let err = res.err().unwrap();
            assert!(err.to_string().starts_with("Extrapolation is not allowed"))
        })
    }

    #[test]
    fn test_linear_extrapolation() {
        let time = array![0.0, 0.5, 1.0, 1.5];
        let y = array![5.0, 8.0, 9.0];

        let target = vec![-0.5, -0.25, 0.45, 1.5, 2.0];
        let exps = vec![2.0, 3.5, 7.7, 10.0, 11.0];

        let interpolator = Interp1dLinearSpline::new(&time, &y, true);

        zip(target.into_iter(), exps.into_iter()).for_each(|(t, e)| {
            println!("target={}, expected={}", t, e);
            assert!(is_close!(interpolator.interpolate(t).unwrap(), e));
        })
    }
}

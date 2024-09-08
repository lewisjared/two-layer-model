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
    ) -> (SegmentOptions, Option<usize>) {
        let end_segment_idx = time_bounds
            .as_slice()
            .unwrap()
            // Have to use binary_search_by as
            .binary_search_by(|v| v.partial_cmp(&target).expect("Couldn't compare values"))
            .unwrap();

        let needs_extrap_forward = (end_segment_idx == time_bounds.len());
        let needs_extrap_backward = !needs_extrap_forward & (end_segment_idx == 0);

        // Check if we can fast return because there is an exact match
        if (!needs_extrap_forward) {
            if is_close!(time_bounds[end_segment_idx], target) {
                return (SegmentOptions::OnBoundary, Option::from(end_segment_idx));
            }
        }

        let needs_extrap = needs_extrap_backward | needs_extrap_forward;

        if (needs_extrap & (!allow_extrapolation)) {
            if (needs_extrap_backward) {
                println!(
                    "Extrapolation is not allowed and time_target is
                before the start of the interpolation range.
                time_target={}
                start of interpolation range={}",
                    target, time_bounds[0]
                );
            } else {
                println!(
                    "Extrapolation is not allowed and time_target is after the end of the interpolation range.
                time_target={}
                end of interpolation range= {}", target,  time_bounds[time_bounds.len()-1]
                )
            }
            panic!("Extrapolation is not allowed")
        }
        if needs_extrap_backward {
            (SegmentOptions::ExtrapolateBackward, None)
        } else if needs_extrap_forward {
            (SegmentOptions::ExtrapolateForward, None)
        } else {
            (SegmentOptions::InSegment, Option::from(end_segment_idx))
        }
    }

    fn interpolate(&self, time_target: T) -> V;
}

pub struct LinearSpline<'a, T, V> {
    time: &'a Array1<T>,
    y: &'a Array1<V>,
    allow_extrapolation: bool,
}

impl<'a, T, V> Interpolate<T, V> for LinearSpline<'a, T, V>
where
    T: Float + Into<V> + Display,
    V: Float + Into<T>,
{
    fn interpolate(&self, time_target: T) -> V {
        let (segment_options, end_segment_idx) =
            self.find_segment(time_target, self.time, self.allow_extrapolation);

        if (segment_options == SegmentOptions::OnBoundary) {
            // Fast return
            return self.y[end_segment_idx.unwrap()];
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

        m * (time_target - time1) + y1
    }
}

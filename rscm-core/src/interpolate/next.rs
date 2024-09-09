use ndarray_interp::interp1d::{Interp1D, Interp1DStrategy, Interp1DStrategyBuilder};
use ndarray_interp::{BuilderError, InterpolateError};
use std::{fmt::Debug, ops::Sub};

use num_traits::{Num, NumCast};
use numpy::ndarray::{ArrayBase, ArrayViewMut, Data, Dimension, Ix1, RemoveAxis, Zip};

/// Next-value 1D interpolation
///
/// The interpolated value is always equal to the next value in the array
/// from which to interpolate.
///
/// This can be confusing to think about.
///
/// At the boundaries (i.e time(i)) we return values(i).
/// For other values of time_target between time(i) and time(i + 1),
/// we always take y(i + 1) (i.e. we always take the 'next' value).
/// As a result,
/// y_target = y(i + 1) for time(i) < time_target <= time(i + 1)
///
/// If helpful, we have drawn a picture of how this works below.
/// Symbols:
/// - time: y-value selected for this time-value
/// - i: closed (i.e. inclusive) boundary
/// - o: open (i.e. exclusive) boundary
///
/// y(4):                                    oxxxxxxxxxxxxxxxxxxxxxxxxxx
/// y(3):                        oxxxxxxxxxxxi
/// y(2):            oxxxxxxxxxxxi
/// y(1): xxxxxxxxxxxi
///       -----------|-----------|-----------|-----------|--------------
///               time(1)     time(2)     time(3)     time(4)
///
/// One other way to think about this is
/// that the y-values are shifted to the left compared to the time-values.
/// As a result, y(1) is only used for (backward) extrapolation,
/// it isn't actually used in the interpolation domain at all.
#[derive(Debug)]
pub struct Next {
    extrapolate: bool,
}

impl<Sd, Sx, D> Interp1DStrategyBuilder<Sd, Sx, D> for Next
where
    Sd: Data,
    Sd::Elem: Num + PartialOrd + NumCast + Copy + Debug + Sub + Send,
    Sx: Data<Elem = Sd::Elem>,
    D: Dimension + RemoveAxis,
{
    const MINIMUM_DATA_LENGHT: usize = 2;
    type FinishedStrat = Next;
    fn build<Sx2>(
        self,
        _x: &ArrayBase<Sx2, Ix1>,
        _data: &ArrayBase<Sd, D>,
    ) -> Result<Self::FinishedStrat, BuilderError>
    where
        Sx2: Data<Elem = Sd::Elem>,
    {
        Ok(self)
    }
}

impl<Sd, Sx, D> Interp1DStrategy<Sd, Sx, D> for Next
where
    Sd: Data,
    Sd::Elem: Num + PartialOrd + NumCast + Copy + Debug + Sub + Send,
    Sx: Data<Elem = Sd::Elem>,
    D: Dimension + RemoveAxis,
{
    fn interp_into(
        &self,
        interpolator: &Interp1D<Sd, Sx, D, Self>,
        target: ArrayViewMut<'_, <Sd>::Elem, <D as Dimension>::Smaller>,
        x: Sx::Elem,
    ) -> Result<(), InterpolateError> {
        let this = interpolator;
        if !self.extrapolate && !this.is_in_range(x) {
            return Err(InterpolateError::OutOfBounds(format!(
                "x = {x:#?} is not in range",
            )));
        }

        // find the relevant index
        let idx = this.get_index_left_of(x);

        // lookup the data
        let (x1, y1) = this.index_point(idx);
        let (_, y2) = this.index_point(idx + 1);

        // do interpolation
        Zip::from(y1).and(y2).and(target).for_each(|&y1, &y2, t| {
            *t = match x1 < x {
                true => y1,
                false => y2,
            };
        });
        Ok(())
    }
}

impl Next {
    /// create a linear interpolation strategy
    pub fn new() -> Self {
        Self { extrapolate: false }
    }

    /// set the extrapolate property, default is `false`
    pub fn extrapolate(mut self, extrapolate: bool) -> Self {
        self.extrapolate = extrapolate;
        self
    }

    /// linearly interpolate/extrapolate between two points
    pub(crate) fn calc_frac<T>((x1, y1): (T, T), (x2, y2): (T, T), x: T) -> T
    where
        T: Num + Copy,
    {
        let b = y1;
        let m = (y2 - y1) / (x2 - x1);
        m * (x - x1) + b
    }
}

impl Default for Next {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use is_close::is_close;
    use numpy::array;
    use std::iter::zip;

    #[test]
    fn test_next() {
        let time = array![0.0, 0.5, 1.0, 1.5];
        let y = array![5.0, 8.0, 9.0];

        let target = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let exps = vec![5.0, 8.0, 8.0, 9.0, 9.0];

        let interpolator = Interp1D::new_unchecked(time, y, Next::new());

        zip(target.into_iter(), exps.into_iter()).for_each(|(t, e)| {
            println!("target={}, expected={}", t, e);
            assert!(is_close!(interpolator.interp_scalar(t).unwrap(), e));
        })
    }

    #[test]
    fn test_next_extrapolation_error() {
        let time = array![0.0, 1.0];
        let y = array![5.0];

        let target = vec![-1.0, -0.01, 1.01, 1.2];

        let interpolator = Interp1D::new_unchecked(time, y, Next::new());

        target.into_iter().for_each(|t| {
            println!("target={t}");
            let res = interpolator.interp_scalar(t);
            assert!(res.is_err());

            let err = res.err().unwrap();
            assert!(err.to_string().starts_with("Extrapolation is not allowed"))
        })
    }

    #[test]
    fn test_next_extrapolation() {
        let time = array![0.0, 0.5, 1.0, 1.5];
        let y = array![5.0, 8.0, 9.0];

        let target = vec![-1.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.2];
        let exps = vec![5.0, 5.0, 8.0, 8.0, 9.0, 9.0, 9.0];

        let interpolator = Interp1D::new_unchecked(time, y, Next::new().extrapolate(true));

        zip(target.into_iter(), exps.into_iter()).for_each(|(t, e)| {
            let value = interpolator.interp_scalar(t).unwrap();
            println!("target={}, expected={} found={}", t, e, value);
            assert!(is_close!(value, e));
        })
    }
}

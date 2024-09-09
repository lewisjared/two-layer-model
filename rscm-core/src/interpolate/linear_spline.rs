#[cfg(test)]
mod tests {
    use is_close::is_close;
    use ndarray_interp::interp1d::{Interp1D, Linear};
    use numpy::array;
    use std::iter::zip;

    #[test]
    fn test_linear() {
        let time = array![0.0, 0.5, 1.0, 1.5];
        let y = array![5.0, 8.0, 9.0];

        let target = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let exps = vec![5.0, 6.5, 8.0, 8.5, 9.0];

        let interpolator = Interp1D::new_unchecked(time, y, Linear::new());

        zip(target.into_iter(), exps.into_iter()).for_each(|(t, e)| {
            println!("target={}, expected={}", t, e);
            assert!(is_close!(interpolator.interp_scalar(t).unwrap(), e));
        })
    }

    #[test]
    fn test_linear_extrapolation_error() {
        let time = array![0.0, 1.0];
        let y = array![5.0];

        let target = vec![-1.0, -0.01, 1.01, 1.2];

        let interpolator = Interp1D::new_unchecked(time, y, Linear::new());

        target.into_iter().for_each(|t| {
            println!("target={t}");
            let res = interpolator.interp_scalar(t);
            assert!(res.is_err());
        })
    }

    #[test]
    fn test_linear_extrapolation() {
        let time = array![0.0, 0.5, 1.0, 1.5];
        let y = array![5.0, 8.0, 9.0];

        let target = vec![-0.5, -0.25, 0.45, 1.5, 2.0];
        let exps = vec![2.0, 3.5, 7.7, 10.0, 11.0];

        let interpolator = Interp1D::new_unchecked(time, y, Linear::new().extrapolate(true));

        zip(target.into_iter(), exps.into_iter()).for_each(|(t, e)| {
            println!("target={}, expected={}", t, e);
            assert!(is_close!(interpolator.interp_scalar(t).unwrap(), e));
        })
    }
}

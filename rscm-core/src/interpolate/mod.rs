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
use crate::errors::RSCMResult;
use num::Float;
use numpy::ndarray::Array1;

use strategies::{Interp1DStrategy, InterpolationStrategy};

pub mod strategies;

/// Interpolator
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolate::strategies::next::Interp1DNext;
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

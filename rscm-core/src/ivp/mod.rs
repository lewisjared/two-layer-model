use crate::component::{Component, InputState};
use crate::timeseries::Time;
use nalgebra::allocator::Allocator;
use nalgebra::{DefaultAllocator, Dim};
use ode_solvers::dop_shared::{FloatNumber, IntegrationError, Stats};
use ode_solvers::*;

/// this module uses [lifetime elision](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-elision)
/// which is terribly confusing,
/// but I couldn't see how to handle correctly handle the lifetime of component in IVPSolver.
/// I didn't want IVPSolver to take ownership of the component,
/// but I needed to ensure that the component outlived the IVPSolver.

pub trait IVP<T, S> {
    fn calculate_dy_dt(&self, t: T, input_state: &InputState, y: &S, dy_dt: &mut S);
}

pub struct IVPSolver<C, S> {
    component: Box<C>,
    y0: S,
    input_state: InputState,
}

impl<T, D: Dim, C> System<T, OVector<T, D>> for IVPSolver<C, OVector<T, D>>
where
    T: FloatNumber,
    C: IVP<T, OVector<T, D>>,
    OVector<T, D>: std::ops::Mul<T, Output = OVector<T, D>>,
    DefaultAllocator: Allocator<T, D>,
{
    fn system(&self, t: T, y: &OVector<T, D>, dy: &mut OVector<T, D>) {
        self.component.calculate_dy_dt(t, &self.input_state, y, dy)
    }
}

impl<T, D: Dim, C> IVPSolver<C, OVector<T, D>>
where
    T: FloatNumber,
    C: IVP<T, OVector<T, D>>,
    OVector<T, D>: std::ops::Mul<T, Output = OVector<T, D>>,
    DefaultAllocator: Allocator<T, D>,
{
    pub fn new(component: Box<C>, input_state: InputState, y0: OVector<T, D>) -> Self {
        Self {
            component,
            y0,
            input_state,
        }
    }
    pub fn integrate(self, t0: T, t1: T, step: T) -> Result<Stats, IntegrationError> {
        let y0 = self.y0.clone();
        let mut stepper = Rk4::new(self, t0, y0, t1, step);
        stepper.integrate()
    }
}

mod rk4;


use crate::component::{Component, InputState};
use crate::timeseries::Time;
// use rk4::{Rk4, System};
use ode_solvers::*;

/// this module uses [lifetime elision](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-elision)
/// which is terribly confusing,
/// but I couldn't see how to handle correctly handle the lifetime of component in IVPSolver.
/// I didn't want IVPSolver to take ownership of the component,
/// but I needed to ensure that the component outlived the IVPSolver.

pub trait IVP<ModelState> {

    fn y0(&self) -> ModelState;

    fn calculate_dy_dt(
        &self,
        t: Time,
        input_state: &InputState,
        y: &ModelState,
        dy_dt: &mut ModelState,
    );

}

impl<ModelState> System<Time, ModelState> for dyn IVP<ModelState> {
    fn system(&self, t: Time, y: &ModelState, dy: &mut ModelState) {
        self.calculate_dy_dt(t, &self.input_state, y, dy)
    }
}



pub struct IVPSolver<'a, T>
where
    T: Component<T>,
{
    component: &'a T,
    input_state: InputState,
}

impl<'a, T, ModelState> System<Time, ModelState> for IVPSolver<'a, T>
where
    T: Component<T> + IVP<ModelState>,
{
    fn system(&self, t: Time, y: &ModelState, dy: &mut ModelState) {
        self.component.calculate_dy_dt(t, &self.input_state, y, dy)
    }
}

impl<'a, T, ModelState> IVPSolver<'a, T>
where
    T: Component<T> + IVP<ModelState>,
{
    pub fn new(component: &'a T, input_state: InputState) -> Self {
        Self {
            component,
            input_state,
        }
    }

    pub fn integrate(&self, t0: Time, t1: Time, y0: ModelState) -> Result<Stats, IntegrationError> {
        let solver = Rk4::new(&)
        let t0 = self.input_state.time();
        let mut stepper = Rk4::new(self, t0, y0, t1, step);
        stepper.integrate()
    }
}

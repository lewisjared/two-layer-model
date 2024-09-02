mod model;
pub mod python;

extern crate uom;

use numpy::ndarray::array;
use ode_solvers::dop_shared::{IntegrationError, Stats};
use ode_solvers::*;

use rscm_core::timeseries::Timeseries;
use std::sync::Arc;

// Define some types that are used by OdeSolvers
type ModelState = Vector3<f32>;
type Time = f32;

#[derive(Clone)]
pub struct TwoLayerModelParameters {
    lambda0: f32,
    a: f32,
    efficacy: f32,
    eta: f32,
    heat_capacity_surface: f32,
    heat_capacity_deep: f32,
}

#[derive(Clone)]
pub struct TwoLayerModelState {
    erf: Timeseries,
}

#[derive(Clone)]
pub struct TwoLayerModel {
    parameters: Arc<TwoLayerModelParameters>,
    state: Option<Arc<TwoLayerModelState>>,
}

impl TwoLayerModel {
    pub fn new(parameters: Arc<TwoLayerModelParameters>, state: Arc<TwoLayerModelState>) -> Self {
        Self {
            parameters,
            state: Option::from(state),
        }
    }
    pub fn from_parameters(parameters: Arc<TwoLayerModelParameters>) -> Self {
        Self {
            parameters,
            state: None,
        }
    }

    fn with_state(mut self, state: Arc<TwoLayerModelState>) -> Self {
        self.state = Option::from(state);
        self
    }
    fn solve(&self) -> Result<Stats, IntegrationError> {
        let y0 = ModelState::new(0.0, 0.0, 0.0);

        // Create the solver
        let mut stepper = Rk4::new(self.clone(), 1848.0, y0, 1900.0, 1.0);
        stepper.integrate()
    }
}

// Create the set of ODEs to represent the two layer model
impl System<Time, ModelState> for TwoLayerModel {
    fn system(&self, t: Time, y: &ModelState, dy: &mut ModelState) {
        let temperature_surface = y[0];
        let temperature_deep = y[1];
        let erf = self.state.as_ref().unwrap().erf.at_time(t).unwrap();

        let temperature_difference = temperature_surface - temperature_deep;

        let lambda_eff = self.parameters.lambda0 - self.parameters.a * temperature_surface;
        let heat_exchange_surface =
            self.parameters.efficacy * self.parameters.eta * temperature_difference;
        let dtemperature_surface_dt =
            (erf - lambda_eff * temperature_surface - heat_exchange_surface)
                / self.parameters.heat_capacity_surface;

        let heat_exchange_deep = self.parameters.eta * temperature_difference;
        let dtemperature_deep_dt = heat_exchange_deep / self.parameters.heat_capacity_deep;

        dy[0] = dtemperature_surface_dt;
        dy[1] = dtemperature_deep_dt;
        dy[2] = self.parameters.heat_capacity_surface * dtemperature_surface_dt
            + self.parameters.heat_capacity_deep * dtemperature_deep_dt;
    }
}

pub fn solve_tlm() {
    // Initialise the model
    let model = TwoLayerModel::from_parameters(Arc::new(TwoLayerModelParameters {
        lambda0: 0.5,
        a: 0.01,
        efficacy: 0.5,
        eta: 0.1,
        heat_capacity_surface: 1.0,
        heat_capacity_deep: 100.0,
    }));
    let erf = Timeseries::from_values(
        array![0.0, 0.0, 2.0, 2.0],
        array![1848.0, 1849.0, 1850.0, 1900.0],
    );
    let state = Arc::new(TwoLayerModelState { erf });

    // Create the solver
    let res = model.with_state(state).solve();

    // Handle result
    match res {
        Ok(stats) => {
            println!("Stats: {}", stats)

            // Do something with the output...
            // let path = Path::new("./outputs/kepler_orbit_dopri5.dat");
            // save(stepper.x_out(), stepper.y_out(), path);
            // println!("Results saved in: {:?}", path);
        }
        Err(_) => println!("An error occured."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        solve_tlm();
    }
}

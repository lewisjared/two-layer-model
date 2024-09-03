use numpy::ndarray::array;
use ode_solvers::dop_shared::{IntegrationError, Stats};
use ode_solvers::*;

use rscm_core::component::{Component, InputState, OutputState, State};
use rscm_core::ivp::{IVP, IVPSolver}
use rscm_core::timeseries::{Time, Timeseries};
use rscm_core::timeseries_collection::{TimeseriesCollection, VariableType};
use std::sync::Arc;

// Define some types that are used by OdeSolvers
type ModelState = Vector3<f32>;

#[derive(Clone, Debug)]
pub struct TwoLayerModelParameters {
    lambda0: f32,
    a: f32,
    efficacy: f32,
    eta: f32,
    heat_capacity_surface: f32,
    heat_capacity_deep: f32,
}

pub struct TwoLayerComponent {
    parameters: TwoLayerModelParameters,
}

impl Component<TwoLayerModelParameters> for TwoLayerComponent {
    fn from_parameters(parameters: TwoLayerModelParameters) -> Self {
        Self { parameters }
    }

    fn inputs() -> Vec<String> {
        vec!["erf".to_string()]
    }

    fn outputs() -> Vec<String> {
        vec!["Surface Temperature".to_string()]
    }

    fn extract_state(&self, collection: &TimeseriesCollection, t_current: Time) -> InputState {
        InputState::new(vec![collection.get_timeseries("erf").unwrap().at_time(t_current).unwrap()], TwoLayerComponent::inputs())
    }

    fn solve(
        &self,
        t_current: Time,
        t_next: Time,
        input_state: InputState,
    ) -> Result<OutputState, String> {
        let erf = input_state.get("erf");

        let y0 = ModelState::new(0.0, 0.0, 0.0);

        let a = Arc::from(self);
        let solver = IVPSolver::new(&self, input_state);
        let res = solver.integrate();

        // Create the solver
        let mut stepper = Rk4::new(self.clone(), t_current, y0, t_next, 1.0);
        let res = stepper.integrate();

        println!("Solving {:?} with state: {:?}", self, input_state);

        Ok(OutputState::new(
            vec![erf * self.parameters.lambda0],
            TwoLayerComponent::outputs(),
        ))
    }
}

// Create the set of ODEs to represent the two layer model
impl IVP<ModelState> for TwoLayerComponent {
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
    let model = TwoLayerComponent::from_parameters(TwoLayerModelParameters {
        lambda0: 0.5,
        a: 0.01,
        efficacy: 0.5,
        eta: 0.1,
        heat_capacity_surface: 1.0,
        heat_capacity_deep: 100.0,
    });

    let mut ts_collection = TimeseriesCollection::new();
    ts_collection.add_timeseries(
        "erf".to_string(),
        Timeseries::from_values(
            array![0.0, 0.0, 2.0, 2.0],
            array![1848.0, 1849.0, 1850.0, 1900.0],
        ),
        VariableType::Endogenous
    );

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

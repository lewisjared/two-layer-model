use numpy::ndarray::array;
use ode_solvers::*;

use rscm_core::component::{Component, InputState, OutputState, State};
use rscm_core::ivp::{IVPSolver, IVP};
use rscm_core::timeseries::{Time, Timeseries};
use rscm_core::timeseries_collection::{TimeseriesCollection, VariableType};

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

#[derive(Debug)]
pub struct TwoLayerComponent {
    parameters: TwoLayerModelParameters,
}

// Create the set of ODEs to represent the two layer model
impl IVP<Time, ModelState> for TwoLayerComponent {
    fn calculate_dy_dt(
        &self,
        _t: Time,
        input_state: &InputState,
        y: &ModelState,
        dy_dt: &mut ModelState,
    ) {
        let temperature_surface = y[0];
        let temperature_deep = y[1];
        let erf = input_state.get("erf");

        let temperature_difference = temperature_surface - temperature_deep;

        let lambda_eff = self.parameters.lambda0 - self.parameters.a * temperature_surface;
        let heat_exchange_surface =
            self.parameters.efficacy * self.parameters.eta * temperature_difference;
        let dtemperature_surface_dt =
            (erf - lambda_eff * temperature_surface - heat_exchange_surface)
                / self.parameters.heat_capacity_surface;

        let heat_exchange_deep = self.parameters.eta * temperature_difference;
        let dtemperature_deep_dt = heat_exchange_deep / self.parameters.heat_capacity_deep;

        dy_dt[0] = dtemperature_surface_dt;
        dy_dt[1] = dtemperature_deep_dt;
        dy_dt[2] = self.parameters.heat_capacity_surface * dtemperature_surface_dt
            + self.parameters.heat_capacity_deep * dtemperature_deep_dt;
    }
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
        InputState::new(
            vec![collection
                .get_timeseries("erf")
                .unwrap()
                .at_time(t_current)
                .unwrap()],
            TwoLayerComponent::inputs(),
        )
    }

    fn solve(
        &self,
        t_current: Time,
        t_next: Time,
        input_state: &InputState,
    ) -> Result<OutputState, String> {
        let erf = input_state.get("erf");

        let y0 = ModelState::new(0.0, 0.0, 0.0);

        let s = Box::new(self.to_owned());
        let mut solver = IVPSolver::new(Box::new(self.to_owned()), input_state.clone(), y0)
            .integrate(t_current, t_next, 0.1);
        let res = solver;

        // Create the solver
        println!("Solving {:?} with state: {:?}", self, input_state);

        Ok(OutputState::new(
            vec![erf * self.parameters.lambda0],
            TwoLayerComponent::outputs(),
        ))
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
        VariableType::Endogenous,
    );

    let input_state = model.extract_state(&ts_collection, 1848.0);
    println!("Input: {:?}", input_state);

    // Create the solver
    let res = model.solve(1848.0, 1849.0, &input_state);

    // Handle result
    match res {
        Ok(output_state) => {
            println!("Output: {:?}", output_state)
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

use numpy::ndarray::array;
use ode_solvers::*;
use std::sync::Arc;

use rscm_core::component::{
    Component, InputState, OutputState, RequirementDefinition, RequirementType, State,
};
use rscm_core::ivp::{IVPBuilder, IVP};
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

#[derive(Debug, Clone)]
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

impl TwoLayerComponent {
    fn from_parameters(parameters: TwoLayerModelParameters) -> Self {
        Self { parameters }
    }
}

impl Component for TwoLayerComponent {
    fn definitions(&self) -> Vec<RequirementDefinition> {
        vec![
            RequirementDefinition::new("erf", "W/m^2", RequirementType::Input),
            RequirementDefinition::new("Surface Temperature", "K", RequirementType::Output),
        ]
    }

    fn extract_state(&self, collection: &TimeseriesCollection, t_current: Time) -> InputState {
        InputState::from_vectors(
            vec![collection
                .get_timeseries_by_name("erf")
                .unwrap()
                .at_time(t_current)
                .unwrap()],
            self.input_names(),
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

        let solver = IVPBuilder::new(Arc::new(self.to_owned()), input_state.clone(), y0);
        println!("Solving {:?} with state: {:?}", self, input_state);

        let mut solver = solver.to_rk4(t_current, t_next, 0.1);
        let stats = solver.integrate().expect("Failed solving");

        let results = solver.results();

        println!("Stats {:?}", stats);
        println!("Results {:?}", results);

        // Create the solver

        Ok(OutputState::from_vectors(
            vec![erf * self.parameters.lambda0],
            self.output_names(),
        ))
    }
}

pub fn solve_tlm() -> Result<OutputState, String> {
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
            array![1.0, 1.5, 2.0, 2.0],
            array![1848.0, 1849.0, 1850.0, 1900.0],
        ),
        VariableType::Endogenous,
    );

    let input_state = model.extract_state(&ts_collection, 1848.0);
    println!("Input: {:?}", input_state);

    // Create the solver
    let output_state = model.solve(1848.0, 1849.0, &input_state);

    println!("Output: {:?}", output_state);
    output_state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let res = solve_tlm().unwrap();
    }
}

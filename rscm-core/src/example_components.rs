use crate::component::{
    Component, InputState, OutputState, ParameterDefinition, ParameterType, State,
};
use crate::timeseries::Time;
use crate::timeseries_collection::TimeseriesCollection;

#[derive(Debug)]
pub(crate) struct TestComponentParameters {
    pub p: f32,
}

#[derive(Debug)]
pub(crate) struct TestComponent {
    parameters: TestComponentParameters,
}

impl TestComponent {
    pub fn from_parameters(parameters: TestComponentParameters) -> Self {
        Self { parameters }
    }
}

impl Component for TestComponent {
    fn definitions(&self) -> Vec<ParameterDefinition> {
        vec![
            ParameterDefinition::new("Emissions|CO2", "GtCO2", ParameterType::Input),
            ParameterDefinition::new("Concentrations|CO2", "ppm", ParameterType::Output),
        ]
    }

    fn extract_state(&self, _collection: &TimeseriesCollection, _t_current: Time) -> InputState {
        InputState::new(vec![1.3], self.input_names())
    }
    fn solve(
        &self,
        _t_current: Time,
        _t_next: Time,
        input_state: &InputState,
    ) -> Result<OutputState, String> {
        let emission_co2 = input_state.get("Emissions|CO2");

        println!("Solving {:?} with state: {:?}", self, input_state);

        Ok(OutputState::new(
            vec![emission_co2 * self.parameters.p],
            self.output_names(),
        ))
    }
}

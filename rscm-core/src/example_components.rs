#![allow(dead_code)]

use crate::component::{
    Component, InputState, OutputState, RequirementDefinition, RequirementType, State,
};
use crate::errors::RSCMResult;
use crate::timeseries::{FloatValue, Time};
use crate::timeseries_collection::TimeseriesCollection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TestComponentParameters {
    pub p: FloatValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TestComponent {
    parameters: TestComponentParameters,
}

impl TestComponent {
    pub fn from_parameters(parameters: TestComponentParameters) -> Self {
        Self { parameters }
    }
}

#[typetag::serde]
impl Component for TestComponent {
    fn definitions(&self) -> Vec<RequirementDefinition> {
        vec![
            RequirementDefinition::new("Emissions|CO2", "GtCO2", RequirementType::Input),
            RequirementDefinition::new("Concentrations|CO2", "ppm", RequirementType::Output),
        ]
    }

    fn extract_state(&self, _collection: &TimeseriesCollection, _t_current: Time) -> InputState {
        InputState::from_vectors(vec![1.3], self.input_names())
    }
    fn solve(
        &self,
        _t_current: Time,
        _t_next: Time,
        input_state: &InputState,
    ) -> RSCMResult<OutputState> {
        let emission_co2 = input_state.get("Emissions|CO2");

        println!("Solving {:?} with state: {:?}", self, input_state);

        Ok(OutputState::from_vectors(
            vec![emission_co2 * self.parameters.p],
            self.output_names(),
        ))
    }
}

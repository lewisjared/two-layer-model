use rscm_core::component::{
    Component, InputState, OutputState, ParameterDefinition, RequirementType, State,
};
use rscm_core::timeseries::Time;

#[derive(Debug, Clone)]
pub struct CO2ERFParameters {
    /// ERF due to a doubling of atmospheric CO_2 concentrations
    /// unit: W / m^2
    pub erf_2xco2: f32,
    /// Pre-industrial atmospheric CO_2 concentration
    /// unit: ppm
    pub conc_pi: f32,
}

#[derive(Debug, Clone)]
/// CO2 effective radiative forcing (ERF) calculations
pub struct CO2ERF {
    parameters: CO2ERFParameters,
}

impl CO2ERF {
    pub fn from_parameters(parameters: CO2ERFParameters) -> Self {
        Self { parameters }
    }
}

impl Component for CO2ERF {
    fn definitions(&self) -> Vec<ParameterDefinition> {
        vec![
            ParameterDefinition::new(
                "Atmospheric Concentration|CO2",
                "ppm",
                RequirementType::Input,
            ),
            ParameterDefinition::new(
                "Effective Radiative Forcing|CO2",
                "W / m^2",
                RequirementType::Output,
            ),
        ]
    }

    fn solve(
        &self,
        _t_current: Time,
        _t_next: Time,
        input_state: &InputState,
    ) -> Result<OutputState, String> {
        let erf = self.parameters.erf_2xco2 / 2.0f32.log10()
            * (1.0
                + (input_state.get("Atmospheric Concentration|CO2") - self.parameters.conc_pi)
                    / self.parameters.conc_pi)
                .log10();

        Ok(OutputState::new(vec![erf], self.output_names()))
    }
}

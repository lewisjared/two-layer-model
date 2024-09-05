use nalgebra::Vector3;
use rscm_core::component::{
    Component, InputState, OutputState, ParameterDefinition, ParameterType, State,
};
use rscm_core::ivp::{get_last_step, IVPBuilder, IVP};
use rscm_core::timeseries::Time;
use rscm_core::timeseries_collection::TimeseriesCollection;
use std::collections::HashMap;
use std::sync::Arc;

const GTC_PER_PPM: f32 = 2.13;
type ModelState = Vector3<f32>;

#[derive(Debug, Clone)]
pub struct CarbonCycleParameters {
    /// Timescale of the box's response
    /// unit: yr
    pub tau: f32,
    /// Pre-industrial atmospheric CO_2 concentration
    /// unit: ppm
    pub conc_pi: f32,
    /// Sensitivity of lifetime to changes in global-mean temperature
    /// unit: 1 / K
    pub alpha_temperature: f32,
}

#[derive(Debug, Clone)]
pub struct CarbonCycleComponent {
    parameters: CarbonCycleParameters,
}

impl CarbonCycleComponent {
    pub fn from_parameters(parameters: CarbonCycleParameters) -> Self {
        Self { parameters }
    }
}

impl Component for CarbonCycleComponent {
    fn definitions(&self) -> Vec<ParameterDefinition> {
        vec![
            ParameterDefinition::new(
                "Emissions|CO2|Anthropogenic",
                "GtC / yr",
                ParameterType::Input,
            ),
            ParameterDefinition::new("Atmospheric Concentration|CO2", "ppm", ParameterType::Input),
            ParameterDefinition::new("Surface Temperature", "K", ParameterType::Input),
            ParameterDefinition::new(
                "Atmospheric Concentration|CO2",
                "ppm",
                ParameterType::Output,
            ),
            ParameterDefinition::new("Land uptake|Cumulative", "ppm", ParameterType::Output),
            ParameterDefinition::new("Land uptake", "ppm", ParameterType::Output),
            ParameterDefinition::new("Cumulative Emissions|CO2", "ppm", ParameterType::Output),
        ]
    }

    fn extract_state(&self, collection: &TimeseriesCollection, t_current: Time) -> InputState {
        let mut state = HashMap::new();

        self.input_names().into_iter().for_each(|name| {
            let ts = collection.get_timeseries(name.as_str()).unwrap();

            state.insert(name, ts.at_time(t_current).unwrap());
        });

        InputState::from_hashmap(state, self.input_names())
    }
    fn solve(
        &self,
        t_current: Time,
        t_next: Time,
        input_state: &InputState,
    ) -> Result<OutputState, String> {
        let y0 = ModelState::new(
            input_state.get("Atmospheric Concentration|CO2"),
            input_state.get("Cumulative Land Uptake"),
            input_state.get("Cumulative Emissions|CO2"),
        );

        let solver = IVPBuilder::new(Arc::new(self.to_owned()), input_state.clone(), y0);
        println!("Solving {:?} with state: {:?}", self, input_state);

        let mut solver = solver.to_rk4(t_current, t_next, 0.1);
        let stats = solver.integrate().expect("Failed solving");

        let results = get_last_step(solver.results(), t_next);

        println!("Stats {:?}", stats);
        println!("Results {:?}", results);

        let mut output = HashMap::new();
        output.insert("Atmospheric Concentration|CO2".to_string(), results[0]);
        output.insert("Cumulative Land Uptake".to_string(), results[1]);
        output.insert("Cumulative Emissions|CO2".to_string(), results[2]);

        Ok(OutputState::from_hashmap(output, self.output_names()))
    }
}

impl IVP<Time, ModelState> for CarbonCycleComponent {
    fn calculate_dy_dt(
        &self,
        _t: Time,
        input_state: &InputState,
        y: &Vector3<f32>,
        dy_dt: &mut Vector3<f32>,
    ) {
        let emissions = input_state.get("Emissions|CO2");
        let temperature = input_state.get("Surface Temperature");
        let conc = input_state.get("Atmospheric Concentration|CO2");

        // dC / dt = E - (C - C_0) / (\tau \exp(alpha_temperature * temperature))
        let lifetime =
            self.parameters.tau * (self.parameters.alpha_temperature * temperature).exp();
        let uptake = (conc - self.parameters.conc_pi) / lifetime; // ppm / yr

        dy_dt[0] = emissions / GTC_PER_PPM - uptake; // ppm / yr
        dy_dt[1] = uptake * GTC_PER_PPM; // GtC / yr
        dy_dt[2] = emissions // GtC / yr
    }
}

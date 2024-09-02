use crate::timeseries::Time;
use crate::timeseries_collection::TimeseriesCollection;

pub enum StateDirection {
    Input,
    Output,
}

pub struct State {
    pub values: Vec<f32>,
    pub names: Vec<String>,
    pub direction: StateDirection,
}

impl State {
    fn input(values: Vec<f32>, names: Vec<String>) -> Self {
        Self {
            values,
            names,
            direction: StateDirection::Input,
        }
    }

    fn output(values: Vec<f32>, names: Vec<String>) -> Self {
        Self {
            values,
            names,
            direction: StateDirection::Output,
        }
    }

    fn get(&self, name: &str) -> f32 {
        let index = self.names.iter().position(|x| x == name).unwrap();
        self.values[index]
    }
}

pub trait Component<Parameters> {
    fn from_parameters(parameters: Parameters) -> Self;

    /// Variables that are required to solve this component
    fn inputs() -> Vec<String>;

    /// Variables that are solved by this component
    ///
    /// The names of the solved variables must be unique for a given model.
    /// i.e. No two components within a model can produce the same variable names.
    /// These names can contain '|' to namespace variables to avoid collisions,
    /// for example, 'Emissions|CO2' and 'Atmospheric Concentrations|CO2'
    fn outputs() -> Vec<String>;

    /// Extract the state for the current time step
    ///
    /// The result should contain values for the current time step for all input variable
    fn extract_state(&self, collection: &TimeseriesCollection, t_current: Time) -> State;

    /// Solve the component until `t_next`
    ///
    /// The result should contain values for the current time step for all output variables
    fn solve(&self, t_current: Time, t_next: Time, input_state: State) -> Result<State, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponentParameters {
        p: f32,
    }

    struct TestComponent {
        parameters: TestComponentParameters,
    }

    impl Component<TestComponentParameters> for TestComponent {
        fn from_parameters(parameters: TestComponentParameters) -> Self {
            Self { parameters }
        }

        fn inputs() -> Vec<String> {
            vec!["Emissions|CO2".to_string()]
        }

        fn outputs() -> Vec<String> {
            vec!["Concentrations|CO2".to_string()]
        }
        fn extract_state(&self, _collection: &TimeseriesCollection, _t_current: Time) -> State {
            State::input(vec![1.3], TestComponent::inputs())
        }
        fn solve(
            &self,
            t_current: Time,
            t_next: Time,
            input_state: State,
        ) -> Result<State, String> {
            let emission_co2 = input_state.get("Emissions|CO2");

            Ok(State::output(
                vec![emission_co2 * self.parameters.p],
                TestComponent::outputs(),
            ))
        }
    }

    #[test]
    fn step() {
        let component = TestComponent::from_parameters(TestComponentParameters { p: 2.0 });

        let input_state = component.extract_state(&TimeseriesCollection::new(), 2020.0);
        let output_state = component.solve(2020.0, 2021.0, input_state).unwrap();

        assert_eq!(output_state.get("Concentrations|CO2"), 2.0 * 1.3);
    }
}

use crate::timeseries::Time;
use crate::timeseries_collection::TimeseriesCollection;

/// Generic state representation
///
/// A state is a collection of values
/// that can be used to represent the state of a system at a given time.
///
/// This is very similar to a Hashmap (with likely worse performance),
/// but provides strong type separation.
pub trait State {
    fn names(&self) -> &Vec<String>;
    fn values(&self) -> &Vec<f32>;

    fn get(&self, name: &str) -> f32 {
        let index = self.names().iter().position(|x| x == name).unwrap();
        *self.values().get(index).unwrap()
    }
}

#[derive(Debug)]
pub struct InputState {
    values: Vec<f32>,
    names: Vec<String>,
}

impl InputState {
    pub fn new(values: Vec<f32>, names: Vec<String>) -> Self {
        assert_eq!(values.len(), names.len());
        Self { values, names }
    }
}
impl State for InputState {
    fn names(&self) -> &Vec<String> {
        &self.names
    }

    fn values(&self) -> &Vec<f32> {
        &self.values
    }
}

#[derive(Debug)]
pub struct OutputState {
    values: Vec<f32>,
    names: Vec<String>,
}

impl OutputState {
    pub fn new(values: Vec<f32>, names: Vec<String>) -> Self {
        assert_eq!(values.len(), names.len());
        Self { values, names }
    }
}

impl State for OutputState {
    fn names(&self) -> &Vec<String> {
        &self.names
    }

    fn values(&self) -> &Vec<f32> {
        &self.values
    }
}

/// Component of a reduced complexity climate model
///
/// Each component encapsulates some set of physics that can be solved for a given time step.
/// Generally these components can be modelled as a set of Ordinary Differential Equations (ODEs)
/// with an input state that can be solved as an initial value problem over a given time domain.
///
/// The resulting state of a component can then be used by other components as part of a `Model`
/// or solved alone during calibration.
///
/// Each component contains:
/// * parameters: Time invariant constants used to parameterize the components physics
/// * inputs: State information required to solve the model. This come from either other
/// components as part of a coupled system or from exogenous data.
/// * outputs: Information that is solved by the component
pub trait Component<Parameters> {
    fn from_parameters(parameters: Parameters) -> Self
    where
        // Used to opt out of this method when used as an object trait
        // See https://doc.rust-lang.org/error_codes/E0038.html
        Self: Sized;

    /// Variables that are required to solve this component
    fn inputs() -> Vec<String>
    where
        Self: Sized;

    /// Variables that are solved by this component
    ///
    /// The names of the solved variables must be unique for a given model.
    /// i.e. No two components within a model can produce the same variable names.
    /// These names can contain '|' to namespace variables to avoid collisions,
    /// for example, 'Emissions|CO2' and 'Atmospheric Concentrations|CO2'
    fn outputs() -> Vec<String>
    where
        Self: Sized;

    /// Extract the input state for the current time step
    ///
    /// The result should contain values for the current time step for all input variable
    fn extract_state(&self, collection: &TimeseriesCollection, t_current: Time) -> InputState;

    /// Solve the component until `t_next`
    ///
    /// The result should contain values for the current time step for all output variables
    fn solve(
        &self,
        t_current: Time,
        t_next: Time,
        input_state: &InputState,
    ) -> Result<OutputState, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestComponentParameters {
        p: f32,
    }

    #[derive(Debug)]
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
        fn extract_state(
            &self,
            _collection: &TimeseriesCollection,
            _t_current: Time,
        ) -> InputState {
            InputState::new(vec![1.3], TestComponent::inputs())
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
                TestComponent::outputs(),
            ))
        }
    }

    #[test]
    fn solve() {
        let component = TestComponent::from_parameters(TestComponentParameters { p: 2.0 });

        let input_state = component.extract_state(&TimeseriesCollection::new(), 2020.0);
        let output_state = component.solve(2020.0, 2021.0, &input_state).unwrap();

        assert_eq!(output_state.get("Concentrations|CO2"), 2.0 * 1.3);
    }
}

use crate::timeseries::Time;
use crate::timeseries_collection::TimeseriesCollection;
use std::collections::hash_map::IntoIter;
use std::collections::HashMap;
use std::fmt::Debug;
use std::iter::zip;

/// Generic state representation
///
/// A state is a collection of values
/// that can be used to represent the state of a system at a given time.
///
/// This is very similar to a Hashmap (with likely worse performance),
/// but provides strong type separation.
pub trait State<T> {
    fn get(&self, name: &str) -> &T;
}

#[derive(Debug, Clone)]
pub struct InputState(HashMap<String, f32>);

impl InputState {
    pub fn from_vectors(values: Vec<f32>, names: Vec<String>) -> Self {
        assert_eq!(values.len(), names.len());
        let mut map = HashMap::new();
        zip(names, values).for_each(|(k, v)| {
            map.insert(k, v);
        });
        Self(map)
    }

    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn from_hashmap_and_verify(
        items: HashMap<String, f32>,
        expected_items: Vec<String>,
    ) -> Self {
        for key in expected_items {
            assert!(items.contains_key(&key))
        }
        Self::from_hashmap(items)
    }

    pub fn from_hashmap(items: HashMap<String, f32>) -> Self {
        Self(items)
    }

    pub fn has(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    /// Merge state into this state
    ///
    /// Overrides any existing values with the same name
    pub fn merge(&mut self, state: InputState) -> &mut Self {
        state.into_iter().for_each(|(key, value)| {
            self.0.insert(key, value);
        });
        self
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, f32> {
        self.0.iter()
    }

    pub fn into_iter(self) -> IntoIter<String, f32> {
        self.0.into_iter()
    }
}
impl State<f32> for InputState {
    fn get(&self, name: &str) -> &f32 {
        match self.0.get(name) {
            Some(val) => val,
            None => panic!("No state named {} found in {:?}", name, self),
        }
    }
}

pub type OutputState = InputState;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum RequirementType {
    Input,
    Output,
    InputAndOutput, // TODO: Figure out how to compose input and output together
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct RequirementDefinition {
    pub name: String,
    pub unit: String,
    pub requirement_type: RequirementType,
}

impl RequirementDefinition {
    pub fn new(name: &str, unit: &str, parameter_type: RequirementType) -> Self {
        Self {
            name: name.to_string(),
            unit: unit.to_string(),
            requirement_type: parameter_type,
        }
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

pub trait Component: Debug {
    fn definitions(&self) -> Vec<RequirementDefinition>;

    /// Variables that are required to solve this component
    fn inputs(&self) -> Vec<RequirementDefinition> {
        self.definitions()
            .iter()
            .filter(|d| {
                (d.requirement_type == RequirementType::Input)
                    | (d.requirement_type == RequirementType::InputAndOutput)
            })
            .cloned()
            .collect()
    }
    fn input_names(&self) -> Vec<String> {
        self.inputs().into_iter().map(|d| d.name).collect()
    }

    /// Variables that are solved by this component
    ///
    /// The names of the solved variables must be unique for a given model.
    /// i.e. No two components within a model can produce the same variable names.
    /// These names can contain '|' to namespace variables to avoid collisions,
    /// for example, 'Emissions|CO2' and 'Atmospheric Concentrations|CO2'
    fn outputs(&self) -> Vec<RequirementDefinition> {
        self.definitions()
            .iter()
            .filter(|d| {
                (d.requirement_type == RequirementType::Output)
                    | (d.requirement_type == RequirementType::InputAndOutput)
            })
            .cloned()
            .collect()
    }
    fn output_names(&self) -> Vec<String> {
        self.outputs().into_iter().map(|d| d.name).collect()
    }

    /// Extract the input state for the current time step
    ///
    /// The result should contain values for the current time step for all input variable
    fn extract_state(&self, collection: &TimeseriesCollection, t_current: Time) -> InputState {
        let mut state = HashMap::new();

        self.input_names().into_iter().for_each(|name| {
            let ts = collection.get_timeseries(name.as_str()).unwrap();

            state.insert(name, ts.at_time(t_current).unwrap());
        });

        InputState::from_hashmap_and_verify(state, self.input_names())
    }

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
    use crate::example_components::{TestComponent, TestComponentParameters};

    #[test]
    fn solve() {
        let component = TestComponent::from_parameters(TestComponentParameters { p: 2.0 });

        let input_state = component.extract_state(&TimeseriesCollection::new(), 2020.0);
        let output_state = component.solve(2020.0, 2021.0, &input_state).unwrap();

        assert_eq!(*output_state.get("Concentrations|CO2"), 2.0 * 1.3);
    }
}

use crate::component::{Component, ParameterDefinition};
use crate::timeseries::{Time, TimeAxis, Timeseries};
use crate::timeseries_collection::{TimeseriesCollection, VariableType};
use numpy::ndarray::Array;
use petgraph::algo::is_cyclic_directed;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::visit::Bfs;
use petgraph::Graph;
use std::collections::HashMap;
use std::ops::Index;
use std::sync::Arc;

type C = Arc<dyn Component + Send + Sync>;

struct VariableDefinition {
    name: String,
    unit: String,
}

impl VariableDefinition {
    fn from_parameter_definition(definition: &ParameterDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            unit: definition.unit.clone(),
        }
    }
}

/// Build a new model from a set of components
///
/// The builder generates a graph that defines the inter-component dependencies
/// and determines what variables are endogenous and exogenous to the model.
/// This graph is used by the model to define the order in which components are solved.
///
/// # Examples
/// TODO: figure out how to share example components throughout the docs
pub struct ModelBuilder {
    components: Vec<C>,
    timeseries: TimeseriesCollection,
    time_axis: Arc<TimeAxis>,
}

/// Checks if the new definition is valid
///
/// If any definitions share a name then the units must be equivalent
///
/// Panics if the parameter definition is inconsistent with any existing definitions.
fn verify_definition(
    definitions: &mut HashMap<String, VariableDefinition>,
    definition: &ParameterDefinition,
) {
    let existing = definitions.get(&definition.name);
    match existing {
        Some(existing) => {
            assert_eq!(existing.unit, definition.unit);
        }
        None => {
            definitions.insert(
                definition.name.clone(),
                VariableDefinition::from_parameter_definition(definition),
            );
        }
    }
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            components: vec![],
            timeseries: TimeseriesCollection::new(),
            time_axis: Arc::new(TimeAxis::from_values(Array::range(2000.0, 2100.0, 1.0))),
        }
    }

    /// Register a component with the builder
    pub fn with_component(&mut self, component: Arc<dyn Component + Send + Sync>) -> &mut Self {
        self.components.push(component);
        self
    }

    /// Supply exogenous data to be used by the model
    ///
    /// Any unneeded timeseries will be ignored.
    pub fn with_exogenous_variable(
        &mut self,
        name: &str,
        timeseries: Timeseries<f32>,
    ) -> &mut Self {
        self.timeseries
            .add_timeseries(name.to_string(), timeseries, VariableType::Exogenous);
        self
    }

    /// Supply exogenous data to be used by the model
    ///
    /// Any unneeded timeseries will be ignored.
    pub fn with_exogenous_collection(&mut self, collection: TimeseriesCollection) -> &mut Self {
        collection.into_iter().for_each(|x| {
            self.timeseries
                .add_timeseries(x.name, x.timeseries, x.variable_type)
        });
        self
    }

    /// Specify the time axis that will be used by the model
    ///
    /// This time axis defines the time steps (including bounds) on which the model will be iterated.
    pub fn with_time_axis(&mut self, time_axis: TimeAxis) -> &mut Self {
        self.time_axis = Arc::new(time_axis);
        self
    }

    /// Builds the component graph for the registered components and creates a concrete model
    ///
    /// Panics if the required data to build a model is not available.
    pub fn build(&self) -> Model {
        let mut graph: Graph<Option<C>, Option<ParameterDefinition>> = Graph::new();
        let mut endrogoneous: HashMap<String, NodeIndex> = HashMap::new();
        let mut exogenous: Vec<String> = vec![];
        let mut definitions: HashMap<String, VariableDefinition> = HashMap::new();
        let initial_node = graph.add_node(Option::None);

        self.components.iter().for_each(|component| {
            let node = graph.add_node(Option::from(component.clone()));
            let mut has_dependencies = false;

            let requires = component.inputs();
            let provides = component.outputs();

            requires.iter().for_each(|requirement| {
                verify_definition(&mut definitions, requirement);

                if exogenous.contains(&requirement.name) {
                    // Link to the node that provides the requirement
                    graph.add_edge(
                        endrogoneous[&requirement.name],
                        node,
                        Option::from(requirement.clone()),
                    );
                    has_dependencies = true;
                } else {
                    // Add a new variable that must be defined outside of the model
                    exogenous.push(requirement.name.clone())
                }
            });

            if !has_dependencies {
                // If the node has no dependecies create a link to the initial node
                // This ensures that we have a single connected graph
                // There might be smarter ways to iterate over the nodes, but this is fine for now
                graph.add_edge(initial_node, node, None);
            }

            provides.iter().for_each(|requirement| {
                verify_definition(&mut definitions, requirement);

                let val = endrogoneous.get(&requirement.name);

                match val {
                    None => {
                        endrogoneous.insert(requirement.name.clone(), node);
                    }
                    Some(node_index) => {
                        println!("Duplicate definition of {:?} requirement", requirement.name);

                        graph.add_edge(*node_index, node, Option::from(requirement.clone()));
                        endrogoneous.insert(requirement.name.clone(), node);
                    }
                }
            });
        });

        // Check that the component graph doesn't contain any loops
        assert!(!is_cyclic_directed(&graph));

        // Create the timeseries collection using the information from the components
        let mut collection = TimeseriesCollection::new();
        for (_, definition) in definitions {
            if exogenous.contains(&definition.name) {
                // Exogenous variable is expected to be supplied

                // todo: This should consume the timeseries and then interpolate onto the correct timeaxis
                let timeseries = self.timeseries.get_timeseries(&definition.name);

                match timeseries {
                    Some(timeseries) => collection.add_timeseries(
                        definition.name,
                        timeseries.to_owned(),
                        VariableType::Exogenous,
                    ),

                    None => panic!("No exogenous data for {}", definition.name),
                }
            } else {
                // Create a placeholder for data that will be generated by the model
                collection.add_timeseries(
                    definition.name,
                    Timeseries::new_empty(self.time_axis.clone(), definition.unit),
                    VariableType::Endogenous,
                )
            }
        }

        // Add the components to the graph
        Model::new(graph, initial_node, collection, self.time_axis.clone())
    }
}

pub struct Model {
    components: Graph<Option<C>, Option<ParameterDefinition>>,
    initial_node: NodeIndex,
    collection: TimeseriesCollection,
    time_axis: Arc<TimeAxis>,
    time_index: usize,
}

/// A model represents a collection of components that can be solved together
/// Each component may require information from other components to be solved (endrogenous) or
/// predefined data (exogenous).
impl Model {
    pub fn new(
        components: Graph<Option<C>, Option<ParameterDefinition>>,
        initial_node: NodeIndex,
        collection: TimeseriesCollection,
        time_axis: Arc<TimeAxis>,
    ) -> Self {
        Self {
            components,
            initial_node,
            collection,
            time_axis,
            time_index: 0,
        }
    }

    /// Gets the time value at the current step
    pub fn current_time(&self) -> Time {
        self.time_axis.at(self.time_index).unwrap()
    }
    pub fn current_time_bounds(&self) -> (Time, Time) {
        self.time_axis.at_bounds(self.time_index).unwrap()
    }

    fn process_node(&mut self, component: C) {
        let input_state = component.extract_state(&self.collection, self.current_time());

        let (start, end) = self.current_time_bounds();

        let result = component.solve(start, end, &input_state);

        match result {
            Ok(output_state) => output_state.iter().for_each(|(key, value)| {
                let ts = self.collection.get_timeseries_mut(key).unwrap();
                ts.set(self.time_index + 1, *value)
            }),
            Err(err) => {
                println!("Solving failed: {}", err)
            }
        }
    }

    fn step_model(&mut self) {
        let mut bfs = Bfs::new(&self.components, self.initial_node);
        while let Some(nx) = bfs.next(&self.components) {
            let c = self.components.index(nx);

            if c.is_some() {
                let c = c.as_ref().unwrap().clone();
                self.process_node(c)
            }
        }
    }

    /// Steps the model forward one time step
    pub fn step(&mut self) {
        assert!(self.time_index < self.time_axis.len());
        self.step_model();

        self.time_index += 1;
    }

    /// Steps the model until the end of the time axis
    pub fn run(&mut self) {
        while self.time_index < self.time_axis.len() {
            self.step();
        }
    }

    pub fn as_dot(&self) -> Dot<&Graph<Option<C>, Option<ParameterDefinition>>> {
        Dot::with_attr_getters(
            &self.components,
            &[Config::NodeNoLabel, Config::EdgeNoLabel],
            &|_, er| {
                let requirement = er.weight();
                match requirement {
                    None => "".to_string(),
                    Some(r) => format!("label = \"{:?}\"", r),
                }
            },
            &|_, (_, component)| match component {
                None => "".to_string(),
                Some(c) => format!("label = \"{:?}\"", c),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::example_components::{TestComponent, TestComponentParameters};
    use numpy::ndarray::Array;

    #[test]
    fn build() {
        let time_axis = TimeAxis::from_values(Array::range(2020.0, 2025.0, 1.0));
        let mut model = ModelBuilder::new()
            .with_time_axis(time_axis)
            .with_component(Arc::new(TestComponent::from_parameters(
                TestComponentParameters { p: 0.5 },
            )))
            .build();
    }

    #[test]
    fn step() {
        let time_axis = TimeAxis::from_values(Array::range(2020.0, 2025.0, 1.0));
        let mut model = ModelBuilder::new()
            .with_time_axis(time_axis)
            .with_component(Arc::new(TestComponent::from_parameters(
                TestComponentParameters { p: 0.5 },
            )))
            .build();

        assert_eq!(model.time_index, 0);
        model.step();
        model.step();
        assert_eq!(model.time_index, 2);
        assert_eq!(model.current_time(), 2022.0);
        model.run();
        assert_eq!(model.time_index, 5);
    }

    #[test]
    fn dot() {
        let time_axis = TimeAxis::from_values(Array::range(2020.0, 2025.0, 1.0));
        let model = ModelBuilder::new()
            .with_time_axis(time_axis)
            .with_component(Arc::new(TestComponent::from_parameters(
                TestComponentParameters { p: 0.5 },
            )))
            .build();

        let exp = "digraph {
    0 [ ]
    1 [ label = \"TestComponent { parameters: TestComponentParameters { p: 0.5 } }\"]
    0 -> 1 [ ]
}
";

        let res = format!("{:?}", model.as_dot());
        assert_eq!(res, exp);
    }
}

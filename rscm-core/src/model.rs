use crate::component::{Component, InputState, RequirementDefinition, State};
use crate::timeseries::{Time, TimeAxis, Timeseries};
use crate::timeseries_collection::{TimeseriesCollection, VariableType};
use numpy::ndarray::Array;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::visit::{Bfs, IntoNeighbors, IntoNodeIdentifiers, Visitable};
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
    fn from_requirement_definition(definition: &RequirementDefinition) -> Self {
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
    exogenous_variables: TimeseriesCollection,
    initial_values: InputState,
    time_axis: Arc<TimeAxis>,
}

/// Checks if the new definition is valid
///
/// If any definitions share a name then the units must be equivalent
///
/// Panics if the parameter definition is inconsistent with any existing definitions.
fn verify_definition(
    definitions: &mut HashMap<String, VariableDefinition>,
    definition: &RequirementDefinition,
) {
    let existing = definitions.get(&definition.name);
    match existing {
        Some(existing) => {
            assert_eq!(existing.unit, definition.unit);
        }
        None => {
            definitions.insert(
                definition.name.clone(),
                VariableDefinition::from_requirement_definition(definition),
            );
        }
    }
}

/// Check that a component graph is valid
///
/// We require a directed acyclic graph which doesn't contain any cycles (other than a self-referential node).
/// This avoids the case where component `A` depends on a component `B`,
/// but component `B` also depends on component `A`.
fn is_valid_graph<G>(g: G) -> bool
where
    G: IntoNodeIdentifiers + IntoNeighbors + Visitable,
{
    use petgraph::visit::{depth_first_search, DfsEvent};

    depth_first_search(g, g.node_identifiers(), |event| match event {
        DfsEvent::BackEdge(a, b) => {
            // If the cycle is self-referential then that is fine
            match a == b {
                true => Ok(()),
                false => Err(()),
            }
        }
        _ => Ok(()),
    })
    .is_err()
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            components: vec![],
            initial_values: InputState::empty(),
            exogenous_variables: TimeseriesCollection::new(),
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
        self.exogenous_variables.add_timeseries(
            name.to_string(),
            timeseries,
            VariableType::Exogenous,
        );
        self
    }

    /// Supply exogenous data to be used by the model
    ///
    /// Any unneeded timeseries will be ignored.
    pub fn with_exogenous_collection(&mut self, collection: TimeseriesCollection) -> &mut Self {
        collection.into_iter().for_each(|x| {
            self.exogenous_variables
                .add_timeseries(x.name, x.timeseries, x.variable_type)
        });
        self
    }

    /// Adds some state to the set of initial values
    ///
    /// These initial values are used to provide some initial values at `t_0`.
    /// Initial values are used for requirements which have a type of `RequirementType::InputAndOutput`.
    /// These requirements use state from the current timestep in order to generate a value for the
    /// next timestep.
    /// Building a model where any variables which have `RequirementType::InputAndOutput`, but
    /// do not have an initial value will result in an error.
    pub fn with_initial_values(&mut self, initial_values: InputState) -> &mut Self {
        self.initial_values.merge(initial_values);
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
        // todo: refactor once this is more stable
        let mut graph: Graph<Option<C>, Option<RequirementDefinition>> = Graph::new();
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
                // If the node has no dependencies on other components,
                // create a link to the initial node.
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
        assert!(!is_valid_graph(&graph));

        // Create the timeseries collection using the information from the components
        let mut collection = TimeseriesCollection::new();
        for (_, definition) in definitions {
            if exogenous.contains(&definition.name) {
                // Exogenous variable is expected to be supplied
                if self.initial_values.has(&definition.name) {
                    // An initial value was provided
                    let mut ts = Timeseries::new_empty(self.time_axis.clone(), definition.unit);
                    ts.set(0, *self.initial_values.get(&definition.name));

                    // Note that timeseries that are initialised are defined as Endogenous
                    // all but the first time point come from the model.
                    // This could potentially be defined as a different VariableType if needed.
                    collection.add_timeseries(definition.name, ts, VariableType::Endogenous)
                } else {
                    // Check if the timeseries is available in the provided exogenous variables
                    // todo: This should consume the timeseries and then interpolate onto the correct timeaxis
                    let timeseries = self.exogenous_variables.get_timeseries(&definition.name);

                    match timeseries {
                        Some(timeseries) => collection.add_timeseries(
                            definition.name,
                            timeseries.to_owned(),
                            VariableType::Exogenous,
                        ),
                        None => println!("Requires data for {}", definition.name), // None => panic!("No exogenous data for {}", definition.name),
                    }
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
    components: Graph<Option<C>, Option<RequirementDefinition>>,
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
        components: Graph<Option<C>, Option<RequirementDefinition>>,
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

    /// Step the model forward a step by solving each component for the current time step.
    ///
    /// A breadth-first search across the component graph starting at the initial node
    /// will solve the components in a way that ensures any models with dependencies are solved
    /// after the dependent component is first solved.
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

    /// Create a diagram the represents the component graph
    ///
    /// Useful for debugging
    pub fn as_dot(&self) -> Dot<&Graph<Option<C>, Option<RequirementDefinition>>> {
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

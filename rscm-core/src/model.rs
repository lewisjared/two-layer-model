use crate::component::{Component, ParameterDefinition};
use crate::timeseries::{Time, TimeAxis};
use crate::timeseries_collection::TimeseriesCollection;
use numpy::ndarray::Array;
use petgraph::algo::is_cyclic_directed;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::visit::Bfs;
use petgraph::Graph;
use std::collections::HashMap;
use std::ops::Index;
use std::sync::Arc;

/// Algo
/// * Register the components
/// * Register the time axis
/// Build
/// * Iterate through components in order
/// * If an input already exists, use it and link to the node that provides it
///
///
///
type C = Arc<dyn Component + Send + Sync>;

pub struct ModelBuilder {
    components: Vec<C>,
    time_axis: Arc<TimeAxis>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            components: vec![],
            time_axis: Arc::new(TimeAxis::from_values(Array::range(2000.0, 2100.0, 1.0))),
        }
    }
    pub fn with_component(&mut self, component: Arc<dyn Component + Send + Sync>) -> &mut Self {
        self.components.push(component);
        self
    }
    pub fn with_time_axis(&mut self, time_axis: TimeAxis) -> &mut Self {
        self.time_axis = Arc::new(time_axis);
        self
    }

    /// Builds the component graph for the registered components
    pub fn build(&self) -> Model {
        let mut graph: Graph<Option<C>, Option<ParameterDefinition>> = Graph::new();
        let mut endrogoneous: HashMap<String, NodeIndex> = HashMap::new();
        let mut exogenous: Vec<String> = vec![];
        let initial_node = graph.add_node(Option::None);

        self.components.iter().for_each(|component| {
            let node = graph.add_node(Option::from(component.clone()));
            let mut has_dependencies = false;

            let requires = component.inputs();
            let provides = component.outputs();

            requires.iter().for_each(|requirement| {
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

        assert!(!is_cyclic_directed(&graph));

        // Add the components to the graph
        Model::new(graph, initial_node, self.time_axis.clone())
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
        time_axis: Arc<TimeAxis>,
    ) -> Self {
        Self {
            components,
            initial_node,
            collection: TimeseriesCollection::new(),
            time_axis,
            time_index: 0,
        }
    }

    /// Gets the time value at the current step
    pub fn current_time(&self) -> Time {
        self.time_axis.at(self.time_index).unwrap()
    }

    fn process_node(&self, _component_link: &C) {}

    fn step_model(&mut self) {
        let mut bfs = Bfs::new(&self.components, self.initial_node);
        while let Some(nx) = bfs.next(&self.components) {
            let c = self.components.index(nx);

            if c.is_some() {
                self.process_node(c.as_ref().unwrap())
            }
        }
    }

    /// Steps the model forward one time step
    pub fn step(&mut self) {
        assert!(self.time_index < self.time_axis.len());

        self.time_index += 1;
        self.step_model()
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
    use crate::component::{TestComponent, TestComponentParameters};
    use numpy::ndarray::Array;

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

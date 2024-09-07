use crate::component::Component;
use crate::timeseries::{Time, TimeAxis};
use crate::timeseries_collection::TimeseriesCollection;
use numpy::ndarray::Array;
use petgraph::algo::is_cyclic_directed;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::visit::{Bfs, IntoNeighborsDirected};
use petgraph::Graph;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Index;
use std::sync::Arc;

/// Algo
/// * Register the components
/// * Register the time axis
/// Build
/// * Iterate through components in order
/// * If an input already exists, use it and link to the node that provides it
///

#[derive(Clone, Debug)]
pub(self) struct ComponentLink {
    component: Option<Arc<dyn Component + Send + Sync>>,
    provides: Vec<String>,
    requires: Vec<String>,
}

pub struct ModelBuilder {
    links: Vec<ComponentLink>,
    time_axis: Arc<TimeAxis>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            links: vec![],
            time_axis: Arc::new(TimeAxis::from_values(Array::range(2000.0, 2100.0, 1.0))),
        }
    }
    pub fn with_component(&mut self, component: Arc<dyn Component + Send + Sync>) -> &mut Self {
        self.links.push(ComponentLink {
            component: Option::from(component),
            provides: vec![],
            requires: vec![],
        });
        self
    }
    pub fn with_time_axis(&mut self, time_axis: TimeAxis) -> &mut Self {
        self.time_axis = Arc::new(time_axis);
        self
    }

    /// Builds the component graph for the registered components
    pub fn build(&self) -> Model {
        let mut graph: Graph<ComponentLink, String> = Graph::new();
        let mut endrogoneous: HashMap<String, NodeIndex> = HashMap::new();
        let mut exogenous: Vec<String> = vec![];
        let initial_node = graph.add_node(ComponentLink {
            component: Option::None,
            provides: vec![],
            requires: vec![],
        });

        self.links.iter().for_each(|link| {
            let node = graph.add_node(link.clone());
            link.requires.iter().for_each(|requirement| {
                if exogenous.contains(requirement) {
                    // Link to the node that provides the requirement
                    graph.add_edge(node, endrogoneous[requirement], requirement.clone());
                } else {
                    // Add a new variable that must be defined outside of the model
                    exogenous.push(requirement.to_string())
                }
            });

            link.provides.iter().for_each(|requirement| {
                if exogenous.contains(requirement) {
                    // TODO: Fix this so we can chain together something that takes and maybe
                    // modifies the calculated state for that step.
                    panic!("Multiple definitions of {}", requirement);
                } else {
                    // Keep a reference to the node that provides the requirement
                    endrogoneous.insert(requirement.to_string(), node);
                }
            });
        });

        println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

        assert!(!is_cyclic_directed(&graph));

        // Add the components to the graph
        Model::new(graph, initial_node, self.time_axis.clone())
    }
}

pub struct Model {
    components: Graph<ComponentLink, String>,
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
        components: Graph<ComponentLink, String>,
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

    fn process_node(&self, component_link: &ComponentLink) {
        println!("Visiting {:?}", component_link)
    }

    fn step_model(&mut self) {
        let mut bfs = Bfs::new(&self.components, self.initial_node);
        while let Some(nx) = bfs.next(&self.components) {
            let c = self.components.index(nx);

            if (c.component.is_some()) {
                self.process_node(c)
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
}

use crate::component::{Component, InputState, RequirementDefinition};
use crate::example_components::{TestComponent, TestComponentParameters};
use crate::timeseries::{FloatValue, Time};
use crate::{impl_component, impl_component_from_parameters};
use pyo3::prelude::*;
use pyo3::pyclass;
use std::collections::HashMap;

#[pyclass]
#[pyo3(name = "TestComponent")]
pub struct PyTestComponent(pub(crate) TestComponent);

impl_component!(PyTestComponent);
impl_component_from_parameters!(PyTestComponent, TestComponent, TestComponentParameters);

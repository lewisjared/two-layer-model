use crate::example_components::{TestComponent, TestComponentParameters};
use crate::impl_component;
use pyo3::prelude::*;
use pyo3::pyclass;

use crate::component::{Component, RequirementDefinition};

#[pyclass]
#[pyo3(name = "TestComponent")]
pub struct PyTestComponent(pub(crate) TestComponent);

impl_component!(PyTestComponent, TestComponent, TestComponentParameters);

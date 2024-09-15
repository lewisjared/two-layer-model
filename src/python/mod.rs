use crate::two_layer::{TwoLayerComponent, TwoLayerComponentParameters};
use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use pyo3_stub_gen::define_stub_info_gatherer;
use rscm_core::component::RequirementDefinition;
use rscm_core::component::{Component, InputState};
use rscm_core::python::core;
use rscm_core::timeseries::{FloatValue, Time};
use rscm_core::{impl_component, impl_component_from_parameters};
use std::collections::HashMap;

#[pyclass]
#[pyo3(name = "TwoLayerComponent")]
pub struct PyTwoLayerComponent(pub TwoLayerComponent);

impl_component!(PyTwoLayerComponent);
impl_component_from_parameters!(
    PyTwoLayerComponent,
    TwoLayerComponent,
    TwoLayerComponentParameters
);

#[pymodule]
#[pyo3(name = "_lib")]
fn two_layer_model(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(core))?;
    m.add_class::<PyTwoLayerComponent>()?;

    set_path(m, "two_layer_model._lib.core", "core")?;

    Ok(())
}

fn set_path(m: &Bound<'_, PyModule>, path: &str, module: &str) -> PyResult<()> {
    m.py().run_bound(
        &format!(
            "\
import sys
sys.modules['{path}'] = {module}
    "
        ),
        None,
        Some(&m.dict()),
    )
}

// Define a function to gather stub information.
define_stub_info_gatherer!(stub_info);

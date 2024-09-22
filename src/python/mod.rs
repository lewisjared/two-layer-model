use crate::two_layer::{TwoLayerComponent, TwoLayerComponentParameters};
use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use rscm_components::python::components;
use rscm_core::create_component_builder;
use rscm_core::python::{core, PyRustComponent};

create_component_builder!(
    TwoLayerComponentBuilder,
    TwoLayerComponent,
    TwoLayerComponentParameters
);

#[pymodule]
#[pyo3(name = "_lib")]
fn rscm(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(core))?;
    m.add_wrapped(wrap_pymodule!(components))?;
    m.add_class::<TwoLayerComponentBuilder>()?;

    set_path(m, "rscm._lib.core", "core")?;
    set_path(m, "rscm._lib.components", "components")?;

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

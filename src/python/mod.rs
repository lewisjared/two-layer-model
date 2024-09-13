use crate::two_layer::{TwoLayerComponent, TwoLayerComponentParameters};
use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::gen_stub_pyfunction};
use rscm_core::component::{Component, InputState};
use rscm_core::python::core;
use rscm_core::timeseries::FloatValue;
use std::collections::HashMap;

#[gen_stub_pyfunction]
#[pyfunction]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[pyclass]
#[pyo3(name = "TwoLayerComponent")]
pub struct PyTwoLayerComponent(pub TwoLayerComponent);

#[pymethods]
impl PyTwoLayerComponent {
    #[staticmethod]
    fn from_parameters(
        lambda0: FloatValue,
        a: FloatValue,
        efficacy: FloatValue,
        eta: FloatValue,
        heat_capacity_surface: FloatValue,
        heat_capacity_deep: FloatValue,
    ) -> Self {
        Self(TwoLayerComponent::from_parameters(
            TwoLayerComponentParameters {
                lambda0,
                a,
                efficacy,
                eta,
                heat_capacity_surface,
                heat_capacity_deep,
            },
        ))
    }

    fn solve<'py>(
        &self,
        t_current: FloatValue,
        t_next: FloatValue,
        state: HashMap<String, FloatValue>,
    ) -> PyResult<HashMap<String, FloatValue>> {
        let input_state = InputState::from_hashmap(state);
        let output_state = self.0.solve(t_current, t_next, &input_state);

        Ok(output_state.map(|t| t.to_hashmap()).unwrap())
    }
}
#[pymodule]
#[pyo3(name = "_lib")]
fn two_layer_model(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(core))?;
    m.add_function(wrap_pyfunction!(add, m)?)?;
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

use crate::{solve_tlm, TwoLayerModel, TwoLayerModelParameters};
use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::gen_stub_pyfunction};
use rscm_core::python::core;
use std::sync::Arc;

#[gen_stub_pyfunction]
#[pyfunction]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[gen_stub_pyfunction]
#[pyfunction]
#[pyo3(name = "solve_tlm")]
pub fn py_solve_tlm() {
    solve_tlm()
}

#[pyclass]
#[pyo3(name = "TwoLayerModel")]
pub struct PyTwoLayerModel(pub TwoLayerModel);

#[pymethods]
impl PyTwoLayerModel {
    #[new]
    fn from_parameters(
        lambda0: f32,
        a: f32,
        efficacy: f32,
        eta: f32,
        heat_capacity_surface: f32,
        heat_capacity_deep: f32,
    ) -> Self {
        Self(TwoLayerModel::from_parameters(Arc::new(
            TwoLayerModelParameters {
                lambda0,
                a,
                efficacy,
                eta,
                heat_capacity_surface,
                heat_capacity_deep,
            },
        )))
    }

    fn solve(&self) {
        self.0.solve().expect("TODO: panic message");
    }
}
#[pymodule]
#[pyo3(name = "_lib")]
fn two_layer_model(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(core))?;
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(py_solve_tlm, m)?)?;
    m.add_class::<PyTwoLayerModel>()?;

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

extern crate uom;

use ode_solvers::dop853::*;
use ode_solvers::*;
use pyo3::prelude::*;
use pyo3::{pyfunction, pymodule};

#[pyfunction]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[pymodule]
#[pyo3(name = "_core")]
fn string_sum(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    Ok(())
}

type State = Vector3<f64>;
type Time = f64;

struct TwoLayerModel {
    lambda0: f32,
    a: f32,
    efficacy: f32,
    eta: f32,
    heat_capacity_surface: f32,
    heat_capacity_deep: f32,
}

impl TwoLayerModel {
    fn calculate(&self, erf: f32, temperature_surface: f32, temperature_deep: f32) -> [f32; 3] {
        let temperature_difference = temperature_surface - temperature_deep;

        let lambda_eff = self.lambda0 - self.a * temperature_surface;
        let heat_exchange_surface = self.efficacy * self.eta * temperature_difference;
        let dtemperature_surface_dt =
            (erf - lambda_eff * temperature_surface - heat_exchange_surface)
                / self.heat_capacity_surface;

        let heat_exchange_deep = self.eta * temperature_difference;
        let dtemperature_deep_dt = heat_exchange_deep / self.heat_capacity_deep;

        let dy_dt = [
            dtemperature_surface_dt,
            dtemperature_deep_dt,
            self.heat_capacity_surface * dtemperature_surface_dt
                + self.heat_capacity_deep * dtemperature_deep_dt,
        ];

        dy_dt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

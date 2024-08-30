mod model;
mod timeseries;

extern crate uom;

use crate::timeseries::Timeseries;
use ndarray::array;
use ode_solvers::*;
use pyo3::prelude::*;
use pyo3::{pyfunction, pymodule};

#[pyfunction]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

// Define some types that are used by OdeSolvers
type ModelState = Vector3<f32>;
type Time = f32;

struct TwoLayerModel {
    erf: Timeseries,
    lambda0: f32,
    a: f32,
    efficacy: f32,
    eta: f32,
    heat_capacity_surface: f32,
    heat_capacity_deep: f32,
}

// Create the set of ODEs to represent the two layer model
impl System<Time, ModelState> for TwoLayerModel {
    fn system(&self, t: Time, y: &ModelState, dy: &mut ModelState) {
        let temperature_surface = y[0];
        let temperature_deep = y[1];
        let erf = self.erf.at_time(t).unwrap();

        let temperature_difference = temperature_surface - temperature_deep;

        let lambda_eff = self.lambda0 - self.a * temperature_surface;
        let heat_exchange_surface = self.efficacy * self.eta * temperature_difference;
        let dtemperature_surface_dt =
            (erf - lambda_eff * temperature_surface - heat_exchange_surface)
                / self.heat_capacity_surface;

        let heat_exchange_deep = self.eta * temperature_difference;
        let dtemperature_deep_dt = heat_exchange_deep / self.heat_capacity_deep;

        dy[0] = dtemperature_surface_dt;
        dy[1] = dtemperature_deep_dt;
        dy[2] = self.heat_capacity_surface * dtemperature_surface_dt
            + self.heat_capacity_deep * dtemperature_deep_dt;
    }
}

#[pyfunction]
fn solve_tlm() {
    // Initialise the model
    let model = TwoLayerModel {
        erf: Timeseries::from_values(
            array![0.0, 0.0, 2.0, 2.0],
            array![1848.0, 1849.0, 1850.0, 1900.0],
        ),
        lambda0: 0.5,
        a: 0.01,
        efficacy: 0.5,
        eta: 0.1,
        heat_capacity_surface: 1.0,
        heat_capacity_deep: 100.0,
    };

    let y0 = ModelState::new(0.0, 0.0, 0.0);

    // Create the solver
    let mut stepper = Rk4::new(model, 1848.0, y0, 1900.0, 1.0);
    let res = stepper.integrate();

    // Handle result
    match res {
        Ok(stats) => {
            stats.print();

            // Do something with the output...
            // let path = Path::new("./outputs/kepler_orbit_dopri5.dat");
            // save(stepper.x_out(), stepper.y_out(), path);
            // println!("Results saved in: {:?}", path);
        }
        Err(_) => println!("An error occured."),
    }
}

#[pymodule]
#[pyo3(name = "_core")]
fn core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(solve_tlm, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        solve_tlm();
    }
}

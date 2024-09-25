mod carbon_cycle;
mod co2_erf;
pub mod ocean_carbon_cycle;

pub use carbon_cycle::{CarbonCycleComponent, CarbonCycleParameters, SolverOptions};
pub use co2_erf::{CO2ERFParameters, CO2ERF};

use crate::models::carbon_cycle::CarbonCycleParameters;
use crate::models::co2_erf::CO2ERFParameters;
use numpy::array;
use numpy::ndarray::Array;
use rscm_core::component::InputState;
use rscm_core::interpolate::strategies::{InterpolationStrategy, NextStrategy, PreviousStrategy};
use rscm_core::model::ModelBuilder;
use rscm_core::timeseries::{Time, TimeAxis, Timeseries};
use std::sync::Arc;

mod carbon_cycle;
mod co2_erf;

#[test]
fn test_carbon_cycle() {
    let tau = 20.3;
    let conc_pi = 280.0;
    let conc_initial = 280.0;
    let t_initial = 1750.0;
    let emissions_level = 10.0;
    let step_year = 1850.0;
    //Can use any temperature as the temperature feedback is set to zero
    // so this is effectively a noise parameter.
    let temperature_value = 1.0;
    let step_size = 1.0 / 120.0;

    let gtc_per_ppm = 2.13;

    // Have to have no temperature feedback for this to work
    let alpha_temperature = 0.0;

    let time_axis = TimeAxis::from_values(Array::range(t_initial, 2100.0, 1.0));

    let mut builder = ModelBuilder::new();

    let get_exp_values_before_step = |time: Time| -> f32 {
        (conc_initial - conc_pi) * (-(time - t_initial) / tau).exp() + conc_pi
    };

    let get_exp_values_after_step = |time: Time| -> f32 {
        emissions_level / gtc_per_ppm * tau * (1.0 - (-(time - step_year) / tau).exp())
            + get_exp_values_before_step(time)
    };

    let emissions = Timeseries::new(
        array![0.0, 0.0, step_size, step_size],
        Arc::new(TimeAxis::from_bounds(array![
            t_initial,
            (t_initial + step_year) / 2.0,
            step_year,
            step_year + 50.0,
            2100.0
        ])),
        "GtC / yr".to_string(),
        InterpolationStrategy::from(PreviousStrategy::new(true)),
    );
    let temperature = Timeseries::new(
        array![temperature_value],
        Arc::new(TimeAxis::from_bounds(array![t_initial, 2100.0])),
        "K".to_string(),
        InterpolationStrategy::from(NextStrategy::new(true)),
    );

    // Build a model consisting of a single carbon cycle component
    let mut model = builder
        .with_component(Arc::new(
            carbon_cycle::CarbonCycleComponent::from_parameters(CarbonCycleParameters {
                tau,
                conc_pi,
                alpha_temperature,
            }),
        ))
        .with_initial_values(InputState::from_vectors(
            vec![0.0, 0.0, conc_initial],
            vec![
                "Cumulative Land Uptake".to_string(),
                "Cumulative Emissions|CO2".to_string(),
                "Atmospheric Concentration|CO2".to_string(),
            ],
        ))
        .with_time_axis(time_axis)
        .with_exogenous_variable("Emissions|CO2|Anthropogenic", emissions)
        .with_exogenous_variable("Surface Temperature", temperature)
        .build();

    model.run()
}

#[test]
fn test_coupled_model() {
    let tau = 20.3;
    let conc_pi = 280.0;
    let t_initial = 1750.0;
    let erf_2xco2 = 4.0;
    let step_year = 1850.0;
    let step_size = 1.0 / 120.0;

    // Have to have no temperature feedback for this to work
    let alpha_temperature = 0.0;

    let time_axis = TimeAxis::from_values(Array::range(t_initial, 2100.0, 1.0));
    let emissions = Timeseries::new(
        array![0.0, 0.0, step_size, step_size],
        Arc::new(TimeAxis::from_bounds(array![
            t_initial,
            (t_initial + step_year) / 2.0,
            step_year,
            step_year + 50.0,
            2100.0
        ])),
        "GtC / yr".to_string(),
        InterpolationStrategy::from(PreviousStrategy::new(true)),
    );

    let mut builder = ModelBuilder::new();

    // Build a model consisting of a carbon cycle and a CO2-only ERF component
    let mut model = builder
        .with_component(Arc::new(
            carbon_cycle::CarbonCycleComponent::from_parameters(CarbonCycleParameters {
                tau,
                conc_pi,
                alpha_temperature,
            }),
        ))
        .with_component(Arc::new(co2_erf::CO2ERF::from_parameters(
            CO2ERFParameters { erf_2xco2, conc_pi },
        )))
        .with_time_axis(time_axis)
        .with_exogenous_variable("Emissions|CO2", emissions)
        .build();

    println!("{:?}", model.as_dot());

    // Run the model
    model.run()
}

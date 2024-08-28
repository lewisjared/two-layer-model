use pyo3::{pyfunction, pymodule};
use pyo3::prelude::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

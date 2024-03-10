use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

pub fn load_plugins() -> bool {
    Python::with_gil(|py| {
        let m = py.import("breezy.plugin").unwrap();
        match m.call_method0("load_plugins") {
            Ok(_) => true,
            Err(e)
                if e.is_instance_of::<PyRuntimeError>(py)
                    && e.to_string().contains("Breezy already initialized") =>
            {
                false
            }
            Err(e) => panic!("Error loading plugins: {}", e),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_plugins() {
        load_plugins();
    }
}

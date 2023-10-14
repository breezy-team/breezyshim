use pyo3::prelude::*;

pub fn retrieve_github_token() -> String {
    Python::with_gil(|py| {
        let m = py.import("breezy.plugins.github.forge").unwrap();

        let token = m.call_method0("retrieve_github_token").unwrap();

        token.extract().unwrap()
    })
}

//! Basic support for interacting with GitHub.
use pyo3::prelude::*;

/// Retrieve a GitHub authentication token.
pub fn retrieve_github_token() -> String {
    Python::attach(|py| {
        let m = py.import("breezy.plugins.github.forge").unwrap();

        let token = m.call_method0("retrieve_github_token").unwrap();

        token.extract().unwrap()
    })
}

/// Login to GitHub using saved credentials.
pub fn login() -> PyResult<()> {
    Python::attach(|py| {
        let m = py.import("breezy.plugins.github.cmds").unwrap();
        let cmd = m.getattr("cmd_github_login").unwrap();

        let cmd_gl = cmd.call0().unwrap();
        cmd_gl.call_method0("_setup_outf").unwrap();

        cmd_gl.call_method0("run").unwrap();

        Ok(())
    })
}

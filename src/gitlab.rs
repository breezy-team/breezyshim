//! Basic support for interacting with GitLab
use pyo3::prelude::*;

pub fn login(url: &url::Url) -> PyResult<()> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.plugins.gitlab.cmds").unwrap();
        let cmd = m.getattr("cmd_gitlab_login").unwrap();

        let cmd_gl = cmd.call0().unwrap();
        cmd_gl.call_method0("_setup_outf").unwrap();

        cmd_gl.call_method1("run", (url.as_str(),)).unwrap();

        Ok(())
    })
}

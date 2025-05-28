//! Launchpad login and related functions
use pyo3::prelude::*;
use launchpadlib::uris;

pub fn login(url: &url::Url) {
    Python::with_gil(|py| -> PyResult<()> {
        let m = py.import("breezy.plugins.launchpad.cmds")?;
        let cmd = m.getattr("cmd_launchpad_login")?;

        let cmd_lp = cmd.call0()?;
        cmd_lp.call_method0("_setup_outf")?;

        cmd_lp.call_method1("run", (url.as_str(),))?;

        let lp_api = py.import("breezy.plugins.launchpad.lp_api")?;

        // The original code extracted a service root like this:
        let lp_uris = lp_api.getattr("uris")?.call0()?.extract::<Vec<(String, String)>>()?;

        let lp_service_root = lp_uris
            .iter()
            .find(|(_key, root)| {
                url.host_str() == Some(root) || url.host_str() == Some(root.trim_end_matches('/'))
            })
            .unwrap()
            .1.clone();

        let kwargs = pyo3::types::PyDict::new(py);
        kwargs.set_item("version", "devel")?;
        lp_api.call_method("connect_launchpad", (lp_service_root,), Some(&kwargs))?;
        Ok(())
    })
    .unwrap()
}

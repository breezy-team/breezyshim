//! Launchpad login and related functions
use pyo3::prelude::*;

pub fn login(url: &url::Url) {
    Python::with_gil(|py| -> PyResult<()> {
        let m = py.import_bound("breezy.plugins.launchpad.cmds")?;
        let cmd = m.getattr("cmd_launchpad_login")?;

        let cmd_lp = cmd.call0()?;
        cmd_lp.call_method0("_setup_outf")?;

        cmd_lp.call_method1("run", (url.as_str(),))?;

        let lp_api = py.import_bound("breezy.plugins.launchpad.lp_api")?;

        let lp_uris = launchpadlib::uris()?;

        let lp_service_root = lp_uris
            .iter()
            .find(|(_key, root)| {
                url.host_str() == Some(root) || url.host_str() == Some(root.trim_end_matches('/'))
            })
            .unwrap()
            .1;
        let kwargs = pyo3::types::PyDict::new_bound(py);
        kwargs.set_item("version", "devel")?;
        lp_api.call_method("connect_launchpad", (lp_service_root,), Some(&kwargs))?;
        Ok(())
    })
    .unwrap()
}

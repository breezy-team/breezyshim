//! Launchpad login and related functions
use launchpadlib::uris;
use pyo3::prelude::*;

/// Log in to Launchpad using the provided URL.
///
/// This function authenticates the user with Launchpad via OAuth, allowing
/// subsequent API calls to be made with the authenticated user's credentials.
pub fn login(url: &url::Url) {
    Python::with_gil(|py| -> PyResult<()> {
        let m = py.import("breezy.plugins.launchpad.cmds")?;
        let cmd = m.getattr("cmd_launchpad_login")?;

        let cmd_lp = cmd.call0()?;
        cmd_lp.call_method0("_setup_outf")?;

        cmd_lp.call_method1("run", (url.as_str(),))?;

        let lp_api = py.import("breezy.plugins.launchpad.lp_api")?;

        // The original code extracted a service root like this:
        let lp_uris = lp_api
            .getattr("uris")?
            .call0()?
            .extract::<Vec<(String, String)>>()?;

        let lp_service_root = lp_uris
            .iter()
            .find(|(_key, root)| {
                url.host_str() == Some(root) || url.host_str() == Some(root.trim_end_matches('/'))
            })
            .unwrap()
            .1
            .clone();

        let kwargs = pyo3::types::PyDict::new(py);
        kwargs.set_item("version", "devel")?;
        lp_api.call_method("connect_launchpad", (lp_service_root,), Some(&kwargs))?;
        Ok(())
    })
    .unwrap()
}

// Test function to identify uri function
#[test]
fn test_uri_functions() {
    // Sample URL
    let url_str = "https://launchpad.net/breezy";
    let url = url::Url::parse(url_str).unwrap();

    // Try available functions
    println!("Host: {}", url.host_str().unwrap_or_default());

    // Try lookup_service_root
    let result1 = uris::lookup_service_root("production");
    println!("lookup_service_root('production'): {:?}", result1);

    // Try lookup_web_root
    let result2 = uris::lookup_web_root("production");
    println!("lookup_web_root('production'): {:?}", result2);

    // Try web_root_for_service_root
    let result3 = uris::web_root_for_service_root(&result1.unwrap());
    println!("web_root_for_service_root: {:?}", result3);
}

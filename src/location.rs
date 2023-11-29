use pyo3::prelude::*;
use url::Url;

pub fn cvs_to_url(cvsroot: &str) -> Url {
    Python::with_gil(|py| {
        let breezy_location = py.import("breezy.location").unwrap();

        breezy_location
            .call_method1("cvs_to_url", (cvsroot,))
            .unwrap()
            .extract::<String>()
            .unwrap()
            .parse()
            .unwrap()
    })
}

pub fn rcp_location_to_url(rcp_location: &str) -> Result<Url, String> {
    Python::with_gil(|py| {
        let breezy_location = py.import("breezy.location").unwrap();

        Ok(breezy_location
            .call_method1("rcp_location_to_url", (rcp_location,))
            .map_err(|e| e.to_string())?
            .extract::<String>()
            .unwrap()
            .parse()
            .unwrap())
    })
}

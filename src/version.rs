use pyo3::prelude::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReleaseLevel {
    Dev,
    Alpha,
    Beta,
    Candidate,
    Final,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u32,
    minor: u32,
    micro: u32,
    level: ReleaseLevel,
    serial: u32,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}{}{}",
            self.major,
            self.minor,
            self.micro,
            match self.level {
                ReleaseLevel::Dev => "dev",
                ReleaseLevel::Alpha => "a",
                ReleaseLevel::Beta => "b",
                ReleaseLevel::Candidate => "rc",
                ReleaseLevel::Final => "",
            },
            if self.serial > 0 {
                format!("{}", self.serial)
            } else {
                "".to_string()
            }
        )
    }
}

pub fn version() -> Version {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let m = py.import("breezy").unwrap();

        let version_info = m.getattr("version_info").unwrap();

        let major = version_info.get_item(0).unwrap().extract::<u32>().unwrap();
        let minor = version_info.get_item(1).unwrap().extract::<u32>().unwrap();
        let micro = version_info.get_item(2).unwrap().extract::<u32>().unwrap();
        let level = match version_info
            .get_item(3)
            .unwrap()
            .extract::<String>()
            .unwrap()
            .as_str()
        {
            "dev" => ReleaseLevel::Dev,
            "alpha" => ReleaseLevel::Alpha,
            "beta" => ReleaseLevel::Beta,
            "candidate" => ReleaseLevel::Candidate,
            "final" => ReleaseLevel::Final,
            _ => panic!("Invalid release level"),
        };
        let serial = version_info.get_item(4).unwrap().extract::<u32>().unwrap();

        Version {
            major,
            minor,
            micro,
            level,
            serial,
        }
    })
}

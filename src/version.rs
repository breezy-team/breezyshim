//! Version information for the Breezy library.
use pyo3::prelude::*;

/// The release level of a version.
///
/// This enum represents the different stages of a software release.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReleaseLevel {
    /// Development version.
    Dev,
    /// Alpha version.
    Alpha,
    /// Beta version.
    Beta,
    /// Release candidate.
    Candidate,
    /// Final release.
    Final,
}

/// Version information.
///
/// This struct represents a version number with major, minor, and micro components,
/// a release level, and a serial number.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    /// The major version number.
    major: u32,
    /// The minor version number.
    minor: u32,
    /// The micro (patch) version number.
    micro: u32,
    /// The release level.
    level: ReleaseLevel,
    /// The serial number within the release level.
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

/// Get the version of the Breezy library.
///
/// # Returns
///
/// The version of the Breezy library.
pub fn version() -> Version {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let m = py.import_bound("breezy").unwrap();

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

#[test]
fn test_version_serialize() {
    let v = Version {
        major: 1,
        minor: 2,
        micro: 3,
        level: ReleaseLevel::Final,
        serial: 0,
    };
    assert_eq!(v.to_string(), "1.2.3");
}

#[test]
fn test_version() {
    version().to_string();
}

use anyhow::anyhow;
use anyhow::Result;
use serde::Deserialize;
use std::{fmt::Display, str::FromStr};

use crate::provider::Provider;
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub value: String,
    pub lts: bool,
    pub provider: Provider,
    pub semver_value: Option<String>,
}

impl Version {
    pub fn new(value: u32, is_lts: bool, provider: Provider) -> Self {
        Self {
            value: value.to_string(),
            lts: is_lts,
            semver_value: None,
            provider: provider,
        }
    }

    pub fn new_from_disk(name: &str) -> Self {
        let values: Vec<&str> = name.split('-').collect();

        if values.len() == 2 {
            Self {
                value: values[0].to_string(),
                lts: false,
                provider: Provider::from_str(values[1]).expect("Cannot read version from disk"),
                semver_value: None,
            }
        } else {
            Self {
                value: values[0].to_string(),
                lts: true,
                provider: Provider::from_str(values[2]).expect("Cannot read version from disk"),
                semver_value: None,
            }
        }
    }

    pub fn is_lts(&self) -> bool {
        self.lts
    }

    pub fn as_disk_version(&self) -> String {
        let version_name: &str = self.value.as_ref();
        if self.is_lts() {
            format!("{}-lts-{}", self.value, self.provider)
        } else {
            format!("{}", version_name.to_owned())
        }
    }

    pub fn to_version_number(&self) -> Result<u8> {
        self.value
            .parse::<u8>()
            .map_err(|_e| anyhow!("Cannot parse string to major version"))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use crate::provider::Provider;

    use super::Version;

    #[test]
    fn version_should_display_main_version() {
        let version = Version::new(10, false, crate::provider::Provider::Azul);
        let string_result = format!("{}", version);
        assert_eq!(format!("{}", 10), string_result);
    }

    #[test]
    fn should_extract_major_version_as_number() {
        let version = Version::new_from_disk("8-lts-azul").to_version_number();
        assert!(version.is_ok());
        assert_eq!(8, version.unwrap());
    }

    #[test]
    fn should_create_version_from_disk() {
        let version = Version::new_from_disk("8-lts-azul");
        assert_eq!(version.is_lts(), true);
        assert_eq!(version.provider, Provider::Azul);
    }

    #[test]
    fn should_create_version_without_lts_from_disk() {
        let version = Version::new_from_disk("8-azul");
        assert_eq!(version.is_lts(), false);
        assert_eq!(version.provider, Provider::Azul);
    }
}

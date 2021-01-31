use anyhow::anyhow;
use serde::Deserialize;
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Provider {
    AdoptOpenJDK,
    Azul,
}

impl Provider {
    pub fn as_str(&self) -> String {
        match self {
            Provider::AdoptOpenJDK => "adoptopenjdk".to_owned(),
            Provider::Azul => "azul".to_owned(),
        }
    }
}

impl FromStr for Provider {
    type Err = anyhow::Error;

    fn from_str(provider_name: &str) -> Result<Self, Self::Err> {
        match provider_name {
            "adoptopenjdk" => Ok(Provider::AdoptOpenJDK),
            "azul" => Ok(Provider::Azul),
            _ => Err(anyhow!("Cannot parse provider name {}", provider_name)),
        }
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

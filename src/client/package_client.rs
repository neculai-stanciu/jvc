use crate::{
    config::{ClientConfig, VersionRequirements},
    provider::Provider,
    version::Version,
};
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{adoptopenjdk_client::AdoptOpenJDKClient, azul_client::AzulClient};

#[derive(Debug)]
pub struct DownloadResponse {
    pub download_path: PathBuf,
    pub package_name: String,
}
#[async_trait]
pub trait PackageClient: Sync + Send {
    fn get_base_url(&self) -> String;
    // - get all available versions for package
    async fn get_all_version(&self, requirements: VersionRequirements) -> Result<Vec<Version>>;
    // - download package
    async fn download(&self, client_config: ClientConfig) -> Result<DownloadResponse>;
}

pub fn get_client(provider: &Provider) -> Box<dyn PackageClient> {
    match provider {
        Provider::AdoptOpenJDK => Box::new(AdoptOpenJDKClient::new()),
        Provider::Azul => Box::new(AzulClient::new()),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        client::{adoptopenjdk_client::AdoptOpenJDKClient, azul_client::AzulClient},
        config::VersionRequirements,
        provider::Provider,
    };

    use super::{get_client, PackageClient};

    #[test]
    fn get_client_should_return_correct_value() {
        let client = get_client(&Provider::Azul);
        let expected_client: Box<dyn PackageClient> = Box::new(AzulClient::new());
        assert_eq!(client.get_base_url(), expected_client.get_base_url());

        let adopt_client = get_client(&Provider::AdoptOpenJDK);
        let adopt_expected_client: Box<dyn PackageClient> = Box::new(AdoptOpenJDKClient::new());
        assert_eq!(
            adopt_client.get_base_url(),
            adopt_expected_client.get_base_url()
        );
    }

    #[tokio::test]
    async fn get_all_versions_should_work_for_azul() {
        let client = get_client(&Provider::Azul);
        let versions = client.get_all_version(VersionRequirements::default()).await;
        assert!(versions.is_ok());
    }
    #[tokio::test]
    async fn get_all_versions_should_work_for_adoptopenjdk() {
        let client = get_client(&Provider::AdoptOpenJDK);
        let versions = client.get_all_version(VersionRequirements::default()).await;
        assert!(versions.is_ok());
    }

    #[test]
    fn get_base_url_should_work_for_azul() {
        let client = get_client(&Provider::Azul);
        let base_url = client.get_base_url();
        assert_eq!(
            base_url,
            "https://api.azul.com/zulu/download/community/v1.0".to_owned()
        );
    }

    #[test]
    fn get_base_url_should_work_for_adoptopenjdk() {
        let client = get_client(&Provider::AdoptOpenJDK);
        let base_url = client.get_base_url();
        assert_eq!(base_url, "https://api.adoptopenjdk.net/v3".to_owned());
    }
}

use crate::{
    config::{ClientConfig, VersionRequirements},
    version::Version,
};

use super::package_client::{DownloadResponse, PackageClient};
use async_trait::async_trait;
#[derive(Debug)]
pub struct GithubClient {}

impl GithubClient {
    pub fn new() -> Self {
        GithubClient {}
    }
}

#[async_trait]
impl PackageClient for GithubClient {
    fn get_base_url(&self) -> String {
        "".to_owned()
    }

    async fn get_all_version(
        &self,
        _requirements: VersionRequirements,
    ) -> anyhow::Result<Vec<Version>> {
        let _client = GithubClient::new();
        todo!()
    }

    async fn download(&self, _config: ClientConfig) -> anyhow::Result<DownloadResponse> {
        todo!()
    }
}

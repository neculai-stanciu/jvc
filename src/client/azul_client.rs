use std::collections::HashSet;

use crate::{
    config::{ClientConfig, VersionRequirements},
    version::Version,
};

use super::package_client::{DownloadResponse, PackageClient};
use anyhow::Result;
use async_trait::async_trait;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, warn};
use serde::Deserialize;
use tokio::{fs, io::AsyncWriteExt};

#[derive(Debug)]
pub struct AzulClient;

#[derive(Debug, Deserialize)]
pub struct AzulPackageDetails {
    id: u32,
    url: String,
    arch: String,
    abi: Option<String>,
    hw_bitness: String,
    os: String,
    ext: String,
    bundle_type: String,
    release_status: String,
    support_term: String,
    last_modified: String,
    name: String,
    zulu_version: Vec<u32>,
    jdk_version: Vec<u32>,
    size: u32,
    md5_hash: String,
    sha256_hash: String,
    javafx: bool,
    features: Vec<String>,
}
#[derive(Debug, Deserialize)]
pub struct AzulPackage {
    id: u32,
    name: String,
    url: String,
    jdk_version: Vec<u32>,
    zulu_version: Vec<u32>,
}

impl AzulClient {
    pub fn new() -> Self {
        AzulClient {}
    }
}

#[async_trait]
impl PackageClient for AzulClient {
    fn get_base_url(&self) -> String {
        "https://api.azul.com/zulu/download/community/v1.0".to_owned()
    }

    async fn get_all_version(&self, requirements: VersionRequirements) -> Result<Vec<Version>> {
        let available_releases_url =
            compose_all_version_url(self.get_base_url().as_ref(), requirements);
        debug!("Request all version: {}", &available_releases_url);
        let release_info: Vec<AzulPackage> =
            reqwest::get(&available_releases_url).await?.json().await?;
        self.get_base_url();

        let major_releases: HashSet<_> = release_info.iter().map(|v| v.jdk_version[0]).collect();
        debug!("Receive the following versions: {:?}", major_releases);
        // get all lts releases
        let lts_major_releases = get_lts_releases(available_releases_url.as_ref())
            .await
            .map_err(|_e| warn!("Cannot retrieve support type information for Azul Zulu"));
        debug!(
            "Receive the following lts versions: {:?}",
            lts_major_releases
        );
        let mut versions: Vec<Version> =
            combine_versions_info(major_releases, lts_major_releases.ok());
        versions.sort_by(|v1, v2| {
            v1.value
                .parse::<u32>()
                .unwrap_or(0)
                .cmp(&v2.value.parse::<u32>().unwrap_or(0))
        });
        Ok(versions)
    }

    async fn download(&self, client_conf: ClientConfig) -> Result<DownloadResponse> {
        let version_value = &client_conf.version.value;
        let tmp_dir = &client_conf.download_dir.clone();
        let file_name = tmp_dir.join(version_value);

        let details_url = compose_details_url(
            self.get_base_url().as_ref(),
            version_value.as_ref(),
            client_conf.requirements,
        );
        let package_details: AzulPackageDetails = reqwest::get(&details_url).await?.json().await?;

        let mut response = reqwest::get(&package_details.url).await?;
        debug!(
            "Try to write in temp dir: {}",
            &tmp_dir.as_path().to_str().expect("cannot get dir path")
        );

        let pb = ProgressBar::new(u64::from(package_details.size));
        pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .progress_chars("#>-"));

        let mut dest = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_name)
            .await?;
        while let Some(chunk) = response.chunk().await? {
            dest.write_all(&chunk).await?;
            pb.inc(chunk.len() as u64);
        }
        Ok(DownloadResponse {
            download_path: file_name,
            package_name: package_details.name,
        })
    }
}

pub fn combine_versions_info(
    major_versions: HashSet<u32>,
    lts_major_releases: Option<HashSet<u32>>,
) -> Vec<Version> {
    major_versions
        .iter()
        .map(|v| {
            Version::new(
                *v,
                lts_major_releases.is_some() && lts_major_releases.as_ref().unwrap().contains(v),
                crate::provider::Provider::Azul,
            )
        })
        .collect()
}

fn compose_details_url(base_url: &str, version: &str, requirements: VersionRequirements) -> String {
    format!(
        "{}/bundles/latest/?jdk_version={}&os={}&arch={}&ext=zip&bundle_type={}&release_status={}",
        base_url,
        version,
        requirements.os.unwrap_or_default(),
        requirements.arch.unwrap_or("x86".to_owned()),
        requirements.image_type.unwrap_or("jdk".to_owned()),
        requirements.release_type.unwrap_or("ga".to_owned())
    )
}

fn compose_all_version_url(base_url: &str, requirements: VersionRequirements) -> String {
    format!(
        "{}/bundles/?os={}&arch={}&ext=zip&bundle_type={}&release_status={}",
        base_url,
        requirements.os.unwrap_or_default(),
        requirements.arch.unwrap_or("x86".to_owned()),
        requirements.image_type.unwrap_or("jdk".to_owned()),
        requirements.release_type.unwrap_or("ga".to_owned())
    )
}

async fn get_lts_releases(bundles_url: &str) -> Result<HashSet<u32>> {
    let available_releases_url = format!("{}&support_term=lts", bundles_url);
    let release_info: Vec<AzulPackage> =
        reqwest::get(&available_releases_url).await?.json().await?;
    debug!("release_info: {:?}", release_info);
    let lts_releases: HashSet<u32> = release_info.iter().map(|v| v.jdk_version[0]).collect();
    Ok(lts_releases)
}

#[cfg(test)]
mod tests {
    use crate::config::VersionRequirements;

    use super::{compose_all_version_url, compose_details_url};

    const BASE_URL: &str = "https://api.azul.com/zulu/download/community/v1.0";

    fn setup() -> VersionRequirements {
        VersionRequirements {
            arch: Some("x86".to_owned()),
            os: Some("windows".to_owned()),
            image_type: Some("jdk".to_owned()),
            release_type: Some("ga".to_owned()),
            jvm_impl: None,
            heap_size: None,
            vendor: None,
            project: None,
        }
    }

    #[test]
    fn test_url_creation() {
        let url = compose_details_url(BASE_URL, "10", setup());
        let expected_url = format!("{}/bundles/latest/?jdk_version=10&os=windows&arch=x86&ext=zip&bundle_type=jdk&release_status=ga",
        BASE_URL);
        assert_eq!(url, expected_url)
    }

    #[test]
    fn test_url_all_version_creation() {
        let url = compose_all_version_url(BASE_URL, setup());
        let expected_url = format!(
            "{}/bundles/?os=windows&arch=x86&ext=zip&bundle_type=jdk&release_status=ga",
            BASE_URL
        );
        assert_eq!(url, expected_url)
    }
}

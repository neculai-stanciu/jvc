use crate::{
    config::{self},
    version::Version,
};

use super::package_client::{DownloadResponse, PackageClient};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use config::{ClientConfig, VersionRequirements};
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use reqwest::{redirect::Policy, Url};
use serde::Deserialize;
use tokio::{fs, io::AsyncWriteExt};

#[derive(Debug)]
pub struct AdoptOpenJDKClient;

#[derive(Debug, Deserialize)]
struct BinariesInfo {
    binaries: Vec<BinaryInfo>,
    download_count: u32,
    id: String,
    release_link: String,
    release_name: String,
    release_type: String,
    timestamp: String,
    updated_at: String,
    vendor: String,
    version_data: VersionInfo,
}
#[derive(Debug, Deserialize)]
struct VersionInfo {
    adopt_build_number: Option<u32>,
    build: u32,
    major: u32,
    minor: u32,
    openjdk_version: String,
    security: u32,
    semver: String,
}
#[derive(Debug, Deserialize)]
struct BinaryInfo {
    architecture: String,
    download_count: u32,
    heap_size: String,
    image_type: String,
    installer: Option<InstallerInfo>,
    jvm_impl: String,
    os: String,
    package: PackageInfo,
    project: String,
    scm_ref: Option<String>,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct InstallerInfo {
    checksum: String,
    checksum_link: String,
    download_count: u32,
    link: String,
    name: String,
    size: u32,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    checksum: String,
    checksum_link: String,
    download_count: u32,
    link: String,
    name: String,
    size: u32,
}

#[allow(dead_code)]
impl AdoptOpenJDKClient {
    pub fn new() -> Self {
        AdoptOpenJDKClient {}
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct AdoptOpenJDKVersion {
    pub available_lts_releases: Vec<u32>,
    pub available_releases: Vec<u32>,
    pub most_recent_feature_release: u32,
    pub most_recent_feature_version: u32,
    pub most_recent_lts: u32,
    pub tip_version: u32,
}

#[async_trait]
impl PackageClient for AdoptOpenJDKClient {
    fn get_base_url(&self) -> String {
        "https://api.adoptopenjdk.net/v3".to_owned()
    }

    async fn get_all_version(&self, _requirements: VersionRequirements) -> Result<Vec<Version>> {
        let available_releases = format!("{}/info/available_releases", self.get_base_url());
        debug!("Request all available versions: {}", available_releases);
        let result: AdoptOpenJDKVersion = reqwest::get(&available_releases).await?.json().await?;

        convert_adoptopenjdk_to_version(result)
    }

    async fn download(&self, client_conf: ClientConfig) -> Result<DownloadResponse> {
        let binary_info_url = download_package_url(
            client_conf.base_url.to_owned(),
            client_conf.requirements,
            &client_conf.version,
        );

        let binary_info: Vec<BinariesInfo> = reqwest::get(binary_info_url).await?.json().await?;
        let bin = binary_info
            .first()
            .ok_or(anyhow!("Cannot get first element"))?;
        let total_size: u32 = get_total_size(&bin);
        let package_name: &str = get_package_name(&bin);

        let package_link: &str = bin
            .binaries
            .first()
            .ok_or(anyhow!("Cannot extract package link"))?
            .package
            .link
            .as_ref();

        let package_url = get_link_after_redirect(package_link).await?;
        let tmp_dir = client_conf.download_dir;
        let mut response = reqwest::get(package_url).await?;
        let file_name = tmp_dir.join(client_conf.version.value);
        debug!(
            "Try to write in temp dir: {}",
            &tmp_dir.as_path().to_str().expect("cannot get dir path")
        );

        let pb = ProgressBar::new(u64::from(total_size));
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
            package_name: package_name.to_owned(),
        })
    }
}

fn get_total_size(binary_info: &BinariesInfo) -> u32 {
    let bin = binary_info
        .binaries
        .first()
        .expect("Cannot extract package details");
    bin.package.size
}
fn get_package_name(binary_info: &BinariesInfo) -> &str {
    let bin = binary_info
        .binaries
        .first()
        .expect("Cannot extract package details");
    bin.package.name.as_ref()
}

fn convert_adoptopenjdk_to_version(result: AdoptOpenJDKVersion) -> Result<Vec<Version>> {
    let lts_versions = &result.available_lts_releases;
    let versions: Vec<Version> = result
        .available_releases
        .iter()
        .map(|v| {
            Version::new(
                *v,
                lts_versions.contains(v),
                crate::provider::Provider::AdoptOpenJDK,
            )
        })
        .collect();
    Ok(versions)
}

async fn get_link_after_redirect(package_link: &str) -> Result<Url> {
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()?;

    let response = client.get(package_link).send().await?;

    if response.status() == 302 {
        let location_value = response
            .headers()
            .get("Location")
            .ok_or(anyhow!("Cannot retrieve download url for selected version"))?
            .to_str()?;
        let error_message = format!("Cannot parse url from string {}", location_value);
        let location_url = Url::parse(location_value).expect(error_message.as_ref());
        return Ok(location_url);
    } else {
        Err(anyhow!("Cannot compose download url for selected version"))
    }
}

fn download_package_url(
    api_base_url: String,
    requirements: VersionRequirements,
    version: &Version,
) -> Url {
    let url = compose_download_url(requirements, api_base_url, version);
    debug!("Composed url is: {}", &url.to_string());
    url
}

fn compose_download_url(
    requirements: VersionRequirements,
    api_base_url: String,
    version: &Version,
) -> Url {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "mac") {
        "mac"
    } else {
        "linux"
    };

    Url::parse(&format!(
        "{}/assets/feature_releases/{}/{}?architecture={}&heap_size={}&image_type={}&jvm_impl={}&os={}&page=0&page_size=1&project={}&sort_method=DEFAULT&sort_order=DESC&vendor={}",
        api_base_url,
        version,
        requirements.release_type.unwrap_or("ga".to_owned()),
        requirements.arch.unwrap_or("x64".to_owned()),
        requirements.heap_size.unwrap_or("normal".to_owned()),
        requirements.image_type.unwrap_or("jdk".to_owned()),
        requirements.jvm_impl.unwrap_or("hotspot".to_owned()),
        requirements.os.unwrap_or(os.to_owned()),
        requirements.project.unwrap_or("jdk".to_owned()),
        requirements.vendor.unwrap_or("adoptopenjdk".to_owned()),
        )).expect("Cannot create download url!")
}

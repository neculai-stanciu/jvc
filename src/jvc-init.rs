#![cfg(windows)]

mod loglevel;

use std::{
    env::temp_dir,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::anyhow;
use anyhow::Result;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use log::{debug, error, info, warn};
use loglevel::LogLevel;
use std::io::Write;
use structopt::StructOpt;
use tokio::{fs, io::AsyncWriteExt};
use winreg::{
    enums::{RegType, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE},
    RegKey, RegValue,
};

/// A cli application to handle all java version for windows, linux and macos
#[derive(Debug, StructOpt)]
#[structopt(name = "jvc-init")]
pub struct Cli {
    /// Version to install
    #[structopt(long = "release", short = "r", default_value = "latest")]
    pub release: String,

    /// The root directory for jvc. This will contain aliases, all downloaded versions and optional config.
    #[structopt(long = "install-dir", short = "d")]
    pub install_dir: Option<PathBuf>,
}

fn init_logging(log_level: &LogLevel) {
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .filter(Some("jvc"), log_level.into())
        .filter(Some("jvc-init"), log_level.into())
        .init();
}

async fn install_jvc(release_tag: Option<String>, install_dir: PathBuf) -> Result<()> {
    let filename = "jvc-win-amd64.exe".to_owned();
    let download_url = if let Some(tag) = release_tag {
        format!(
            "https://github.com/neculai-stanciu/jvc/releases/{}/download/{}",
            tag, filename
        )
    } else {
        format!(
            "https://github.com/neculai-stanciu/jvc/releases/latest/download/{}",
            filename
        )
    };

    let download_dir = temp_dir();
    download_binary(
        download_url.as_ref(),
        download_dir.as_path(),
        filename.as_ref(),
    )
    .await?;
    let download_binary_path = download_dir.join(filename.clone());
    let installation_dir = install_dir.join("jvc.exe");
    move_to_install_dir(download_binary_path, installation_dir.as_path()).await?;
    add_to_path(installation_dir).await?;

    Ok(())
}

async fn add_to_path(binary_path: PathBuf) -> Result<()> {
    print!("Add to path not implemented yet! {:?}", binary_path);
    let value = binary_path
        .as_os_str()
        .to_str()
        .ok_or(anyhow!("Cannot transform os path to string"))?;
    set_persistent_path(value).await?;
    Ok(())
}

async fn move_to_install_dir(from: PathBuf, install_dir: &Path) -> Result<u64> {
    info!(
        "Try to copy from {:?} to {:?}",
        from.as_path().as_os_str(),
        install_dir.as_os_str()
    );
    std::fs::copy(from, install_dir)
        .map_err(|e| anyhow!("Cannot copy binary to installation dir \n {:?}", e))
}

async fn download_binary(
    download_url: &str,
    download_path: &Path,
    binary_name: &str,
) -> Result<()> {
    info!("Starting download binary: {}", download_url);

    let file_name = download_path.join(binary_name);
    let mut response = reqwest::get(download_url).await?;
    info!(
        "Try to write in temp dir: {} - response status {}",
        file_name.to_str().expect("cannot get dir path"),
        &response.status()
    );
    if response.status().as_u16() != 200u16 {
        error!("Binary path has changed. Open in issue on https://github.com/neculai-stanciu/jvc/issues for this.");
    }
    let binary_size = response
        .content_length()
        .ok_or(anyhow!("Cannot find binary size"))?;

    let pb = ProgressBar::new(binary_size);
    pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .progress_chars("#>-"));
    if file_name.exists() {
        fs::remove_file(&file_name).await?;
    }
    let mut dest = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_name)
        .await?;
    while let Some(chunk) = response.chunk().await? {
        dest.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }

    Ok(())
}

/// Encodes a utf-8 string as a null-terminated UCS-2 string in bytes
#[cfg(windows)]
pub fn string_to_winreg_bytes(s: &str) -> Vec<u8> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    let v: Vec<u16> = OsStr::new(s).encode_wide().chain(Some(0)).collect();
    unsafe { std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), v.len() * 2).to_vec() }
}

// This is used to decode the value of HKCU\Environment\PATH. If that
// key is not unicode (or not REG_SZ | REG_EXPAND_SZ) then this
// returns null.  The winreg library itself does a lossy unicode
// conversion.
#[cfg(windows)]
pub fn string_from_winreg_value(val: &winreg::RegValue) -> Option<String> {
    use std::slice;

    match val.vtype {
        RegType::REG_SZ | RegType::REG_EXPAND_SZ => {
            // Copied from winreg
            let words = unsafe {
                #[allow(clippy::cast_ptr_alignment)]
                slice::from_raw_parts(val.bytes.as_ptr().cast::<u16>(), val.bytes.len() / 2)
            };
            String::from_utf16(words).ok().map(|mut s| {
                while s.ends_with('\u{0}') {
                    s.pop();
                }
                s
            })
        }
        _ => None,
    }
}

// Get the windows PATH variable out of the registry as a String. If
// this returns None then the PATH variable is not unicode and we
// should not mess with it.
fn get_windows_path_var() -> Result<Option<String>> {
    use std::io;
    let root = RegKey::predef(HKEY_CURRENT_USER);
    let environment = root.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

    let reg_value = environment.get_raw_value("PATH");
    match reg_value {
        Ok(val) => {
            if let Some(s) = string_from_winreg_value(&val) {
                Ok(Some(s))
            } else {
                warn!("the registry key HKEY_CURRENT_USER\\Environment\\PATH does not contain valid Unicode. \
                    Not modifying the PATH variable");
                Ok(None)
            }
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(Some(String::new())),
        Err(_e) => Err(anyhow!("Cannot read path")),
    }
}

async fn set_persistent_path(value: &str) -> Result<()> {
    println!("Extend path with value: {}", value);
    let root = RegKey::predef(HKEY_CURRENT_USER);
    let current_path_value = get_windows_path_var()?;

    if let Some(path_val) = current_path_value {
        if path_val.contains(value) {
            info!("Ignore adding value because is already configured");
            return Ok(());
        }
        debug!("Path values: \n {}", path_val);
        let final_path = format!("{};{}", value, path_val);
        debug!("End value: \n {}", final_path);
        let environment = root.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

        let reg_value = RegValue {
            bytes: string_to_winreg_bytes(final_path.as_ref()),
            vtype: RegType::REG_EXPAND_SZ,
        };
        environment.set_raw_value("PATH", &reg_value)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = Instant::now();
    let args: Cli = Cli::from_args();
    init_logging(&LogLevel::Info);
    let install_dir = args.install_dir.unwrap_or(
        dirs::home_dir()
            .ok_or(anyhow!(
                "Cannot get home dir. Please specify your installation dir option."
            ))?
            .join(".jvc"),
    );

    install_jvc(Some(args.release), install_dir).await?;

    let end = start_time.elapsed();
    debug!("Time elapsed is: {}", HumanDuration(end));
    Ok(())
}

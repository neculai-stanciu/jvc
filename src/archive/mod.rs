use anyhow::{anyhow, Result};
use flate2::bufread::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use std::{ffi::OsStr, fs::File, io::BufReader, path::Path};

pub fn unarchive<T: AsRef<Path>>(package_name: &str, from: T, to: T) -> Result<()> {
    if let Some(ext) = get_extension_from_filename(package_name) {
        match ext {
            "zip" => unarchive_zip(from, to),
            "gz" => unarchive_gzip(from, to),
            _ => Err(anyhow!("Unknown archive type")),
        }
    } else {
        Err(anyhow!("Cannot unarchive this type"))
    }
}

// zip -> examples -> extract
fn unarchive_zip<T: AsRef<Path>>(from: T, to: T) -> Result<()> {
    debug!("Try to unarchive zip to: {:?}", to.as_ref());
    let file = File::open(from)?;
    let file_size = file.metadata()?.len();
    let buf_reader = BufReader::new(file);
    let pb = ProgressBar::new(file_size);
    pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .progress_chars("#>-"));

    let mut archive = zip::ZipArchive::new(pb.wrap_read(buf_reader))?;
    archive.extract(to.as_ref())?;
    debug!("Finished unarchive zip");
    Ok(())
}
fn unarchive_gzip<T: AsRef<Path>>(from: T, to: T) -> Result<()> {
    debug!("Try to unarchive gzip: {:?}", to.as_ref());
    let file = File::open(from)?;
    let file_size = file.metadata()?.len();
    let buf_reader = BufReader::new(file);
    let pb = ProgressBar::new(file_size);
    pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .progress_chars("#>-"));
    let gz_stream = GzDecoder::new(buf_reader);
    let mut tar_archive = tar::Archive::new(pb.wrap_read(gz_stream));
    tar_archive.unpack(to.as_ref())?;
    debug!("Finished unarchive gzip");
    Ok(())
}

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

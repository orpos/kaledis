use std::{io::{Cursor, Write}, path::{Path, PathBuf}};

use tokio::{fs::File, io::AsyncReadExt};
use zip::{result::ZipResult, write::SimpleFileOptions, ZipWriter};

pub async fn add_zip_f_from_buf(
    zip: &mut ZipWriter<Cursor<Vec<u8>>>,
    name: &str,
    buffer: &[u8]
) -> ZipResult<()> {
    zip.start_file(
        name,
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored)
    )?;
    zip.write(buffer)?;
    Ok(())
}

pub async fn add_zip_f_from_path(
    zip: &mut ZipWriter<Cursor<Vec<u8>>>,
    name: &Path,
    prefix: &PathBuf
) -> anyhow::Result<()> {
    zip.start_file_from_path(
        name.strip_prefix(prefix)?,
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored)
    )?;
    let mut file = File::open(name).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    zip.write(&buffer)?;
    Ok(())
}
pub async fn copy_zip_f_from_path(
    zip: &mut ZipWriter<Cursor<Vec<u8>>>,
    name: &Path,
    output: PathBuf
) -> anyhow::Result<()> {
    zip.start_file_from_path(
        output,
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored)
    )?;
    let mut file = File::open(name).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    zip.write(&buffer)?;
    Ok(())
}
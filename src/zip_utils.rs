use std::{
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

use tokio::{fs::File, io::{self, AsyncReadExt}};
use walkdir::WalkDir;
use zip::{result::ZipResult, write::SimpleFileOptions, ZipWriter};

pub struct Zipper {
    inner: ZipWriter<Cursor<Vec<u8>>>,
}

impl Zipper {
    pub fn new() -> Zipper {
        Self {
            inner: ZipWriter::new(Cursor::new(Vec::new())),
        }
    }
    pub fn finish(self) -> Vec<u8> {
        let data = self.inner.finish().unwrap();
        data.into_inner()
    }
    pub async fn add_zip_f_from_buf(&mut self, name: &str, buffer: &[u8]) -> ZipResult<()> {
        let zip = &mut self.inner;
        zip.start_file(
            name,
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored),
        )?;
        zip.write(buffer)?;
        Ok(())
    }

    pub fn add_zip_f_from_path(
        &mut self,
        name: &Path,
        prefix: &PathBuf,
    ) -> anyhow::Result<()> {
        self.copy_zip_f_from_path(name, name.strip_prefix(prefix)?.to_path_buf())
    }
    pub fn copy_zip_f_from_path(
        &mut self,
        name: &Path,
        output: PathBuf,
    ) -> anyhow::Result<()> {
        let zip = &mut self.inner;
        zip.start_file_from_path(
            output,
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored),
        )?;
        let mut file = std::fs::File::open(name)?;
        std::io::copy(&mut file, zip)?;
        Ok(())
    }
    pub fn put_folder_recursively(
        &mut self,
        root: PathBuf,
        prefix: Option<PathBuf>
    ) -> anyhow::Result<()> {
        if !root.is_dir() {
            anyhow::bail!("Not a directory");
        };
        for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
            let path = entry.path();

            let zip_path = path.strip_prefix(path).unwrap().join(
                prefix.as_ref().unwrap_or(&PathBuf::new())
            );

            if zip_path.as_os_str().is_empty() {
                continue; // root
            }
            let name = zip_path.to_string_lossy();
            let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            if path.is_dir() {
                self.inner.add_directory(name.to_string(),options)?;
            }
            else {
                self.add_zip_f_from_path(path, &root)?;
            }
        }
        Ok(())
    }
}

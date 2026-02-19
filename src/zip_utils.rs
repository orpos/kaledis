use std::{
    io::{Cursor, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
use zip::{ZipWriter, result::ZipResult, write::SimpleFileOptions};

pub struct Zipper {
    pub inner: ZipWriter<Cursor<Vec<u8>>>,
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
    // pub async fn add_buffer(&mut self, name: &str, buffer: &[u8]) -> ZipResult<()> {
    //     let zip = &mut self.inner;
    //     zip.start_file(
    //         name,
    //         SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored),
    //     )?;
    //     let _ = zip.write(buffer)?;
    //     Ok(())
    // }

    // Copies an file into a zip stripping it's root
    pub fn add_rootless(&mut self, name: &Path, root: &PathBuf) -> color_eyre::Result<()> {
        self.copy_from_path(name, name.strip_prefix(root)?.to_path_buf())
    }
    pub fn copy_from_path(&mut self, name: &Path, output: PathBuf) -> color_eyre::Result<()> {
        let zip = &mut self.inner;
        zip.start_file_from_path(
            output,
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored),
        )?;
        let mut file = std::fs::File::open(name)?;
        std::io::copy(&mut file, zip)?;
        Ok(())
    }
    pub fn put_folder_recursively(&mut self, folder: &PathBuf) -> color_eyre::Result<()> {
        if !folder.is_dir() {
            return Err(color_eyre::eyre::eyre!("Not a directory"));
        };
        for entry in WalkDir::new(&folder).into_iter().filter_map(Result::ok) {
            let path = entry.path();

            let zip_path = path.strip_prefix(&folder).unwrap();
            // .join(root.as_ref().unwrap_or(&PathBuf::new()));

            if zip_path.as_os_str().is_empty() {
                continue; // root
            }

            let name = zip_path.to_string_lossy();
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            if path.is_dir() {
                self.inner.add_directory(name.to_string(), options)?;
            } else {
                self.add_rootless(path, &folder)?;
            }
        }
        Ok(())
    }
}

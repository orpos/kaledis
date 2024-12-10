use std::{ io::{ Cursor, Write }, path::{ Path, PathBuf } };

use tokio::{ fs::File, io::AsyncReadExt };
use zip::{ result::ZipResult, write::SimpleFileOptions, ZipWriter };

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
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored)
        )?;
        zip.write(buffer)?;
        Ok(())
    }

    pub async fn add_zip_f_from_path(
        &mut self,
        name: &Path,
        prefix: &PathBuf
    ) -> anyhow::Result<()> {
        self.copy_zip_f_from_path(name, name.strip_prefix(prefix)?.to_path_buf()).await
    }
    pub async fn copy_zip_f_from_path(
        &mut self,
        name: &Path,
        output: PathBuf
    ) -> anyhow::Result<()> {
        let zip = &mut self.inner;
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
}

use std::io::{Cursor, Read, Write};

use backhand::{
    FilesystemCompressor, FilesystemReader, FilesystemWriter, InnerNode, NodeHeader,
    compression::Compressor, kind::Kind,
};
use fs_err::tokio::create_dir_all;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{commands::build::Builder, home_manager::Target};

fn is_valid_superblock(data: &[u8]) -> bool {
    if data.len() < 96 {
        return false;
    }

    let read_u16 = |o: usize| u16::from_le_bytes(data[o..o + 2].try_into().unwrap());
    let read_u32 = |o: usize| u32::from_le_bytes(data[o..o + 4].try_into().unwrap());

    let version_major = read_u16(28);
    let version_minor = read_u16(30);
    let block_size = read_u32(12);
    let compressor = read_u16(20);
    let block_log = read_u16(22);

    // SquashFS v4.0 only
    if version_major != 4 || version_minor != 0 {
        return false;
    }

    // block_size must be power of 2 between 4096 and 1MB
    if block_size < 4096 || block_size > 1048576 {
        return false;
    }
    if block_size & (block_size - 1) != 0 {
        return false;
    }

    // block_log must match block_size (log2)
    if (1u32 << block_log) != block_size {
        return false;
    }

    // compressor must be a known value (1=gzip, 2=lzma, 3=lzo, 4=xz, 5=lz4, 6=zstd)
    if compressor == 0 || compressor > 6 {
        return false;
    }

    true
}

fn find_valid_squashfs_offset(data: &[u8]) -> Option<usize> {
    let mut pos = 0;

    while pos + 96 < data.len() {
        // Look for next magic occurrence
        if let Some(rel) = data[pos..].windows(4).position(|w| w == b"hsqs") {
            let offset = pos + rel;

            // Validate the superblock at this offset
            if is_valid_superblock(&data[offset..]) {
                return Some(offset);
            }

            // Not valid, keep scanning after this position
            pos = offset + 4;
        } else {
            break;
        }
    }

    None
}

fn skip_file_from_squashfs<'a>(
    reader: &FilesystemReader,
    skip_path: &'a str,
) -> color_eyre::Result<FilesystemWriter<'a, 'a, 'a>> {
    let mut writer = FilesystemWriter::default();

    for node in reader.files() {
        // Skip the file we want to remove
        if node.fullpath == std::path::Path::new(skip_path) {
            continue;
        }

        match &node.inner {
            InnerNode::File(f) => {
                let mut buf = Vec::new();
                reader.file(&f).reader().read_to_end(&mut buf)?;
                writer.push_file(
                    std::io::Cursor::new(buf),
                    &node
                        .fullpath
                        .to_string_lossy()
                        .trim_start_matches('/')
                        .trim_start_matches("./"),
                    node.header.clone(),
                )?;
            }
            InnerNode::Dir(_) => {
                if node.fullpath == std::path::Path::new(".")
                    || node.fullpath == std::path::Path::new("/")
                    || node.fullpath.as_os_str().is_empty()
                {
                    continue;
                }
                writer.push_dir(
                    &node
                        .fullpath
                        .to_string_lossy()
                        .trim_start_matches('/')
                        .trim_start_matches("./"),
                    node.header.clone(),
                )?;
            }
            InnerNode::Symlink(s) => {
                writer.push_symlink(&s.link, &node.fullpath, node.header.clone())?;
            }
            _ => {}
        }
    }

    Ok(writer)
}

// Returns both the AppImage and the squashfs
fn extract_squashfs_from_appimage(
    mut data: Vec<u8>,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let offset = find_valid_squashfs_offset(&data).ok_or("SquashFS magic not found in AppImage")?;

    let squashfs = data.split_off(offset);

    Ok((data, squashfs))
}

pub async fn build_linux(builder: &Builder, data: &[u8]) -> color_eyre::Result<()> {
    let dists = builder
        .paths
        .dist
        .join(Target::LinuxAppImage.as_ref().to_string());
    let mut file = fs_err::tokio::File::open(
        builder
            .home
            .get_path(&builder.config.love, Target::LinuxAppImage)
            .await
            .join("love2d.AppImage"),
    )
    .await?;

    let mut image_data = vec![];
    file.read_to_end(&mut image_data).await?;

    let (appimage, squashfs) = extract_squashfs_from_appimage(image_data).unwrap();

    let reader = FilesystemReader::from_reader_with_offset_and_kind(
        Cursor::new(squashfs),
        0,
        Kind::from_target("le_v4_0").unwrap(),
    )?;

    let mut bts = vec![];
    for node in reader.files() {
        if node.fullpath == std::path::Path::new("/bin/love")
            && let InnerNode::File(file_reader) = &node.inner
        {
            let mut reader = reader.file(&file_reader).reader();
            reader.read_to_end(&mut bts)?;
            break;
        }
    }

    let mut writer = skip_file_from_squashfs(&reader, "/bin/love").unwrap();
    // The AppImage of love2d doesn't support xz
    writer.set_compressor(FilesystemCompressor::new(Compressor::Zstd, None).unwrap());

    bts.extend_from_slice(&data);
    writer.push_file(
        std::io::Cursor::new(bts),
        "bin/love",
        NodeHeader {
            // This is equivalent of chmod +x for the executable
            permissions: 0o755,
            ..Default::default()
        },
    )?;

    create_dir_all(&dists).await?;

    let mut output_file = fs_err::tokio::File::create(
        dists.join(format!("{}.AppImage", builder.config.project_name)),
    )
    .await?;

    output_file.write_all(&appimage).await?;
    let mut data = Cursor::new(vec![]);
    writer.write(&mut data)?;
    output_file.write_all(data.get_ref()).await?;
    Ok(())
}

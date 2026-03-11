use std::{
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};

use backhand::{
    FilesystemCompressor, FilesystemReader, FilesystemWriter, InnerNode, NodeHeader,
    compression::Compressor, kind::Kind,
};
use color_eyre::{Section, eyre::Context};
use fs_err::tokio::{File, create_dir_all};
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
    skip_paths: Vec<PathBuf>,
    skip_dir_icon: bool,
) -> color_eyre::Result<FilesystemWriter<'a, 'a, 'a>> {
    let mut writer = FilesystemWriter::default();

    for node in reader.files() {
        // Skip the file we want to remove
        if skip_paths.contains(&node.fullpath) {
            continue;
        }
        if node.fullpath.ends_with(".DirIcon") && skip_dir_icon {
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
    let original = builder
        .home
        .get_path(&builder.config.love, Target::LinuxAppImage)
        .await
        .join("love2d.AppImage");
    let mut file = fs_err::tokio::File::open(original).await?;

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

    let mut to_skip = vec![
        Path::new("/bin/love").to_path_buf(),
        Path::new("/love.desktop").to_path_buf(),
    ];
    if builder.config.icon.is_some() {
        to_skip.push(Path::new("/love.svg").to_path_buf());
    }
    // let icon_path = builder.paths.root.join(icon);
    let mut writer =
        skip_file_from_squashfs(&reader, to_skip, builder.config.icon.is_some()).unwrap();
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
    let declaration = format!(
        r#"[Desktop Entry]
Name={}
Comment={}
MimeType=application/x-love-game;
Exec=/home/runner/work/love/love/installdir/bin/love %f
Type=Application
Categories=Development;Game;
Terminal=false
Icon=love
NoDisplay=true"#,
        builder.config.project_name, builder.config.description
    );
    writer.push_file(
        std::io::Cursor::new(declaration.as_bytes().to_vec()),
        "/love.desktop",
        NodeHeader::default(),
    )?;

    if let Some(icon_pth) = &builder.config.icon {
        let mut icon = File::open(builder.paths.root.join(icon_pth)).await?;
        let mut data = vec![];
        icon.read_to_end(&mut data).await?;

        let pth = Path::new(icon_pth);

        writer.push_file(
            std::io::Cursor::new(data),
            PathBuf::new()
                .join("love")
                .with_extension(pth.extension().expect("Icon should have extension")),
            NodeHeader::default(),
        )?;
        writer.push_symlink(
            PathBuf::new()
                .join("love")
                .with_extension(pth.extension().expect("Icon should have extension")),
            PathBuf::new().join(".DirIcon"),
            NodeHeader::default(),
        )?;
    }

    for pattern in &builder.config.layout.external {
        for path in glob::glob(&builder.paths.root.join(pattern).to_string_lossy())
            .context("Building for windows")
            .expect("Failed to parse glob")
            .filter_map(Result::ok)
        {
            let output = PathBuf::new().join("bin").join(
                path.strip_prefix(&builder.paths.root)
                    .context("Building for windows")
                    .suggestion("Don't use assets outside the root of your project")
                    .expect("Failed to strip root"),
            );
            let data = fs_err::tokio::read(&path).await?;
            writer.push_dir_all(output.parent().unwrap(), NodeHeader::default())?;
            writer.push_file(std::io::Cursor::new(data), output, NodeHeader::default())?;
        }
    }

    create_dir_all(&dists).await?;

    fs_err::tokio::remove_file(dists.join("love2d.AppImage")).await?;
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

#![windows_subsystem = "windows"]

use std::{
    env::current_exe,
    fs::{
        copy,
        create_dir_all,
        read_dir,
        read_to_string,
        File,
    },
    io::{
        BufReader,
        Read,
    },
    path::{
        Path,
        PathBuf,
    },
};
use toml::from_str;

fn is_same_file(src: &Path, target: &Path) -> Result<bool, std::io::Error> {
    if !target.exists() {
        return Ok(false);
    }

    let src_content = File::open(src)?;
    let target_content = File::open(target)?;

    if src_content.metadata()?.len() != target_content.metadata()?.len() {
        return Ok(false);
    }

    let src_content = BufReader::new(src_content);
    let target_content = BufReader::new(target_content);

    for (src_byte, target_byte) in src_content
        .bytes()
        .zip(target_content.bytes())
    {
        if src_byte? != target_byte? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn read(config: &Config, dir: &PathBuf) -> std::io::Result<()> {
    let mut entries: Vec<PathBuf> = vec![];
    for entry in read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            if config
                .disallowed_dirs
                .iter()
                .any(|d| path.ends_with(d))
            {
                continue
            }
            read(config, &path)?;
        } else {
            if config
                .exclusion_markers
                .iter()
                .any(|file| path.ends_with(file))
            {
                return Ok(())
            }
            entries.push(path);
        }
    }
    for path in entries {
        let prefix = &path
            .components()
            .next()
            .unwrap();
        let prefix: PathBuf = [format!(
            "{}\\",
            Path::new(prefix).display()
        )]
        .iter()
        .collect();

        let common_path = path
            .strip_prefix(&prefix)
            .unwrap()
            .strip_prefix(&config.src)
            .unwrap();

        let new_path = prefix
            .join(Path::new(&config.dst))
            .join(common_path);

        if is_same_file(
            path.as_path(),
            new_path.as_path(),
        )? {
            continue
        }

        create_dir_all(new_path.parent().unwrap())?;
        copy(path, new_path)?;
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct Config {
    src: String,
    dst: String,
    disallowed_dirs: Vec<String>,
    exclusion_markers: Vec<String>,
}

fn main() -> std::io::Result<()> {
    if let Ok(exe_path) = current_exe() {
        let config: Config = from_str(&read_to_string(
            "./store-config.toml",
        )?)
        .unwrap();
        read(
            &config,
            &exe_path
                .parent()
                .unwrap()
                .join(&config.src),
        )?;
    }
    Ok(())
}

#![windows_subsystem = "windows"]

use std::{
    env::current_exe,
    fmt::Debug,
    fs::{
        copy,
        create_dir_all,
        read_dir,
        read_to_string,
        File,
    },
    io::Write,
    os::windows::fs::symlink_file,
    path::{
        Path,
        PathBuf,
    },
};
use toml::from_str;

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

        create_dir_all(new_path.parent().unwrap())?;
        match symlink_file(&path, &new_path) {
            Ok(_) => continue,
            Err(err) => log_err("Couldn't symlink", &new_path, err)?,
        };

        match copy(&path, &new_path) {
            Ok(_) => continue,
            Err(err) => log_err("Couldn't copy", &new_path, err)?,
        };
    }
    Ok(())
}

fn log_err<E>(str: &str, path: &Path, err: E) -> std::io::Result<()>
where
    E: Debug,
{
    let mut file = File::create("store.log")?;
    let logstr = format!(
        "\n{:?}\n{} {}\n{:?}\n",
        chrono::offset::Local::now(),
        str,
        path.display(),
        err
    );
    file.write_all(logstr.as_bytes())?;
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
        let config: Config =
            from_str(&read_to_string("./store-config.toml")?).unwrap();
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

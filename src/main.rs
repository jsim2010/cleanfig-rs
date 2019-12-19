use {
    std::{env::{self, VarError}, fs, path::PathBuf, process, io, os::windows},
    displaydoc::Display as DisplayDoc,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let home = PathBuf::from(env::var("HOME")?);
    let root = home.join(".config/cleanfig");

    for possible_entry in fs::read_dir(root).map_err(|_| Error::MissingRoot)? {
        let entry = possible_entry?;

        match entry.file_name().to_str().unwrap() {
            "alacritty.yml" => {
                link_file(entry.path(), home.join("AppData/Roaming/alacritty/alacritty.yml"))?;
            }
            "nvim" => {
                let path = home.join("AppData/Local/nvim");

                if !path.exists() {
                    windows::fs::symlink_dir(entry.path(), path).map_err(|_| Error::InvalidPrivilege)?;
                }
            }
            "starship.toml" => {
                link_file(entry.path(), home.join(".config/starship.toml"))?;
            }
            "topgrade.toml" => {
                link_file(entry.path(), home.join("AppData/Roaming/topgrade.toml"))?;
            }
            // Skip files not associated with a config.
            ".git" | "README.md" => {}
            config => {
                return Err(Error::InvalidConfig(config.to_string()));
            }
        }
    }

    Ok(())
}

fn link_file(src: PathBuf, dest: PathBuf) -> Result<(), Error> {
    if dest.exists() {
        if dest.symlink_metadata()?.file_type().is_symlink() {
            Ok(())
        } else {
            Err(Error::ExistingPath(dest))
        }
    } else {
        windows::fs::symlink_file(src, dest).map_err(|_| Error::InvalidPrivilege)?;
        Ok(())
    }
}

#[derive(DisplayDoc)]
enum Error {
    /// `~/.config/cleanfig` does not exist
    MissingRoot,
    /// io error: `{0}`
    Io(io::Error),
    /// invalid config path `{0}`
    InvalidConfig(String),
    /// must run as administrator
    InvalidPrivilege,
    /// {0}
    EnvVar(VarError),
    /// path `{0}` already exists
    ExistingPath(PathBuf),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<VarError> for Error {
    fn from(value: VarError) -> Self {
        Self::EnvVar(value)
    }
}

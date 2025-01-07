use std::fs;
use std::path::Path;

use crate::errors::SftpManError;

pub fn ensure_directory_recursively_created(path_str: &str) -> Result<(), SftpManError> {
    let path = Path::new(&path_str);

    fs::create_dir_all(path).map_err(|err| SftpManError::IO(path.to_path_buf(), err))?;

    Ok(())
}

pub fn remove_empty_directory(path_str: &str) -> Result<(), SftpManError> {
    let path = Path::new(&path_str);

    fs::remove_dir(path).map_err(|err| SftpManError::IO(path.to_path_buf(), err))?;

    Ok(())
}

pub fn get_mounts_under_path_prefix(prefix: &str) -> Result<Vec<mnt::MountEntry>, SftpManError> {
    mnt::get_submounts::<&str>(prefix).map_err(SftpManError::from)
}

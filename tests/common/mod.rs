use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fs::create_dir_all;
use std::io;
use std::{env::temp_dir, path::PathBuf};

pub fn setup() -> Result<PathBuf, io::Error> {
    let mut dir = temp_dir();

    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();

    dir.push(rand_string);
    let path = dir.as_path().to_owned();

    create_dir_all(path)?;

    Ok(dir)
}

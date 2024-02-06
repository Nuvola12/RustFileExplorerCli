use std::fs;
use std::fs::read_dir;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::io::Error;

/// Opens a file at the given path. Returns a string if there was an error.
// NOTE(conaticus): I tried handling the errors nicely here but Tauri was mega cringe and wouldn't let me nest results in async functions, so used string error messages instead.
pub fn open_file(path: String) -> Result<(), Error> {
    open::that(path)
}


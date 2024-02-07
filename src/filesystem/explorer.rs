use std::{fs, path};
use std::fs::read_dir;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::io::{Error, Write};
use std::fs::File;

/// Opens a file at the given path. Returns a string if there was an error.
// NOTE(conaticus): I tried handling the errors nicely here but Tauri was mega cringe and wouldn't let me nest results in async functions, so used string error messages instead.
pub fn open_file(path: String) -> Result<(), Error> {
    open::that(path)
}

pub fn delete_file(path: &String) -> Result<(), Error>{
    let _ = match fs::remove_file(path){
        Ok(data) => data,
        Err(e) => panic!("Could not delete file, Error {}", e),
    };

    Ok(())
}

pub fn make_file(path: &String, file_name: &String) -> Result<(), Error>{
    let full_path = format!("{}/{}", path, file_name);

    let mut file = match File::create(&full_path){
        Ok(data) => data,
        Err(e) => panic!("Could not create file, Error {}", e),
    };

    file.write_all(b"Initial content\n")?;

    Ok(())
}

pub fn rename_file(original_file: &String, path: &String, name: &String) -> Result<(), Error>{
    let full_path = format!("{}/{}", path, name);

    let res = match fs::rename(original_file, full_path){
        Ok(data) => data,
        Err(e) => panic!("Could not rename file, Error: {}", e),
    };

    Ok(())
}

pub fn copy_file(path: &String) -> Result<(), Error>{

    let _ = match fs::copy(path, "./.cache"){
        Ok(data) => data,
        Err(e) => panic!("Could not copy file, Error {}", e),
    };

    Ok(())
}

pub fn cut_file(path: &String) -> Result<(), Error>{

    let _ = copy_file(path);
    let _ = delete_file(path);

    Ok(())
}

pub fn paste_file(path: &String) -> Result<(), Error>{



    Ok(())
}

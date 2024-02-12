use std::{fs, path};
use std::fs::read_dir;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::io::{Error, Write};
use std::fs::File;
use std::env;

use crate::filesystem::util;

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

pub fn copy_file_to_cache(file_path: &str) -> std::io::Result<()> {

    let cache_dir = util::get_cahce_fodler().unwrap();
    
    // Create the cache directory if it doesn't exist
    let _ =  fs::create_dir_all(&cache_dir)?;
    
    // Get the file name from the original file path
    let file_name = util::strip_directory(file_path);

    // Construct the destination file path
    let dest_path = format!("{}\\{}", cache_dir, file_name);

    // Copy the file to the cache directory
    fs::copy(file_path, dest_path)?;

    Ok(())
}

pub fn cut_file(path: &String) -> Result<(), Error>{

    let _ = copy_file_to_cache(path);
    let _ = delete_file(path);

    Ok(())
}

pub fn paste_file(path: &String) -> Result<(), Error>{

    let cache_dir = util::get_cahce_fodler().unwrap();

    let files = util::get_files_in_directory(&cache_dir)?;

    
    let cached_file_path =files.first().unwrap();
    let cached_file_name = util::strip_directory(&cached_file_path);
    
    let dest_path = format!("{}\\{}", path, cached_file_name);
    
    fs::copy(cached_file_path, dest_path)?;

    //Remove pasted file from the cache
    let _ = delete_file(cached_file_path);
    

    Ok(())
}

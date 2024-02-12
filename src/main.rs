use crossterm::{
    event::{self,  Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use serde::{Deserialize, Serialize};
use std::{env::current_dir, fs, path};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
    backend::{Backend, CrosstermBackend}, layout::{Alignment, Constraint, Direction, Layout, Margin}, style::{Color, Modifier, Style}, symbols::line::VERTICAL, text::{Span, Spans}, widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    }, Terminal,Frame
};


mod filesystem;
mod draw;
mod AppInfo;


#[derive(Error, Debug)]
pub enum Error{
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let mut app = AppInfo::App::default();

    app.loaded_files = filesystem::util::fill_hashmap(".").unwrap(); //IMPORANT! CHANGE BACK TO DRIVE

    app.current_directory = "C:/Users/XxAnd/Documents".to_string();
    app.selected_file = "".to_string();

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        let mut last_key_time = Instant::now();

        loop{
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works"){
                if let CEvent::Key(key) = event::read().expect("can read events"){
                    let now = Instant::now();
                    let elapsed = now - last_key_time;

                    if elapsed >= tick_rate {
                        last_key_time = now;
                        tx.send(Event::Input(key)).expect("can send events");
                    }}
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick){
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    app.active_menu_item = AppInfo::MenuItem::Home;
    app.directory_list_state.select(Some(0));
    app.search_list_state.select(Some(0));

    loop{
        //Main Rendering
        let _ = terminal.draw(|f| draw::draw_ui(f, &mut app));
    
        //Input Handeling
        match rx.recv()? {
            Event::Input(event) => {
                match app.input_mode {
                    AppInfo::InputMode::Normal => {
                        // Your existing match statements for normal mode
                        match event.code {
                            KeyCode::Char('q') => {
                                disable_raw_mode()?;
                                terminal.show_cursor()?;
                                break;
                            }
                            
                            KeyCode::Up => {
                                if let Some(selected) = app.directory_list_state.selected() {
                                    let amount_pets = filesystem::util::get_files_in_directory(&app.current_directory).unwrap().len(); // Move to Util
                    
                                    if selected > 0{
                                        app.directory_list_state.select(Some(selected -1));
                                    }else{
                                        app.directory_list_state.select(Some(amount_pets -1));
                                    }
                                }
                            }

                            KeyCode::Down => {
                                if let Some(selected) = app.directory_list_state.selected() {
                                    let amount_pets = filesystem::util::get_files_in_directory(&app.current_directory).unwrap().len(); // Move to Util
                    
                                    if selected >= amount_pets -1{
                                        app.directory_list_state.select(Some(0));
                                    }else{
                                        app.directory_list_state.select(Some(selected+1));
                                    }
                                }
                            }
                            

                            KeyCode::Char('o') =>{
                                let _ = filesystem::explorer::open_file(app.selected_file.clone());
                            }

                            KeyCode::Char('/') => {
                                app.active_menu_item = AppInfo::MenuItem::Text;
                                app.input_mode = AppInfo::InputMode::Typing;

                                app.input_type = AppInfo::InputType::Searching;
                            }

                            KeyCode::Char('n') => {
                                app.active_menu_item = AppInfo::MenuItem::Text;
                                app.input_mode = AppInfo::InputMode::Typing;
                                app.input_type = AppInfo::InputType::MakeFile;


                            }

                            KeyCode::Char('r') => {
                                app.active_menu_item = AppInfo::MenuItem::Text;
                                app.input_mode = AppInfo::InputMode::Typing;
                                app.input_type = AppInfo::InputType::RenameFile;


                            }

                            KeyCode::Char('d') => {
                                let _ = filesystem::explorer::delete_file(&app.selected_file);

                            }

                            KeyCode::Char('c') => {
                                let _ = filesystem::explorer::copy_file_to_cache(&app.selected_file);
                            }

                            KeyCode::Char('v') => {
                                let _ = match filesystem::explorer::paste_file(&app.current_directory){
                                    Ok(data) => data,
                                    Err(e) => panic!("Error could not paste file, Error: {}", e),
                                };
                            }
                    
                            KeyCode::Backspace => {
                                if app.current_directory != "C:/".to_string(){
                                    let mut temp = app.current_directory.clone();
                                    app.current_directory.clear();
                    
                                    temp  = match filesystem::util::move_up_in_path(&temp){ 
                                        Ok(data) => data.unwrap(),
                                        Err(e) => panic!("error when moving up a directory"),
                                    };
                                    
                                    app.current_directory.push_str(&temp);
                                }
                                
                            }
                    
                            KeyCode::Enter =>{
                                //panic!("Switching directory to /{}/", selected_file);
                                filesystem::util::update_current_directory(&app.selected_file, &mut app.current_directory);
                            }
                    
                            _ => {}
                        }
                    }
                    AppInfo::InputMode::Typing => {
                        match event.code {
                            KeyCode::Enter => {
                                app.message = (app.input.drain(..).collect());

                                app.input_mode = AppInfo::InputMode::Normal;

                                match app.input_type{
                                    AppInfo::InputType::None => {
                                        app.active_menu_item = AppInfo::MenuItem::Home;
                                    },
                                    AppInfo::InputType::Searching => {
                                        app.active_menu_item = AppInfo::MenuItem::Search;
                                        app.input_type = AppInfo::InputType::None;
                                    },
                                    AppInfo::InputType::RenameFile => {
                                        let _ = filesystem::explorer::rename_file(&app.selected_file,&app.current_directory, &app.message);
                                    },
                                    AppInfo::InputType::MakeFile => {
                                        let _ = filesystem::explorer::make_file(&app.current_directory,&app.message);
                                    },
                                }

                                app.input_type = AppInfo::InputType::None;
                                app.active_menu_item = AppInfo::MenuItem::Home;

                            }
                            KeyCode::Char(c) => {
                                app.input.push(c);
                            }
                            KeyCode::Backspace => {
                                app.input.pop();
                            }
                            KeyCode::Esc => {
                                app.active_menu_item = AppInfo::MenuItem::Home;
                                app.input_mode = AppInfo::InputMode::Normal;
                            }
                            _ => {}
                        }
                    }
                   
                }
            }
            Event::Tick => {}
            // ... other event cases
        }
    }

    Ok(())
}

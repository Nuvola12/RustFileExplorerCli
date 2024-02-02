use chrono::prelude::*;
use crossterm::{
    event::{self,  Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::fs::metadata;

use serde::{Deserialize, Serialize};
use std::{env::current_dir, fs, path};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use walkdir::WalkDir;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
    backend::CrosstermBackend, layout::{Alignment, Constraint, Direction, Layout, Margin}, style::{Color, Modifier, Style}, symbols::line::VERTICAL, text::{Span, Spans}, widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    }, Terminal
};

enum FileType {
    file,
    directory,
}


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

    let mut current_directory: String = "C:/Users/XxAnd/Documents".to_string();
    let mut selected_file: String = "".to_string();



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

                    // Only process the key if enough time has passed
                    if elapsed >= tick_rate {
                        // Update the last key time
                        last_key_time = now;

                        // Send the key event
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

    let mut directory_list_state = ListState::default();
    directory_list_state.select(Some(0));

    loop{
        //Main Rendering
        let _ = terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(
                    [
                        Constraint::Length(2),
                        Constraint::Min(2),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                ).split(size);

            //Top Bar
            let top_bar = render_directory_display(&selected_file)
                .style(Style::default().fg(Color::LightGreen))
                .alignment(Alignment::Left)
                .block(
                    Block::default()
                        .borders(Borders::NONE)
                        .style(Style::default().fg(Color::White))
                        .border_type(BorderType::Plain),
                );
            rect.render_widget(top_bar, chunks[0]);

            //Main Content
            
            let file_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(60),
                    Constraint::Percentage(40),
                ].as_ref(),
            ).split(chunks[1]);

            let (left, right) = render_file_widget(&directory_list_state, &current_directory, &mut selected_file);

            rect.render_stateful_widget(left, file_chunks[0], &mut directory_list_state);
            rect.render_widget(right, file_chunks[1]);


            //Bottom Bar
            let bottom_bar = render_bottom_bar()
            .style(Style::default().fg(Color::LightGreen))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().fg(Color::White))
            );
            rect.render_widget(bottom_bar, chunks[2])

        });
    
        //Input Handeling
        match rx.recv()?{
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }

                KeyCode::Down => {
                    if let Some(selected) = directory_list_state.selected() {
                        let amount_pets = get_files_in_directory(&current_directory).unwrap().len();

                        if selected >= amount_pets -1{
                            directory_list_state.select(Some(0));
                        }else{
                            directory_list_state.select(Some(selected+1));
                        }
                    }
                }

                KeyCode::Up => {
                    if let Some(selected) = directory_list_state.selected() {
                        let amount_pets = get_files_in_directory(&current_directory).unwrap().len();

                        if selected > 0{
                            directory_list_state.select(Some(selected -1));
                        }else{
                            directory_list_state.select(Some(amount_pets -1));
                        }
                    }
                }

                KeyCode::Backspace => {
                    let mut temp = current_directory.clone();
                    current_directory.clear();
                    temp  = move_up_in_path(&temp).unwrap();
                    current_directory.push_str(&temp);
                    
                }

                KeyCode::Enter =>{
                    //panic!("Switching directory to /{}/", selected_file);
                    update_current_directory(&selected_file, &mut current_directory)
                }

                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

fn move_up_in_path(path_str: &String) -> Option<String>{
    let path =  Path::new(path_str);

    path.parent()
        .map(|parent|
             parent.to_string_lossy()
             .into_owned())
}

fn update_current_directory(selected_directory: &String, current_directory: &mut String) {
    current_directory.clear();
    current_directory.push_str(&selected_directory);
}
fn get_files_in_directory(path: &String) -> Result<Vec<String>, std::io::Error> {
    let paths: fs::ReadDir = fs::read_dir(path)?;

    let mut curr_dir = Vec::new();

    for path in paths {
        let curr = path.unwrap().path();
        let my_str = curr
            .into_os_string()
            .into_string()
            .unwrap()
            .replace("\\", "/");

        curr_dir.push(my_str);
        
    }

    Ok(curr_dir)
}

fn is_directory_empty(path: &String) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        return entries.count() == 0;
    }
    // If an error occurs while reading the directory, you can handle it accordingly.
    false
}

fn path_exists(path: &String) -> bool{
    fs::metadata(path).is_ok()
}

fn is_path_directory(path: &String) -> bool {

    let md = metadata(path).unwrap();

    md.is_dir()
    
}

fn strip_directory(path: &String) -> String{
    path.split('/').last().unwrap().to_string()
}



fn render_directory<'a>(
    list_state: &ListState,
    directory: &String,
) -> Result<(List<'a>, String), Box<dyn std::error::Error>> {
    let md = fs::metadata(directory)?;

    if is_directory_empty(directory) || !md.is_dir() {
        let list = List::new(Vec::new());
        let selected_dir = "";
        return Ok((list, selected_dir.to_string()));
    }

    let pets = Block::default()
        .borders(Borders::RIGHT)
        .style(Style::default().fg(Color::White))
        .border_type(BorderType::Plain);

    let curr_dir = get_files_in_directory(directory)?;

    let items: Vec<_> = curr_dir
        .iter()
        .map(|file| {
            let tmp = strip_directory(file);
            ListItem::new(Spans::from(vec![Span::styled(
                tmp.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_pet = curr_dir
        .get(
            list_state
                .selected()
                .expect("there is always a selected pet"),
        )
        .map(|pet| pet.clone())
        .unwrap_or_else(|| String::new());

    let list = List::new(items).block(pets).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    Ok((list, selected_pet))
}


fn render_file_widget<'a>(directory_list_state: &ListState,current_directory: &String, selected_file: &mut String) -> (List<'a>, Paragraph<'a>){

    let (directory_widget, selected_dir) = match render_directory(directory_list_state, current_directory){
        Ok(data) => data,
        Err(e) => panic!("test"),
    };
    
    if path_exists(&selected_dir){
        selected_file.clear();
        selected_file.push_str(&selected_dir);
    }
    


    let info_bar = render_details();

    (directory_widget, info_bar)
}

fn render_directory_display<'a>( directory: &String) -> Paragraph<'a> {
    Paragraph::new(directory.to_string())
}

fn render_bottom_bar<'a>() -> Paragraph<'a> {
    Paragraph::new("pet-CLI 2024 - all rights reserved")
}

fn render_details<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "pet-CLI",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default());
        home
}

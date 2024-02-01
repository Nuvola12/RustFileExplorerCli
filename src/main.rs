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

#[derive(Serialize, Deserialize, Clone)]
enum FileType {
    file,
    directory,
}

struct File{
    name: String,
    file_type: FileType,
    location: String,
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


#[derive(Copy, Clone, Debug)]
enum MenuItem{
    Home,
    Pets,
}

impl From<MenuItem> for usize{
    fn from(input: MenuItem) -> usize{
        match input {
            MenuItem::Home => 0,
            MenuItem::Pets => 1,
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let mut current_directory = "C:/Users/XxAnd";
    current_directory = "C:/Users/XxAnd/Documents";



    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(20000);

    let mut directory: Vec<String> = Vec::new();

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
                    if elapsed >= Duration::from_millis(100) {
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

    let mut subdirectory_list_state = ListState::default();
    subdirectory_list_state.select(Some(0));


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
            let top_bar = render_directory_display(current_directory)
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
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                    Constraint::Percentage(33)
                ].as_ref(),
            ).split(chunks[1]);

            let (left, mid, right) = render_file_widget(&directory_list_state, &subdirectory_list_state, current_directory);

            rect.render_stateful_widget(left, file_chunks[0], &mut directory_list_state);
            rect.render_stateful_widget(mid, file_chunks[1], &mut subdirectory_list_state);
            rect.render_widget(right, file_chunks[2]);


            //Bottom Bar
            let bottom_bar = Paragraph::new("pet-CLI 2024 - all rights reserved")
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
                        let amount_pets = get_files_in_directory(current_directory).unwrap().len();

                        if selected >= amount_pets -1{
                            directory_list_state.select(Some(0));
                        }else{
                            directory_list_state.select(Some(selected+1));
                        }
                    }
                }

                KeyCode::Up => {
                    if let Some(selected) = directory_list_state.selected() {
                        let amount_pets = get_files_in_directory(current_directory).unwrap().len();

                        if selected > 0{
                            directory_list_state.select(Some(selected -1));
                        }else{
                            directory_list_state.select(Some(amount_pets -1));
                        }
                    }
                }

                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}


fn get_files_in_directory(path: &str) -> Result<Vec<String>, std::io::Error> {
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

fn is_directory_empty(path: &str) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        return entries.count() == 0;
    }
    // If an error occurs while reading the directory, you can handle it accordingly.
    false
}

fn strip_directory(path: &str) -> String{
    path.split('/').last().unwrap().to_string()
}

fn render_directory<'a>(
    list_state: &ListState,
    directory: &str,
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
        .expect("exists")
        .clone();

    let list = List::new(items).block(pets).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    Ok((list, selected_pet))
}

//Crashes if empty for some reason {deprecated as I needed to add proper error handling}
fn render_directory_deprecated<'a>(list_state: &ListState, directory: &str) -> (List<'a>, String){
    
    let md = metadata(directory).unwrap();

    if(is_directory_empty(directory) || !md.is_dir()){
        let list = List::new(Vec::new());
        let selected_dir = "";

        return (list, selected_dir.to_string())
    }

    let pets = Block::default()
        .borders(Borders::RIGHT)
        .style(Style::default().fg(Color::White))
        .border_type(BorderType::Plain);


    let curr_dir =  get_files_in_directory(directory).unwrap();

    let items: Vec<_> = curr_dir
        .iter()
        .map(|file| {
            let tmp = strip_directory(file);
            ListItem::new(Spans::from(vec![Span::styled(tmp.clone(), Style::default())]))
        }).collect();

    let selected_pet = curr_dir
    .get(
        list_state
            .selected()
            .expect("there is allwats a slected pet"),
    ).expect("exists").clone();

    let list = List::new(items).block(pets).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    );
    (list, selected_pet)
}

fn render_file_widget<'a>(directory_list_state: &ListState, subdirectory_list_state: &ListState,current_directory: &str) -> (List<'a>, List<'a>, Paragraph<'a>){

    let (directory_widget, selected_dir) = render_directory(directory_list_state, current_directory).unwrap();
    
    let (subdirectory_widget,_) = match render_directory(subdirectory_list_state, &selected_dir){
        Ok(file) => file,
        Err(error) => {
            let list = List::new(Vec::new());
            let selected_dir = "";

            (list, selected_dir.to_string())
        },
    };

    let info_bar = render_details();

    (directory_widget, subdirectory_widget, info_bar)
}

fn render_directory_display<'a>( directory: &str) -> Paragraph<'a> {
    Paragraph::new(directory.to_string())
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

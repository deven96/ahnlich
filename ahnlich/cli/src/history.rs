use std::{
    fs::OpenOptions,
    io::{self, BufRead, Write},
    path::PathBuf,
};

fn get_history_file_path() -> PathBuf {
    let mut path = dirs::home_dir().expect("Could not find home directory");
    path.push(".ahnlich_cli_history");
    path
}

fn load_command_history() -> Vec<String> {
    let path = get_history_file_path();
    if path.exists() {
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .expect("Unable to open history file");
        let reader = io::BufReader::new(file);
        reader.lines().map_while(Result::ok).collect()
    } else {
        Vec::new()
    }
}

fn save_command_history(commands: &[String]) {
    let path = get_history_file_path();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .expect("Unable to open history file");
    for command in commands {
        writeln!(file, "{}", command).expect("Unable to write to history file");
    }
}

pub(crate) struct HistoryManager {
    command_history: Vec<String>,
    current_command_index: usize,
}

impl HistoryManager {
    pub(crate) fn new() -> Self {
        let command_history = load_command_history();
        let current_command_index = command_history.len();
        Self {
            command_history,
            current_command_index,
        }
    }

    pub(crate) fn down(&mut self) -> String {
        if self.current_command_index < self.command_history.len() {
            self.current_command_index += 1;
        }
        if self.is_index_end() {
            String::new()
        } else {
            self.get_at_index()
        }
    }

    pub(crate) fn up(&mut self) -> String {
        if self.current_command_index > 0 {
            self.current_command_index -= 1;
        }
        if self.command_history.is_empty() && self.current_command_index == 0 {
            String::new()
        } else {
            self.get_at_index()
        }
    }

    fn get_at_index(&self) -> String {
        self.command_history[self.current_command_index].clone()
    }

    pub(crate) fn reset_index(&mut self) {
        self.current_command_index = self.command_history.len();
    }

    pub(crate) fn is_index_end(&self) -> bool {
        self.current_command_index == self.command_history.len()
    }

    pub(crate) fn save_to_disk(&self) {
        save_command_history(&self.command_history);
    }

    pub(crate) fn add_command(&mut self, command: &str) {
        if let Some(last_command) = self.command_history.last() {
            if last_command != command {
                self.command_history.push(command.to_string());
            }
        } else {
            self.command_history.push(command.to_string());
        }
        self.reset_index();
    }
}

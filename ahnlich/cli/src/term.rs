use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyCode, KeyEvent},
    queue,
    style::{Color, Print, SetForegroundColor, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Stdout, Write, stdout};

use crate::{connect::AgentClient, history::HistoryManager};

#[derive(Debug)]
enum SpecialEntry {
    Enter,
    Up,
    Down,
    Left,
    Right,
    Del,
    Exit,
    ClrScr,
}

#[derive(Debug)]
enum Entry {
    Char(char),
    Special(SpecialEntry),
    None,
}

impl Entry {
    fn is_history_key(&self) -> bool {
        matches!(
            self,
            Entry::Special(SpecialEntry::Up) | Entry::Special(SpecialEntry::Down)
        )
    }
}

#[derive(Debug)]
enum LineResult {
    Command(String),
    Exit,
}

pub struct Term {
    client: AgentClient,
}

impl Term {
    pub fn new(client: AgentClient) -> Self {
        Self { client }
    }

    fn read_char(&self) -> io::Result<Entry> {
        match event::read()? {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => {
                if code == KeyCode::Char('c') && modifiers == event::KeyModifiers::CONTROL {
                    return Ok(Entry::Special(SpecialEntry::Exit));
                }
                if code == KeyCode::Char('l') && modifiers == event::KeyModifiers::CONTROL {
                    return Ok(Entry::Special(SpecialEntry::ClrScr));
                }
                Ok(match code {
                    KeyCode::Enter => Entry::Special(SpecialEntry::Enter),
                    KeyCode::Char(c) => Entry::Char(c),
                    KeyCode::Left => Entry::Special(SpecialEntry::Left),
                    KeyCode::Up => Entry::Special(SpecialEntry::Up),
                    KeyCode::Down => Entry::Special(SpecialEntry::Down),
                    KeyCode::Right => Entry::Special(SpecialEntry::Right),
                    KeyCode::Backspace => Entry::Special(SpecialEntry::Del),
                    _ => Entry::None,
                })
            }
            _ => Ok(Entry::None),
        }
    }
    pub fn welcome_message(&self) -> io::Result<()> {
        let mut stdout = stdout();
        queue!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::White),
            Print(format!("Welcome To Ahnlich {}\n\n", self.client)),
            SetForegroundColor(Color::White),
        )?;
        stdout.flush()?;
        Ok(())
    }

    pub(crate) fn ahnlich_prompt(&self, stdout: &mut Stdout) -> io::Result<()> {
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.execute(Print(">>>"))?;
        stdout.execute(SetForegroundColor(Color::White))?;

        stdout.flush()?;
        Ok(())
    }

    pub(crate) fn format_output(&self, query: &str) -> String {
        let matches = |c| c == ';' || c == ' ';
        query
            .split_inclusive(matches)
            .map(|ex| {
                // Trim the trailing space or semicolon from the command part
                let trimmed_ex = ex.trim_end_matches(matches);

                if self
                    .client
                    .commands()
                    .contains(&(trimmed_ex.to_lowercase().as_str()))
                {
                    // Add back the space or semicolon at the end (if present)
                    format!("{}{}", trimmed_ex.magenta(), &ex[trimmed_ex.len()..])
                } else {
                    format!("{}{}", trimmed_ex.white(), &ex[trimmed_ex.len()..])
                }
            })
            .collect::<String>()
    }

    fn remove_at_pos(&self, input: &mut String, char_index: u16) {
        let byte_index = input
            .char_indices()
            .nth(char_index as usize)
            .map(|(entry, _)| entry)
            .unwrap_or_else(|| panic!("Index out of bounds {} --> {}", input.len(), char_index));

        input.remove(byte_index);
    }

    fn move_to_pos_and_print(
        &self,
        stdout: &mut Stdout,
        output: &str,
        col_pos: u16,
    ) -> io::Result<()> {
        let formatted_output = self.format_output(output);
        queue!(
            stdout,
            cursor::MoveToColumn(col_pos),
            terminal::Clear(terminal::ClearType::FromCursorDown),
            Print(formatted_output)
        )?;
        stdout.flush()?;
        Ok(())
    }

    fn delete_and_print_less(
        &self,
        stdout: &mut Stdout,
        output: &str,
        col_pos: u16,
    ) -> io::Result<()> {
        let formatted_output = self.format_output(output);
        let clean = vec![" "; output.len() + 1];
        queue!(
            stdout,
            cursor::MoveToColumn(col_pos),
            Print(clean.join("").to_string()),
            cursor::MoveToColumn(col_pos),
            Print(formatted_output)
        )?;
        stdout.flush()?;
        Ok(())
    }

    fn read_line(
        &self,
        stdout: &mut Stdout,
        history: &mut HistoryManager,
    ) -> io::Result<LineResult> {
        let (start_pos_col, _) = cursor::position()?;
        let mut output = String::new();

        loop {
            let char = self.read_char()?;
            let (current_pos_col, _) = cursor::position()?;
            if !char.is_history_key() {
                history.reset_index();
            }
            match char {
                Entry::Char(c) => {
                    let insertion_position = current_pos_col - start_pos_col;
                    output.insert(insertion_position as usize, c);
                    self.move_to_pos_and_print(stdout, &output, start_pos_col)?;
                    stdout.execute(cursor::MoveToColumn(current_pos_col + 1))?;
                }
                Entry::Special(special) => match special {
                    SpecialEntry::Up => {
                        output = history.up();
                        queue!(
                            stdout,
                            cursor::MoveToColumn(start_pos_col),
                            terminal::Clear(terminal::ClearType::FromCursorDown),
                        )?;
                        self.move_to_pos_and_print(stdout, &output, start_pos_col)?;
                    }
                    SpecialEntry::Down => {
                        output = history.down();
                        queue!(
                            stdout,
                            cursor::MoveToColumn(start_pos_col),
                            terminal::Clear(terminal::ClearType::FromCursorDown),
                        )?;
                        self.move_to_pos_and_print(stdout, &output, start_pos_col)?;
                    }
                    SpecialEntry::Enter => {
                        queue!(stdout, Print("\n"), cursor::MoveToColumn(0))?;
                        stdout.flush()?;
                        break;
                    }
                    SpecialEntry::Left => {
                        if start_pos_col < current_pos_col {
                            stdout.execute(cursor::MoveLeft(1))?;
                        }
                    }
                    SpecialEntry::Right => {
                        if start_pos_col + output.len() as u16 > current_pos_col {
                            stdout.execute(cursor::MoveRight(1))?;
                        }
                    }
                    SpecialEntry::Del => {
                        if current_pos_col == start_pos_col {
                            continue;
                        }
                        let output_current_pos = current_pos_col - start_pos_col;

                        if !output.is_empty() {
                            self.remove_at_pos(&mut output, output_current_pos - 1);
                            self.delete_and_print_less(stdout, &output, start_pos_col)?;
                            stdout.execute(cursor::MoveToColumn(current_pos_col - 1))?;
                        }
                    }
                    SpecialEntry::ClrScr => {
                        queue!(
                            stdout,
                            cursor::MoveTo(0, 0),
                            terminal::Clear(terminal::ClearType::All),
                        )?;
                        self.ahnlich_prompt(stdout)?;
                        self.move_to_pos_and_print(stdout, &output, start_pos_col)?;
                    }
                    SpecialEntry::Exit => return Ok(LineResult::Exit),
                },
                Entry::None => {
                    continue;
                }
            }
        }
        history.add_command(&output);
        history.save_to_disk();
        Ok(LineResult::Command(output))
    }

    pub async fn run(&self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        stdout.execute(cursor::EnableBlinking)?;
        stdout.execute(cursor::SetCursorStyle::BlinkingBar)?;

        let mut history = HistoryManager::new();

        loop {
            self.ahnlich_prompt(&mut stdout)?;
            let input = self.read_line(&mut stdout, &mut history)?;
            match input {
                LineResult::Exit => {
                    break;
                }
                LineResult::Command(input) => match input.as_str() {
                    "quit" | "exit" | "exit()" => break,
                    command => {
                        let response = self.client.parse_queries(command).await;

                        match response {
                            Ok(success) => {
                                disable_raw_mode()?;
                                for msg in success {
                                    queue!(
                                        stdout,
                                        Print(format!("{msg}\n")),
                                        cursor::MoveToColumn(0)
                                    )?;
                                }
                                stdout.flush()?;
                                enable_raw_mode()?
                            }
                            Err(err) => {
                                queue!(
                                    stdout,
                                    Print(format!("{}\n", err.red())),
                                    cursor::MoveToColumn(0)
                                )?;
                                stdout.flush()?;
                            }
                        }
                    }
                },
            };
        }
        disable_raw_mode()?;
        Ok(())
    }
}

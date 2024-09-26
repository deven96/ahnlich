use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    style::{Color, Print, SetForegroundColor, Stylize},
    ExecutableCommand,
};
use std::io::{self, stdout, Write};

use crate::connect::AgentPool;

const RESERVED_WORDS: [&str; 3] = ["hello", "print", "ping"];

pub struct Term {
    client_pool: AgentPool,
}

impl Term {
    pub fn new(client_pool: AgentPool) -> Self {
        Self { client_pool }
    }

    pub(crate) fn read_line(&self) -> io::Result<String> {
        let mut line = String::new();
        while let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Enter => {
                    break;
                }
                KeyCode::Char(c) => {
                    line.push(c);
                }
                KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }
        Ok(line)
    }
    pub fn welcome_message(&self) -> io::Result<()> {
        let mut stdout = stdout();
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.execute(Print(format!("Welcome To Ahnlich {}", self.client_pool)))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.flush()?;
        stdout.flush()?;
        Ok(())
    }

    pub(crate) fn ahnlich_prompt(&self) -> io::Result<()> {
        let mut stdout = stdout();
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.execute(Print(">>> "))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.flush()?;
        stdout.flush()?;
        Ok(())
    }

    pub(crate) fn query_output(&self, query: String) -> io::Result<()> {
        self.ahnlich_prompt()?;
        let output = String::from_iter(query.split(' ').map(|ex| {
            if RESERVED_WORDS.contains(&ex) {
                format!("{} ", ex.magenta())
            } else {
                format!("{} ", ex.white())
            }
        }));

        println!("{output}");

        Ok(())
    }

    pub async fn run(&self) -> io::Result<()> {
        loop {
            self.ahnlich_prompt()?;
            let input = self.read_line()?;
            match input.as_str() {
                "quit" | "exit()" => break,
                _ => self.query_output(input)?,
            };
        }
        Ok(())
    }
}

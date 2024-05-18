use std::{borrow::Cow, marker::PhantomData, process::exit};

use clap::Parser;
use console::style;
use rustyline::{
    completion::Completer, highlight::Highlighter, hint::Hinter, validate::Validator, Cmd, Editor,
    Event, Helper, KeyCode, KeyEvent, Modifiers,
};

pub struct ClapEditorHelper<C: Parser> {
    c_phantom: PhantomData<C>,
}

impl<C: Parser> Completer for ClapEditorHelper<C> {
    type Candidate = &'static str;
}

impl<C: Parser> Highlighter for ClapEditorHelper<C> {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(style(hint).dim().to_string())
    }
}

impl<C: Parser> Validator for ClapEditorHelper<C> {}

impl<C: Parser> Hinter for ClapEditorHelper<C> {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        let command = C::command();
        let args = shlex::split(line)?;

        if let [arg] = args.as_slice() {
            for c in command.get_subcommands() {
                if let Some(x) = c.get_name().strip_prefix(arg) {
                    return Some(x.to_string());
                }
            }
        }
        None
    }
}

impl<C: Parser> Helper for ClapEditorHelper<C> {}

pub struct ClapEditor<C: Parser> {
    rl: Editor<ClapEditorHelper<C>, rustyline::history::FileHistory>,
    prompt: String,
}

impl<C: Parser> Default for ClapEditor<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Parser> ClapEditor<C> {
    fn construct(prompt: String) -> Self {
        let mut rl = Editor::<ClapEditorHelper<C>, _>::new().unwrap();
        rl.set_helper(Some(ClapEditorHelper {
            c_phantom: PhantomData,
        }));
        rl.bind_sequence(
            Event::KeySeq(vec![KeyEvent(KeyCode::Tab, Modifiers::NONE)]),
            Cmd::CompleteHint,
        );
        ClapEditor { rl, prompt }
    }

    /// Creates a new `ClapEditor` with the default prompt.
    pub fn new() -> Self {
        Self::construct(style(">> ").cyan().bright().to_string())
    }

    /// Creates a new `ClapEditor` with the given prompt.
    pub fn new_with_prompt(prompt: &str) -> Self {
        Self::construct(prompt.into())
    }

    pub fn get_editor(&mut self) -> &mut Editor<ClapEditorHelper<C>, rustyline::history::FileHistory> {
        &mut self.rl
    }

    pub fn read_command(&mut self) -> Option<C> {
        let line = match self.rl.readline(&self.prompt) {
            Ok(x) => x,
            Err(e) => match e {
                rustyline::error::ReadlineError::Eof
                | rustyline::error::ReadlineError::Interrupted => exit(0),
                rustyline::error::ReadlineError::WindowResized => return None,
                _ => panic!("Error in read line: {e:?}"),
            },
        };
        if line.trim().is_empty() {
            return None;
        }

        _ = self.rl.add_history_entry(line.as_str());

        match shlex::split(&line) {
            Some(split) => {
                match C::try_parse_from(std::iter::once("").chain(split.iter().map(String::as_str)))
                {
                    Ok(c) => Some(c),
                    Err(e) => {
                        e.print().unwrap();
                        None
                    }
                }
            }
            None => {
                println!(
                    "{} input was not valid and could not be processed",
                    style("error:").red().bold()
                );
                None
            }
        }
    }
}

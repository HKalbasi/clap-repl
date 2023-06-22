use std::{borrow::Cow, marker::PhantomData, process::exit};

use clap::Parser;
use console::style;
use rustyline::{
    completion::Completer, highlight::Highlighter, hint::Hinter, validate::Validator, Cmd, Editor,
    Event, Helper, KeyCode, KeyEvent, Modifiers,
};

struct ClapEditorHelper<C: Parser> {
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
        let args = shlex::split(line).unwrap();
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
}

impl<C: Parser> Default for ClapEditor<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Parser> ClapEditor<C> {
    pub fn new() -> Self {
        let mut rl = Editor::<ClapEditorHelper<C>, _>::new().unwrap();
        rl.set_helper(Some(ClapEditorHelper {
            c_phantom: PhantomData,
        }));
        rl.bind_sequence(
            Event::KeySeq(vec![KeyEvent(KeyCode::Tab, Modifiers::NONE)]),
            Cmd::CompleteHint,
        );
        ClapEditor { rl }
    }

    pub fn read_command(&mut self) -> Option<C> {
        let line = match self.rl.readline(&style(">> ").cyan().bright().to_string()) {
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
        let splited = shlex::split(&line).unwrap();
        match C::try_parse_from(Some("".to_owned()).into_iter().chain(splited)) {
            Ok(c) => Some(c),
            Err(e) => {
                e.print().unwrap();
                None
            }
        }
    }
}

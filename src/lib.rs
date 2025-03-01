use std::{ffi::OsString, marker::PhantomData, path::PathBuf, str::FromStr};

use clap::{Parser, Subcommand};
use console::style;

// reexport reedline to prevent version mixups
pub use reedline;
use reedline::{Prompt, Reedline, Signal, Span};
use shlex::Shlex;

mod builder;

pub use builder::ClapEditorBuilder;

pub struct ClapEditor<C: Parser + Send + Sync + 'static> {
    rl: Reedline,
    prompt: Box<dyn Prompt>,
    c_phantom: PhantomData<C>,
}

struct ReedCompleter<C: Parser + Send + Sync + 'static> {
    c_phantom: PhantomData<C>,
}

impl<C: Parser + Send + Sync + 'static> reedline::Completer for ReedCompleter<C> {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<reedline::Suggestion> {
        let cmd = C::command();
        let mut cmd = clap_complete::dynamic::command::CompleteCommand::augment_subcommands(cmd);
        let args = Shlex::new(line);
        let mut args = std::iter::once("".to_owned())
            .chain(args)
            .map(OsString::from)
            .collect::<Vec<_>>();
        if line.ends_with(' ') {
            args.push(OsString::new());
        }
        let arg_index = args.len() - 1;
        let span = Span::new(pos - args[arg_index].len(), pos);
        let Ok(candidates) = clap_complete::dynamic::complete(
            &mut cmd,
            args,
            arg_index,
            PathBuf::from_str(".").ok().as_deref(),
        ) else {
            return vec![];
        };
        candidates
            .into_iter()
            .map(|c| reedline::Suggestion {
                value: c.get_content().to_string_lossy().into_owned(),
                description: c.get_help().map(|x| x.to_string()),
                style: None,
                extra: None,
                span,
                append_whitespace: true,
            })
            .collect()
    }
}

pub enum ReadCommandOutput<C> {
    /// Input parsed successfully.
    Command(C),

    /// Input was empty.
    EmptyLine,

    /// Clap parse error happened. You should print the error manually.
    ClapError(clap::error::Error),

    /// Input was not lexically valid, for example it had odd number of `"`
    ShlexError,

    /// Reedline failed to work with stdio.
    ReedlineError(std::io::Error),

    /// User pressed ctrl+C
    CtrlC,

    /// User pressed ctrl+D
    CtrlD,
}

impl<C: Parser + Send + Sync + 'static> ClapEditor<C> {
    pub fn builder() -> ClapEditorBuilder<C> {
        ClapEditorBuilder::<C>::new()
    }

    pub fn get_editor(&mut self) -> &mut Reedline {
        &mut self.rl
    }

    pub fn read_command(&mut self) -> ReadCommandOutput<C> {
        let line = match self.rl.read_line(&*self.prompt) {
            Ok(Signal::Success(buffer)) => buffer,
            Ok(Signal::CtrlC) => return ReadCommandOutput::CtrlC,
            Ok(Signal::CtrlD) => return ReadCommandOutput::CtrlD,
            Err(e) => return ReadCommandOutput::ReedlineError(e),
        };
        if line.trim().is_empty() {
            return ReadCommandOutput::EmptyLine;
        }

        // _ = self.rl.add_history_entry(line.as_str());

        match shlex::split(&line) {
            Some(split) => {
                match C::try_parse_from(std::iter::once("").chain(split.iter().map(String::as_str)))
                {
                    Ok(c) => ReadCommandOutput::Command(c),
                    Err(e) => ReadCommandOutput::ClapError(e),
                }
            }
            None => ReadCommandOutput::ShlexError,
        }
    }

    pub fn repl(mut self, mut handler: impl FnMut(C)) {
        loop {
            match self.read_command() {
                ReadCommandOutput::Command(c) => handler(c),
                ReadCommandOutput::EmptyLine => (),
                ReadCommandOutput::ClapError(e) => {
                    e.print().unwrap();
                }
                ReadCommandOutput::ShlexError => {
                    println!(
                        "{} input was not valid and could not be processed",
                        style("Error:").red().bold()
                    );
                }
                ReadCommandOutput::ReedlineError(e) => {
                    panic!("{e}");
                }
                ReadCommandOutput::CtrlC => continue,
                ReadCommandOutput::CtrlD => break,
            }
        }
    }

    #[cfg(feature = "async")]
    pub async fn repl_async(mut self, mut handler: impl AsyncFnMut(C)) {
        loop {
            match self.read_command() {
                ReadCommandOutput::Command(c) => handler(c).await,
                ReadCommandOutput::EmptyLine => (),
                ReadCommandOutput::ClapError(e) => {
                    e.print().unwrap();
                }
                ReadCommandOutput::ShlexError => {
                    println!(
                        "{} input was not valid and could not be processed",
                        style("Error:").red().bold()
                    );
                }
                ReadCommandOutput::ReedlineError(e) => {
                    panic!("{e}");
                }
                ReadCommandOutput::CtrlC => continue,
                ReadCommandOutput::CtrlD => break,
            }
        }
    }
}

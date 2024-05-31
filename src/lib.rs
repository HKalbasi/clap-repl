use std::{ffi::OsString, marker::PhantomData, path::PathBuf, str::FromStr};

use clap::{Parser, Subcommand};
use console::style;
use nu_ansi_term::{Color, Style};
use reedline::{
    default_emacs_keybindings, DefaultHinter, DefaultPrompt, Emacs, IdeMenu, KeyModifiers,
    MenuBuilder, Prompt, Reedline, ReedlineEvent, ReedlineMenu, Signal, Span,
};
use shlex::Shlex;

pub struct ClapEditor<C: Parser + Send + Sync + 'static> {
    rl: Reedline,
    prompt: Box<dyn Prompt>,
    c_phantom: PhantomData<C>,
}

impl<C: Parser + Send + Sync + 'static> Default for ClapEditor<C> {
    fn default() -> Self {
        Self::new()
    }
}

struct ReedCompleter<C: Parser + Send + Sync + 'static> {
    c_phantom: PhantomData<C>,
}

impl<C: Parser + Send + Sync + 'static> reedline::Completer for ReedCompleter<C> {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<reedline::Suggestion> {
        let cmd = C::command();
        let mut cmd = clap_complete::dynamic::shells::CompleteCommand::augment_subcommands(cmd);
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
                value: c.0.to_string_lossy().into_owned(),
                description: c.1.map(|x| x.to_string()),
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
    fn construct(prompt: Box<dyn Prompt>, hook: impl FnOnce(Reedline) -> Reedline) -> Self {
        let completion_menu = Box::new(
            IdeMenu::default()
                .with_default_border()
                .with_name("completion_menu"),
        );
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            reedline::KeyCode::Tab,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::Menu("completion_menu".to_string()),
                ReedlineEvent::MenuNext,
            ]),
        );

        let rl = Reedline::create()
            .with_completer(Box::new(ReedCompleter::<C> {
                c_phantom: PhantomData,
            }))
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_hinter(Box::new(
                DefaultHinter::default().with_style(Style::new().italic().fg(Color::DarkGray)),
            ))
            .with_edit_mode(Box::new(Emacs::new(keybindings)));

        let rl = hook(rl);
        ClapEditor {
            rl,
            prompt,
            c_phantom: PhantomData,
        }
    }

    /// Creates a new `ClapEditor` with the default prompt.
    pub fn new() -> Self {
        Self::construct(Box::<DefaultPrompt>::default(), |e| e)
    }

    /// Creates a new `ClapEditor` with the given prompt.
    pub fn new_with_prompt(
        prompt: Box<dyn Prompt>,
        hook: impl FnOnce(Reedline) -> Reedline,
    ) -> Self {
        Self::construct(prompt, hook)
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
                ReadCommandOutput::CtrlC | ReadCommandOutput::CtrlD => break,
            }
        }
    }
}

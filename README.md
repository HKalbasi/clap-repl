# clap-repl

[![Rust](https://github.com/HKalbasi/clap-repl/actions/workflows/rust.yml/badge.svg)](https://github.com/HKalbasi/clap-repl/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/clap-repl.svg)](https://crates.io/crates/clap-repl)

One of the typical user interfaces for prompting commands is the repl (read eval print loop). One of the best ways of representing commands in a repl
is using space separated arguments, which is what terminal shells do. And the way to parse such commands in Rust is the `clap` crate. This crate uses
`clap` and `reedline` to provide such user interface in a way that you only focus on your app logic.

## Features

Thanks to `clap` and `reedline` this crate handles:
* Parsing the space separated commands into your data structure.
* Help flag for each command.
* Verifying the command is valid, generating useful errors and suggestions otherwise.
* Auto complete and hint for the commands.

## Example

```Rust
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use clap_repl::ClapEditor;
use reedline::{DefaultPrompt, DefaultPromptSegment, FileBackedHistory, Reedline, Signal};

#[derive(Debug, Parser)]
#[command(name = "")] // This name will show up in clap's error messages, so it is important to set it to "".
enum SampleCommand {
    Download {
        path: PathBuf,
        /// Check the integrity of the downloaded object
        ///
        /// Uses SHA256
        #[arg(long)]
        check_sha: bool,
    },
    /// A command to upload things.
    Upload,
    /// Login into the system.
    Login {
        /// Optional. You will be prompted if you don't provide it.
        #[arg(short, long)]
        username: Option<String>,
        #[arg(short, long, value_enum, default_value_t = Mode::Secure)]
        mode: Mode,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Encrypt the password
    Secure,
    /// Send the password plain
    ///
    /// This paragraph is ignored because there is no long help text for possible values in clap.
    Insecure,
}

fn main() {
    let mut prompt = DefaultPrompt::default();
    prompt.left_prompt = DefaultPromptSegment::Basic("simple-example".to_owned());
    let mut rl = ClapEditor::<SampleCommand>::new_with_prompt(Box::new(prompt), |reed| {
        // Do custom things with `Reedline` instance here
        reed.with_history(Box::new(
            FileBackedHistory::with_file(10000, "/tmp/clap-repl-simple-example-history".into())
                .unwrap(),
        ))
    });
    loop {
        // Use `read_command` instead of `readline`.
        let Some(command) = rl.read_command() else {
            continue;
        };
        match command {
            SampleCommand::Download { path, check_sha } => {
                println!("Downloaded {path:?} with checking = {check_sha}");
            }
            SampleCommand::Upload => {
                println!("Uploaded");
            }
            SampleCommand::Login { username, mode } => {
                // You can use another `reedline::Reedline` inside the loop.
                let mut rl = Reedline::create();
                let username = username
                    .unwrap_or_else(|| read_line_with_reedline(&mut rl, "What is your username? "));
                let password = read_line_with_reedline(&mut rl, "What is your password? ");
                println!("Logged in with {username} and {password} in mode {mode:?}");
            }
        }
    }
}

fn read_line_with_reedline(rl: &mut Reedline, prompt: &str) -> String {
    let Signal::Success(x) = rl
        .read_line(&DefaultPrompt::new(
            DefaultPromptSegment::Basic(prompt.to_owned()),
            DefaultPromptSegment::Empty,
        ))
        .unwrap()
    else {
        panic!();
    };
    x
}
```
![Screenshot from 2023-06-22 11-32-58](https://github.com/HKalbasi/clap-repl/assets/45197576/2c1b2ceb-e562-4536-8b42-4025d5a9674a)
![Screenshot from 2023-06-22 11-35-33](https://github.com/HKalbasi/clap-repl/assets/45197576/bec9110e-a399-41e4-8f63-1c8592338625)
![image](https://github.com/HKalbasi/clap-repl/assets/45197576/a5eb04c0-fafc-479e-ba1a-dd5757f585be)



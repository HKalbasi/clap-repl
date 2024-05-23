# clap-repl

[![Rust](https://github.com/HKalbasi/clap-repl/actions/workflows/rust.yml/badge.svg)](https://github.com/HKalbasi/clap-repl/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/clap-repl.svg)](https://crates.io/crates/clap-repl)

One of the typical user interfaces for prompting commands is the repl (read eval print loop). One of the best ways of representing commands in a repl
is using space separated arguments, which is what terminal shells do. And the way to parse such commands in Rust is the `clap` crate. This crate uses
`clap` and `rustyline` to provide such user interface in a way that you only focus on your app logic.

## Features

Thanks to `clap` and `rustyline` this crate handles:
* Parsing the space separated commands into your data structure.
* Help flag for each command.
* Verifying the command is valid, generating useful errors and suggestions otherwise.
* Auto complete and hint for the commands.

## Example

```Rust
use clap::Parser;
use clap_repl::ClapEditor;
use console::style;
use rustyline::DefaultEditor;

#[derive(Debug, Parser)]
#[command(name = "")] // This name will show up in clap's error messages, so it is important to set it to "".
enum SampleCommand {
    Download {
        path: String,
        /// Some explanation about what this flag do.
        #[arg(long)]
        check_sha: bool,
    },
    /// A command to upload things.
    Upload,
    Login {
        /// Optional. You will be prompted if you don't provide it.
        #[arg(short, long)]
        username: Option<String>,
    },
}

fn main() {
    // Use `ClapEditor` instead of the `rustyline::DefaultEditor`.
    let mut rl = ClapEditor::<SampleCommand>::new();
    loop {
        // Use `read_command` instead of `readline`.
        let Some(command) = rl.read_command() else {
            continue;
        };
        match command {
            SampleCommand::Download { path, check_sha } => {
                println!("Downloaded {path} with checking = {check_sha}");
            },
            SampleCommand::Upload => {
                println!("Uploaded");
            },
            SampleCommand::Login { username } => {
                // You can use another `rustyline::Editor` inside the loop.
                let mut rl = DefaultEditor::new().unwrap();
                let username = username.unwrap_or_else(|| rl.readline(&style("What is your username? ").bold().to_string()).unwrap());
                let password = rl.readline(&style("What is your password? ").bold().to_string()).unwrap();
                println!("Logged in with {username} and {password}");
            },
        }
    }
}
```
![Screenshot from 2023-06-22 11-32-58](https://github.com/HKalbasi/clap-repl/assets/45197576/2c1b2ceb-e562-4536-8b42-4025d5a9674a)
![Screenshot from 2023-06-22 11-35-33](https://github.com/HKalbasi/clap-repl/assets/45197576/bec9110e-a399-41e4-8f63-1c8592338625)
![image](https://github.com/HKalbasi/clap-repl/assets/45197576/a5eb04c0-fafc-479e-ba1a-dd5757f585be)



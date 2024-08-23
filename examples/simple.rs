use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use clap_repl::reedline::{
    DefaultPrompt, DefaultPromptSegment, FileBackedHistory, Reedline, Signal,
};
use clap_repl::ClapEditor;

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
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("simple-example".to_owned()),
        ..DefaultPrompt::default()
    };
    let rl = ClapEditor::<SampleCommand>::builder()
        .with_prompt(Box::new(prompt))
        .with_editor_hook(|reed| {
            // Do custom things with `Reedline` instance here
            reed.with_history(Box::new(
                FileBackedHistory::with_file(10000, "/tmp/clap-repl-simple-example-history".into())
                    .unwrap(),
            ))
        })
        .build();
    rl.repl(|command| {
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
    });
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

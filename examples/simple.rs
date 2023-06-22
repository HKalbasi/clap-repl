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
                let mut rl = DefaultEditor::new().unwrap();
                let username = username.unwrap_or_else(|| rl.readline(&style("What is your username? ").bold().to_string()).unwrap());
                let password = rl.readline(&style("What is your password? ").bold().to_string()).unwrap();
                println!("Logged in with {username} and {password}");
            },
        }
    }
}

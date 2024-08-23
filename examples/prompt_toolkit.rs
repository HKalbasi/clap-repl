use clap::{Parser, Subcommand, ValueEnum};
use clap_repl::reedline::{DefaultPrompt, DefaultPromptSegment, FileBackedHistory};
use clap_repl::ClapEditor;
use reedline::{default_emacs_keybindings, Emacs, KeyModifiers, ReedlineEvent};

#[derive(Debug, Parser)]
enum SimpleCommand {
    Command1,
    Command2 {
        #[arg(short, long)]
        input: String,
    },
}

#[derive(Debug, Parser)]
#[command(name = "")] // This name will show up in clap's error messages, so it is important to set it to "".
enum ComplexCommand<N = ComplexCommand<SimpleCommand>>
where
    N: Subcommand,
{
    NoInput,
    PositionalInput {
        input: ValueEnumExample,
    },
    ShortInput {
        #[arg(short)]
        input: ValueEnumExample,
    },
    LongInput {
        #[arg(long)]
        input: ValueEnumExample,
    },
    NestedCommand {
        #[command(subcommand)]
        sub_command: Box<N>,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ValueEnumExample {
    /// This is the case 1
    Case1,
    /// This is the case 2
    ///
    /// This paragraph is ignored because there is no long help text for possible values in clap.
    Case2,
    /// A case with a short name
    C,
    CaseWithoutHelp,
}

fn main() {
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("prompt-toolkit".to_owned()),
        ..DefaultPrompt::default()
    };
    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        reedline::KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::Enter,
        ]),
    );
    let rl = ClapEditor::<ComplexCommand>::builder()
        .with_prompt(Box::new(prompt))
        .with_edit_mode(Box::new(Emacs::new(keybindings)))
        .with_editor_hook(|reed| {
            // Do custom things with `Reedline` instance here
            reed.with_history(Box::new(
                FileBackedHistory::with_file(10000, "/tmp/clap-repl-simple-example-history".into())
                    .unwrap(),
            ))
        })
        .build();
    rl.repl(|command| {
        dbg!(command);
    });
}

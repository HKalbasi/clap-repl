use std::marker::PhantomData;

use clap::Parser;
use nu_ansi_term::{Color, Style};
use reedline::{
    default_emacs_keybindings, DefaultHinter, DefaultPrompt, EditMode, Emacs, IdeMenu,
    KeyModifiers, MenuBuilder, Prompt, Reedline, ReedlineEvent, ReedlineMenu,
};

use crate::{ClapEditor, ReedCompleter};

pub struct ClapEditorBuilder<C: Parser + Send + Sync + 'static> {
    prompt: Box<dyn Prompt>,
    edit_mode: Box<dyn EditMode>,
    hook: Box<dyn FnOnce(Reedline) -> Reedline>,
    c_phantom: PhantomData<C>,
}

impl<C: Parser + Send + Sync + 'static> ClapEditorBuilder<C> {
    pub(crate) fn new() -> Self {
        Self {
            prompt: Box::<DefaultPrompt>::default(),
            edit_mode: {
                let mut keybindings = default_emacs_keybindings();
                keybindings.add_binding(
                    KeyModifiers::NONE,
                    reedline::KeyCode::Tab,
                    ReedlineEvent::UntilFound(vec![
                        ReedlineEvent::Menu("completion_menu".to_string()),
                        ReedlineEvent::MenuNext,
                    ]),
                );
                Box::new(Emacs::new(keybindings))
            },
            hook: Box::new(|e| e),
            c_phantom: PhantomData,
        }
    }

    pub fn with_prompt(mut self, prompt: Box<dyn Prompt>) -> Self {
        self.prompt = prompt;
        self
    }

    pub fn with_edit_mode(mut self, edit_mode: Box<dyn EditMode>) -> Self {
        self.edit_mode = edit_mode;
        self
    }

    pub fn with_editor_hook(mut self, hook: impl FnOnce(Reedline) -> Reedline + 'static) -> Self {
        self.hook = Box::new(hook);
        self
    }

    pub fn build(self) -> ClapEditor<C> {
        let completion_menu = Box::new(
            IdeMenu::default()
                .with_default_border()
                .with_name("completion_menu"),
        );

        let rl = Reedline::create()
            .with_completer(Box::new(ReedCompleter::<C> {
                c_phantom: PhantomData,
            }))
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_hinter(Box::new(
                DefaultHinter::default().with_style(Style::new().italic().fg(Color::DarkGray)),
            ))
            .with_edit_mode(self.edit_mode);
        let rl = (self.hook)(rl);
        ClapEditor {
            rl,
            prompt: self.prompt,
            c_phantom: PhantomData,
        }
    }
}

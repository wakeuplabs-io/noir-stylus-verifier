use dialoguer::theme::ColorfulTheme;

pub(crate) struct Dialoguer;

impl Dialoguer {
    pub(crate) fn new() -> Self {
        Self
    }
}

pub(crate) trait TDialoguer: Send + Sync {
    fn prompt(&self, message: &str) -> String;
    fn confirm(&self, message: &str) -> bool;
}

impl TDialoguer for Dialoguer {
    fn prompt(&self, message: &str) -> String {
        dialoguer::Input::with_theme(&ColorfulTheme::default())
            .with_prompt(message)
            .interact_text()
            .expect("Failed to prompt")
    }

    fn confirm(&self, message: &str) -> bool {
        dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(message)
            .interact()
            .expect("Failed to confirm")
    }
}
use crate::{infrastructure::console::progress::style_spinner, AppContext};
use indicatif::ProgressBar;
use std::{env, path::PathBuf};

pub(crate) struct InitCommand {}

impl InitCommand {
    pub(crate) fn new() -> Self {
        Self {
           
        }
    }

    pub(crate) fn run(&self, _ctx: &AppContext, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut root = PathBuf::from(&name);
        if !root.is_absolute() {
            root = env::current_dir()?.join(root)
        }

        if root.exists() {
            return Err(format!("Directory already exists: {}", root.display()).into());
        } else {
            std::fs::create_dir_all(&root)?;
        }

        let create_spinner = style_spinner(
            ProgressBar::new_spinner(),
            &format!("⏳ Creating {} at {}...", name, root.display()),
        );

        // TODO: create project

        create_spinner.finish_with_message(format!(
            "✔️ Success! Created {} at {}\n",
            name,
            root.display()
        ));

        // print instructions ========================================

        // TODO:

        Ok(())
    }
}
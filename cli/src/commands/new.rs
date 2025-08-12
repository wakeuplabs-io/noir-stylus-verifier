//! # New Command
//!
//! The new command creates a new NSV project with all necessary template files
//! and directory structure. It sets up a complete development environment for
//! working with Noir circuits and Stylus verifiers.

use crate::{
    infrastructure::{
        codegen::{Codegen, TCodegen},
        progress::create_spinner,
        system::{System, TSystem},
        terminal::print_instructions,
    },
    AppContext, AppError,
};
use colored::*;
use std::path::PathBuf;

/// Command for creating new NSV projects with template files.
///
/// This command scaffolds a complete NSV project structure including Noir circuit
/// templates, configuration files, and development scripts. It provides everything
/// needed to start developing with Noir circuits and Stylus verifiers.
pub(crate) struct NewCommand {
    /// System operations interface
    system: Box<dyn TSystem>,
    /// Code generation interface for creating project templates
    codegen: Box<dyn TCodegen>,
}

impl Default for NewCommand {
    fn default() -> Self {
        Self {
            system: Box::new(System),
            codegen: Box::new(Codegen::default()),
        }
    }
}

impl NewCommand {
    /// Executes the new command to create a project.
    ///
    /// # Arguments
    ///
    /// * `_ctx` - Application context (currently unused)
    /// * `name` - Name of the project to create. Must follow cargo naming conventions
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if project creation succeeds, or an `AppError` if creation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The project name is invalid (doesn't follow cargo naming rules)
    /// - A directory with the same name already exists
    /// - Project template generation fails
    /// - File system operations fail
    pub(crate) async fn run(&self, _ctx: &AppContext, name: &str) -> Result<(), AppError> {
        // validate name
        self.validate_name(name)
            .map_err(|_| AppError::InvalidName(name.to_string()))?;

        // create project directory
        let root = PathBuf::from(&name);
        if self.system.exists(&root) {
            return Err(AppError::DirectoryAlreadyExists(root.display().to_string()));
        } else {
            self.system.ensure_dir(&root);
        }

        // all good, let's create the project
        let spinner = create_spinner(&format!("⏳ Creating {name}..."));

        // generate project
        let project_files = self
            .codegen
            .generate_project(name)
            .map_err(|_| AppError::GenerateError)?;
        for file in project_files {
            spinner.set_message(format!("Writing {}", file.path));
            self.system.write_file(&root.join(file.path), file.content)
        }

        spinner.finish_and_clear();
        println!("{} Created {name}\n", "✅ Success!".green());
        print_instructions(&["generate", "check", "deploy"]);

        Ok(())
    }

    /// Validates project name according to cargo naming conventions.
    ///
    /// Project names must follow the same rules as cargo package names:
    /// - Only alphanumeric characters, underscores, and hyphens
    /// - Cannot start with a hyphen
    /// - Maximum 64 characters
    /// - Cannot be empty
    ///
    /// # Arguments
    ///
    /// * `name` - The project name to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the name is valid, or an error describing the issue.
    ///
    /// # Reference
    ///
    /// Same validation as defined by cargo: https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field
    fn validate_name(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if name.is_empty() {
            return Err("Name is required".into());
        }
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(
                "Name can only contain alphanumeric characters, underscores, or hyphens".into(),
            );
        }
        if name.starts_with('-') {
            return Err("Name cannot start with a hyphen".into());
        }
        if name.len() > 64 {
            return Err("Name cannot be longer than 64 characters".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Name must be valid nargo package name as per https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field
    #[test]
    fn validate_name_comprehensive() {
        let command = NewCommand::default();

        // Test all valid patterns
        let valid_patterns = vec![
            // Basic patterns
            ("hello", "simple name"),
            ("hello_world", "with underscore"),
            ("hello-world", "with hyphen"),
            ("hello123", "with numbers"),
            ("123hello", "starting with numbers"),
            ("a", "single character"),
            // Complex patterns
            ("hello_world_123", "mixed underscores and numbers"),
            ("hello-world-123", "mixed hyphens and numbers"),
            ("hello_world-123", "mixed underscores and hyphens"),
            ("a_very_long_name_with_underscores", "long with underscores"),
            ("a-very-long-name-with-hyphens", "long with hyphens"),
            // Edge cases
            (
                "64_char_name_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "exactly 64 chars",
            ),
        ];

        for (name, description) in valid_patterns {
            assert!(
                command.validate_name(name).is_ok(),
                "Should be valid: {} ({})",
                name,
                description
            );
        }

        // Test all invalid patterns
        let invalid_patterns = vec![
            // Empty and whitespace
            ("", "empty string"),
            (" ", "single space"),
            ("  ", "multiple spaces"),
            // Starting with hyphen
            ("-hello", "starts with hyphen"),
            ("-", "just hyphen"),
            // Invalid characters
            ("hello world", "contains space"),
            ("hello.world", "contains dot"),
            ("hello@world", "contains at symbol"),
            ("hello!world", "contains exclamation"),
            ("hello#world", "contains hash"),
            ("hello$world", "contains dollar"),
            ("hello%world", "contains percent"),
            ("hello^world", "contains caret"),
            ("hello&world", "contains ampersand"),
            ("hello*world", "contains asterisk"),
            ("hello(world", "contains open paren"),
            ("hello)world", "contains close paren"),
            ("hello+world", "contains plus"),
            ("hello=world", "contains equals"),
            ("hello[world", "contains open bracket"),
            ("hello]world", "contains close bracket"),
            ("hello{world", "contains open brace"),
            ("hello}world", "contains close brace"),
            ("hello|world", "contains pipe"),
            ("hello\\world", "contains backslash"),
            ("hello:world", "contains colon"),
            ("hello;world", "contains semicolon"),
            ("hello\"world", "contains quote"),
            ("hello'world", "contains apostrophe"),
            ("hello<world", "contains less than"),
            ("hello>world", "contains greater than"),
            ("hello,world", "contains comma"),
            ("hello?world", "contains question mark"),
            ("hello/world", "contains forward slash"),
            ("hello~world", "contains tilde"),
            ("hello`world", "contains backtick"),
            // Too long
            (
                "this_is_a_very_long_project_name_that_exceeds_the_maximum_allowed_length",
                "way too long",
            ),
        ];

        for (name, description) in invalid_patterns {
            assert!(
                command.validate_name(name).is_err(),
                "Should be invalid: {} ({})",
                name,
                description
            );
        }
    }
}

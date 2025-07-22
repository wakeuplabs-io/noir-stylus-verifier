use crate::{
    infrastructure::{
        codegen::{Codegen, TCodegen},
        progress::create_spinner,
        system::{System, TSystem},
    },
    AppContext, AppError,
};
use colored::*;
use std::{env, path::PathBuf};

pub(crate) struct NewCommand {
    system: Box<dyn TSystem>,
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

        spinner.finish_with_message(format!("{} Created {name}\n", "✅ Success!".green()));

        // print instructions ========================================

        println!(
            "\n {title}\n\n  - {bin} {generate_cmd}: Generates a new verifier contract.\n  - {bin} {check_cmd}: Checks the verifier contract.\n  - {bin} {deploy_cmd}: Deploys the verifier to the blockchain.\n",
            title = "What's Next?".bright_white().bold(),
            bin = env!("CARGO_BIN_NAME").blue(),
            generate_cmd = "generate".blue(),
            check_cmd = "check".blue(),
            deploy_cmd = "deploy".blue(),
        );

        Ok(())
    }

    /// Same as defined by cargo https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field
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
    use crate::infrastructure::{
        codegen::{MockTCodegen, ProjectFile},
        system::MockTSystem,
    };
    use mockall::predicate::*;

    const PROJECT_NAME: &str = "hello_world";

    #[test]
    fn test_validate_name() {
        let command = NewCommand::default();

        let valid_names = vec![
            "hello_world",
            "hello-world",
            "hello_world_123",
            "hello",
            "a_very_long_name_123",
        ];
        let invalid_names = vec![
            "",
            "hello world",
            "hello_world!",
            "hello@world",
            "hello.world",
            "64_chars_is_the_max_length_for_a_project_name_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ];

        for name in valid_names {
            assert!(command.validate_name(name).is_ok());
        }

        for name in invalid_names {
            assert!(command.validate_name(name).is_err());
        }
    }

    #[tokio::test]
    async fn test_new_command() {
        let mocked_project_files = vec![ProjectFile {
            path: "README.md".to_string(),
            content: "".to_string(),
        }];

        // check we verify folder doesn't exist before creating it. Then that we write all the files
        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PROJECT_NAME)))
            .returning(|_| false);
        system_mock.expect_ensure_dir().returning(|_| ());
        system_mock
            .expect_write_file()
            .with(
                eq(PathBuf::from(PROJECT_NAME).join("README.md")),
                eq(mocked_project_files[0].content.clone()),
            )
            .returning(|_, _| ());

        // check we generate the project files
        let mut codegen_mock = MockTCodegen::new();
        codegen_mock
            .expect_generate_project()
            .with(eq(PROJECT_NAME))
            .returning(move |_| Ok(mocked_project_files.clone()));

        // run the command
        let result = NewCommand {
            system: Box::new(system_mock),
            codegen: Box::new(codegen_mock),
        }
        .run(&AppContext {}, PROJECT_NAME)
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_new_command_invalid_name() {
        let result = NewCommand::default().run(&AppContext {}, "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_new_existing_folder() {
        let mut system_mock = MockTSystem::new();
        system_mock
            .expect_exists()
            .with(eq(PathBuf::from(PROJECT_NAME)))
            .returning(|_| true);

        let result = NewCommand {
            system: Box::new(system_mock),
            codegen: Box::new(MockTCodegen::new()),
        }
        .run(&AppContext {}, PROJECT_NAME)
        .await;

        assert!(result.is_err());
    }
}

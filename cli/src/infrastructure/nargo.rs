use crate::infrastructure::system::{System, TSystem};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TNargo {
    fn find_package_root(&self, package: &str) -> Result<PathBuf, Box<dyn std::error::Error>>;
    fn read_package_name(&self, root: &Path) -> Result<String, Box<dyn std::error::Error>>;
    fn setup(&self, version: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn compile(&self, circuit_path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    fn execute(&self, root: &Path, package_name: &str) -> Result<(), Box<dyn std::error::Error>>;
}

pub(crate) struct Nargo {
    system: Box<dyn TSystem>,
}

impl Default for Nargo {
    fn default() -> Self {
        Self {
            system: Box::new(System),
        }
    }
}

impl TNargo for Nargo {
    fn find_package_root(&self, package: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let root = PathBuf::from(package);
        if self.system.exists(&root.join("Nargo.toml")) {
            Ok(root)
        } else {
            Err(format!("Circuit not found at: {}", root.display()).into())
        }
    }

    fn read_package_name(&self, root: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let content = self.system.read_file_str(&root.join("Nargo.toml"));
        let toml: toml::Value = toml::from_str(&content)?;

        if let Some(name) = toml
            .get("package")
            .and_then(|pkg| pkg.get("name"))
            .and_then(toml::Value::as_str)
        {
            Ok(name.to_string())
        } else {
            Err(format!("Package name not found in Nargo.toml at {}", root.display()).into())
        }
    }

    fn setup(&self, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.system
            .execute_command(Command::new("noirup").arg("-v").arg(version))?;

        Ok(())
    }

    fn compile(&self, circuit_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.system.execute_command(
            Command::new("nargo")
                .arg("compile")
                .current_dir(circuit_path),
        )?;

        Ok(())
    }

    fn execute(&self, root: &Path, package_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO:
        self.system.execute_command(
            Command::new("nargo")
                .arg("execute")
                .arg("--package")
                .arg(package_name)
                .current_dir(root),
        )?;

        Ok(())
    }
}

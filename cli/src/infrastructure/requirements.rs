use crate::infrastructure::{
    system::{System, TSystem},
    utils::{Sha256Hasher, TSha256Hasher},
};
use regex::Regex;
use semver::Version;
use std::{fmt, process::Command};

#[derive(Debug, Eq, PartialEq, Clone)]
#[allow(dead_code)]
pub(crate) enum Comparison {
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
    GreaterThan,
    LessThan,
    Installed,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) struct Requirement<'a> {
    pub(crate) program: &'a str,
    pub(crate) version_arg: &'a str,
    pub(crate) required_version: &'a str,
    pub(crate) required_hash: &'a [&'a str],
    pub(crate) required_comparator: Comparison,
}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TSystemRequirementsChecker: Send + Sync {
    fn check<'a>(&self, requirements: Vec<Requirement<'a>>) -> Result<(), String>;

    fn check_by_hash<'a>(&self, requirements: Vec<Requirement<'a>>) -> Result<(), String>;

    fn check_by_version<'a>(&self, requirements: Vec<Requirement<'a>>) -> Result<(), String>;
}

pub(crate) struct SystemRequirementsChecker {
    system: Box<dyn TSystem>,
    hasher: Box<dyn TSha256Hasher>,
}

// implementations =============================================

impl fmt::Display for Comparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Comparison::Equal => write!(f, "=="),
            Comparison::GreaterThanOrEqual => write!(f, ">="),
            Comparison::LessThanOrEqual => write!(f, "<="),
            Comparison::GreaterThan => write!(f, ">"),
            Comparison::LessThan => write!(f, "<"),
            Comparison::Installed => write!(f, "installed"),
        }
    }
}

impl Default for SystemRequirementsChecker {
    fn default() -> Self {
        Self {
            system: Box::new(System),
            hasher: Box::new(Sha256Hasher),
        }
    }
}

impl TSystemRequirementsChecker for SystemRequirementsChecker {
    fn check(&self, requirements: Vec<Requirement>) -> Result<(), String> {
        // hash has priority over version as some programs don't have a version command
        let version_requirements: Vec<_> = requirements
            .iter()
            .filter(|r| r.required_hash.is_empty())
            .cloned()
            .collect();
        let hash_requirements: Vec<_> = requirements
            .iter()
            .filter(|r| !r.required_hash.is_empty())
            .cloned()
            .collect();

        self.check_by_version(version_requirements)?;
        self.check_by_hash(hash_requirements)?;

        Ok(())
    }

    fn check_by_hash(&self, requirements: Vec<Requirement>) -> Result<(), String> {
        for requirement in requirements.iter() {
            let path = self
                .system
                .which(requirement.program)
                .ok_or(format!("{} not found", requirement.program))?;
            let hash = self
                .hasher
                .hash(&path)
                .map_err(|_| format!("Failed to calculate hash for {}", requirement.program))?;

            if !requirement.required_hash.contains(&hash.as_str()) {
                return Err(format!(
                    "Hash {} does not match required hash {}",
                    hash,
                    requirement.required_hash.join(", ")
                ));
            }
        }

        Ok(())
    }

    fn check_by_version(&self, requirements: Vec<Requirement>) -> Result<(), String> {
        let re = Regex::new(r"(\d+\.\d+\.\d+(?:-[a-z]+\.\d+)?)").expect("Failed to compile regex");

        for requirement in requirements.iter() {
            let mut installed = true;
            let mut version = Version::parse("0.0.0").unwrap();
            let mut required_version = Version::parse("0.0.0").unwrap();

            if requirement.required_comparator == Comparison::Installed {
                if self
                    .system
                    .execute_command(Command::new("which").arg(requirement.program))
                    .is_err()
                {
                    installed = false;
                }
            } else {
                let output = self
                .system
                .execute_command(Command::new(requirement.program).arg(requirement.version_arg))
                .map_err(|_| {
                    format!(
                        "{} {} did not exited succesfully. Please ensure program is installed and running.",
                        requirement.program, requirement.version_arg
                    )
                })?;

                version = Version::parse(
                    &re.captures(&output)
                        .ok_or(format!("Failed to parse version from output: {output}"))?[1],
                )
                .expect("Failed to parse version");
                required_version = Version::parse(requirement.required_version).unwrap();
            }

            match requirement.required_comparator {
                Comparison::Equal => {
                    if version != required_version {
                        return Err(format!(
                            "Version {} does not equal required version {}",
                            version, requirement.required_version
                        ));
                    }
                }
                Comparison::GreaterThanOrEqual => {
                    if version < required_version {
                        return Err(format!(
                            "Version {} is not greater than or equal to required version {}",
                            version, requirement.required_version
                        ));
                    }
                }
                Comparison::LessThanOrEqual => {
                    if version > required_version {
                        return Err(format!(
                            "Version {} is not less than or equal to required version {}",
                            version, requirement.required_version
                        ));
                    }
                }
                Comparison::GreaterThan => {
                    if version <= required_version {
                        return Err(format!(
                            "Version {} is not greater than required version {}",
                            version, requirement.required_version
                        ));
                    }
                }
                Comparison::LessThan => {
                    if version >= required_version {
                        return Err(format!(
                            "Version {} is not less than required version {}",
                            version, requirement.required_version
                        ));
                    }
                }
                Comparison::Installed => {
                    if !installed {
                        return Err(format!(
                            "{} is not installed. Please install it.",
                            requirement.program
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::system::MockTSystem;
    use crate::infrastructure::utils::MockTSha256Hasher;
    use std::path::PathBuf;

    #[test]
    fn test_check_by_version() {
        let requirements = vec![Requirement {
            program: "rustc",
            version_arg: "--version",
            required_version: "1.0.0",
            required_comparator: Comparison::GreaterThanOrEqual,
            required_hash: &[],
        }];

        let mut mock_system = MockTSystem::new();
        mock_system
            .expect_execute_command()
            .times(1)
            .returning(|_| Ok("rustc 1.0.0".to_string()));

        let mut mock_hasher = MockTSha256Hasher::new();
        mock_hasher.expect_hash().never();

        let checker = SystemRequirementsChecker {
            system: Box::new(mock_system),
            hasher: Box::new(mock_hasher),
        };
        let result = checker.check(requirements);

        assert!(result.is_ok());
    }

    #[test]
    fn test_check_by_version_fails() {
        let requirements = vec![Requirement {
            program: "rustc",
            version_arg: "--version",
            required_version: "1.0.0",
            required_comparator: Comparison::GreaterThanOrEqual,
            required_hash: &[],
        }];

        let mut mock_system = MockTSystem::new();
        mock_system
            .expect_execute_command()
            .times(1)
            .returning(|_| Ok("rustc 0.9.0".to_string()));

        let mut mock_hasher = MockTSha256Hasher::new();
        mock_hasher.expect_hash().never();

        let checker = SystemRequirementsChecker {
            system: Box::new(mock_system),
            hasher: Box::new(mock_hasher),
        };
        let result = checker.check(requirements);

        assert!(result.is_err());
    }

    #[test]
    fn test_check_by_hash() {
        let requirements = vec![Requirement {
            program: "bb",
            version_arg: "--version",
            required_version: "0.86.0",
            required_comparator: Comparison::Equal,
            required_hash: &["0caa9112cd5e446ea336ef9476f0532366dd856f0b2c4ffbd0abd32635c10052"],
        }];

        let mut mock_system = MockTSystem::new();
        mock_system
            .expect_which()
            .times(1)
            .returning(|_| Some(PathBuf::from("/usr/local/bin/bb")));

        let mut mock_hasher = MockTSha256Hasher::new();
        mock_hasher.expect_hash().times(1).returning(|_| {
            Ok("0caa9112cd5e446ea336ef9476f0532366dd856f0b2c4ffbd0abd32635c10052".to_string())
        });

        let checker = SystemRequirementsChecker {
            system: Box::new(mock_system),
            hasher: Box::new(mock_hasher),
        };
        let result = checker.check(requirements);

        assert!(result.is_ok());
    }
}

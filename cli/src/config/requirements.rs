use crate::infrastructure::system::{System, TSystem};
use regex::Regex;
use semver::Version;
use std::{fmt, process::Command};

#[derive(Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum Comparison {
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
    GreaterThan,
    LessThan,
    Installed,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Requirement<'a> {
    pub(crate) program: &'a str,
    pub(crate) version_arg: &'a str,
    pub(crate) required_version: &'a str,
    pub(crate) required_comparator: Comparison,
}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TSystemRequirementsChecker: Send + Sync {
    fn check<'a>(&self, requirements: Vec<Requirement<'a>>) -> Result<(), String>;
}

pub(crate) struct SystemRequirementsChecker {
    system: Box<dyn TSystem>,
}

// requirements constants ======================================

pub(crate) const CARGO_STYLUS_REQUIREMENT: Requirement = Requirement {
    program: "cargo-stylus",
    version_arg: "--version",
    required_version: "0.1.0",
    required_comparator: Comparison::GreaterThanOrEqual,
};
pub(crate) const BB_UP_REQUIREMENT: Requirement = Requirement {
    program: "bbup",
    version_arg: "",
    required_version: "",
    required_comparator: Comparison::Installed,
};
pub(crate) const BB_REQUIREMENT: Requirement = Requirement {
    program: "bb",
    version_arg: "--version",
    required_version: "0.86.0",
    required_comparator: Comparison::Equal,
};
pub(crate) const NOIRUP_REQUIREMENT: Requirement = Requirement {
    program: "noirup",
    version_arg: "",
    required_version: "",
    required_comparator: Comparison::Installed,
};
pub(crate) const NOIR_REQUIREMENT: Requirement = Requirement {
    program: "noir",
    version_arg: "--version",
    required_version: "1.0.0-beta.6",
    required_comparator: Comparison::Equal,
};

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
            system: Box::new(System::default()),
        }
    }
}

impl TSystemRequirementsChecker for SystemRequirementsChecker {
    fn check(&self, requirements: Vec<Requirement>) -> Result<(), String> {
        let re = Regex::new(r"(\d+\.\d+\.\d+)").expect("Failed to compile regex");

        for requirement in requirements.iter() {
            let mut installed;
            let mut version = Version::parse("0.0.0").unwrap();
            let mut required_version = Version::parse("0.0.0").unwrap();

            if requirement.required_comparator == Comparison::Installed {
                if self
                    .system
                    .execute_command(Command::new("which").arg(requirement.program))
                    .is_err()
                {
                    return Err(format!(
                        "{} is not installed. Please install it.",
                        requirement.program
                    ));
                }

                installed = true;
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

                installed = true;
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
    use crate::config::requirements::Comparison;
    use crate::config::requirements::Requirement;
    use crate::config::requirements::SystemRequirementsChecker;
    use crate::infrastructure::system::MockTSystem;

    #[test]
    fn test_check() {
        let requirements = vec![Requirement {
            program: "rustc",
            version_arg: "--version",
            required_version: "1.0.0",
            required_comparator: Comparison::GreaterThanOrEqual,
        }];

        let mut mock_system = MockTSystem::new();
        mock_system
            .expect_execute_command()
            .times(1)
            .returning(|_| Ok("rustc 1.0.0".to_string()));

        let checker = SystemRequirementsChecker {
            system: Box::new(mock_system),
        };
        let result = checker.check(requirements);

        assert!(result.is_ok());
    }

    #[test]
    fn test_check_fails() {
        let requirements = vec![Requirement {
            program: "rustc",
            version_arg: "--version",
            required_version: "1.0.0",
            required_comparator: Comparison::GreaterThanOrEqual,
        }];

        let mut mock_system = MockTSystem::new();
        mock_system
            .expect_execute_command()
            .times(1)
            .returning(|_| Ok("rustc 0.9.0".to_string()));

        let checker = SystemRequirementsChecker {
            system: Box::new(mock_system),
        };
        let result = checker.check(requirements);

        assert!(result.is_err());
    }
}

//! # Code Generation
//!
//! This module handles code generation for NSV projects and Stylus verifier contracts.
//! It uses the Tera templating engine to generate files from predefined templates,
//! enabling creation of complete project structures and verifier contracts.

use crate::infrastructure::templates::{PROJECT_TEMPLATES, VERIFIER_TEMPLATES};
use tera::Tera;

/// Code generation engine for creating projects and verifier contracts.
/// 
/// This struct provides template-based code generation functionality using Tera.
/// It can generate complete NSV project structures and Stylus verifier contracts
/// from predefined templates with dynamic content substitution.
#[derive(Default)]
pub(crate) struct Codegen {}

/// Trait defining the interface for code generation operations.
/// 
/// This trait abstracts code generation functionality to enable testing
/// and different implementation strategies for template-based file generation.
#[cfg_attr(test, mockall::automock)]
pub(crate) trait TCodegen: Send + Sync + 'static {
    /// Generates a complete NSV project with all necessary files.
    /// 
    /// Creates a new NSV project structure including Noir circuit templates,
    /// configuration files, scripts, and documentation. The project name is
    /// used as a template variable for dynamic content generation.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the project to generate
    /// 
    /// # Returns
    /// 
    /// Returns a vector of `ProjectFile` structs containing file paths and
    /// content to be written to disk, or an error if template rendering fails.
    fn generate_project(&self, name: &str) -> Result<Vec<ProjectFile>, Box<dyn std::error::Error>>;

    /// Generates a Stylus verifier contract from templates.
    /// 
    /// Creates all necessary files for a Stylus verifier contract including
    /// Rust source code, Cargo configuration, toolchain settings, and build
    /// artifacts. The project name is used to customize the generated contract.
    /// 
    /// # Arguments
    /// 
    /// * `project_name` - The name of the project to generate the verifier for
    /// 
    /// # Returns
    /// 
    /// Returns a vector of `ProjectFile` structs containing file paths and
    /// content to be written to disk, or an error if template rendering fails.
    fn generate_verifier_contract(
        &self,
        project_name: &str,
    ) -> Result<Vec<ProjectFile>, Box<dyn std::error::Error>>;
}

/// Represents a file to be generated with its path and content.
/// 
/// This struct encapsulates the output of template rendering, containing
/// the relative path where the file should be written and its generated content.
#[cfg_attr(test, derive(Debug, PartialEq, Eq, Clone))]
pub(crate) struct ProjectFile {
    /// Relative path where the file should be written
    pub(crate) path: String,
    /// Generated content of the file
    pub(crate) content: String,
}

impl TCodegen for Codegen {
    /// Generates a complete NSV project using predefined templates.
    /// 
    /// This implementation uses the Tera templating engine to process all
    /// project templates with the provided project name as context.
    fn generate_project(&self, name: &str) -> Result<Vec<ProjectFile>, Box<dyn std::error::Error>> {
        // build tera context
        let context = tera::Context::from_serialize(serde_json::json!({
            "project_name": name,
        }))?;

        // render all templates
        Ok(PROJECT_TEMPLATES
            .iter()
            .map(|(path, template)| {
                Ok(ProjectFile {
                    path: path.to_string(),
                    content: Tera::one_off(template, &context, false)?,
                })
            })
            .collect::<Result<Vec<_>, tera::Error>>()?)
    }

    /// Generates a Stylus verifier contract using predefined templates.
    /// 
    /// This implementation uses the Tera templating engine to process all
    /// verifier contract templates with the provided project name as context.
    fn generate_verifier_contract(
        &self,
        project_name: &str,
    ) -> Result<Vec<ProjectFile>, Box<dyn std::error::Error>> {
        // build tera context
        let context = tera::Context::from_serialize(serde_json::json!({
            "project_name": project_name,
        }))?;

        // render all templates
        Ok(VERIFIER_TEMPLATES
            .iter()
            .map(|(path, template)| {
                Ok(ProjectFile {
                    path: path.to_string(),
                    content: Tera::one_off(template, &context, false)?,
                })
            })
            .collect::<Result<Vec<_>, tera::Error>>()?)
    }
}

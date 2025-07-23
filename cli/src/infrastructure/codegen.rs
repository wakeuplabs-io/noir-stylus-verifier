use crate::infrastructure::templates::{PROJECT_TEMPLATES, VERIFIER_TEMPLATES};
use tera::Tera;

#[derive(Default)]
pub(crate) struct Codegen {}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TCodegen: Send + Sync + 'static {
    /// Generates a new example project, returns files and their content to be written to disk
    fn generate_project(&self, name: &str) -> Result<Vec<ProjectFile>, Box<dyn std::error::Error>>;

    /// Generates a verifier contract, returns files and their content to be written to disk
    fn generate_verifier_contract(
        &self,
        project_name: &str,
    ) -> Result<Vec<ProjectFile>, Box<dyn std::error::Error>>;
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq, Clone))]
pub(crate) struct ProjectFile {
    pub(crate) path: String,
    pub(crate) content: String,
}

impl TCodegen for Codegen {
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

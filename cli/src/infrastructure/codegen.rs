use crate::infrastructure::{
    system::{System, TSystem},
    templates::{PROJECT_TEMPLATES, VERIFIER_TEMPLATES},
};
use std::path::Path;
use tera::Tera;

pub(crate) struct Codegen {
    system: Box<dyn TSystem>,
}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait TCodegen: Send + Sync + 'static {
    /// Generates a new example project, returns files and their content to be written to disk
    fn generate_project(&self, name: &str) -> Result<Vec<ProjectFile>, Box<dyn std::error::Error>>;

    /// Generates a verifier contract, returns files and their content to be written to disk
    fn generate_verifier_contract(
        &self,
        circuit_json_path: &Path,
        vk_path: &Path,
    ) -> Result<Vec<ProjectFile>, Box<dyn std::error::Error>>;
}

struct CircuitInputs {
    name: String,
    visibility: String,
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq, Clone))]
pub(crate) struct ProjectFile {
    pub(crate) path: String,
    pub(crate) content: String,
}

impl Default for Codegen {
    fn default() -> Self {
        Self {
            system: Box::new(System),
        }
    }
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
        circuit_json_path: &Path,
        vk_path: &Path,
    ) -> Result<Vec<ProjectFile>, Box<dyn std::error::Error>> {
        // generate vk bytes
        let vk_bytes = self.system.read_file(vk_path)?;
        let vk_bytes_str = format!(
            "[{}].into()",
            vk_bytes
                .iter()
                .map(|b| format!("{b}"))
                .collect::<Vec<String>>()
                .join(", ")
        );

        // generate inputs prototype and serialization
        let circuit_json_str = self.system.read_file_str(circuit_json_path)?;
        let circuit_json: serde_json::Value = serde_json::from_str(&circuit_json_str)?;
        let circuit_inputs: Vec<CircuitInputs> = circuit_json["abi"]["parameters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|input| CircuitInputs {
                name: input["name"].as_str().unwrap().to_string(),
                visibility: input["visibility"].as_str().unwrap().to_string(),
            })
            .collect::<Vec<CircuitInputs>>();

        let public_inputs = circuit_inputs
            .iter()
            .filter(|input| input.visibility == "public")
            .collect::<Vec<&CircuitInputs>>();

        // generate inputs prototype and serialization
        let mut inputs_prototype_str = "".to_string();
        let mut inputs_serialization_str = "[].into()".to_string();
        if !public_inputs.is_empty() {
            inputs_prototype_str = ", ".to_string()
                + &public_inputs
                    .iter()
                    .map(|input| format!("{}: Bytes", input.name))
                    .collect::<Vec<String>>()
                    .join(", ");
            inputs_serialization_str = format!(
                "[{}].concat().into()",
                public_inputs
                    .iter()
                    .map(|input| format!("{}.to_vec()", input.name))
                    .collect::<Vec<String>>()
                    .join(", ")
            );
        };
        let circuit_comment = circuit_json_str.replace("\n", "");

        // build tera context
        let context = tera::Context::from_serialize(serde_json::json!({
            "vk_bytes": vk_bytes_str,
            "inputs_prototype": inputs_prototype_str,
            "inputs_serialization": inputs_serialization_str,
            "circuit_comment": circuit_comment,
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

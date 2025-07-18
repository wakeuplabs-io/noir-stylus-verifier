use crate::infrastructure::system::{System, TSystem};
use std::{fmt::format, path::Path};
use tera::Tera;

struct CircuitInputs {
    name: String,
    visibility: String,
}

pub(crate) struct ProjectFile {
    pub(crate) path: String,
    pub(crate) content: String,
}

pub(crate) struct VerifierGenerator {
    system: System,
}

impl VerifierGenerator {
    pub(crate) fn new() -> Self {
        Self {
            system: System::new(),
        }
    }

    pub(crate) fn generate_verifier_contract(
        &self,
        circuit_json_path: &Path,
        vk_path: &Path,
    ) -> Result<Vec<ProjectFile>, Box<dyn std::error::Error>> {
        // generate vk bytes
        let vk_bytes = self.system.read_file(vk_path)?;
        let vk_bytes_str = format!("[{}].into()", vk_bytes
            .iter()
            .map(|b| format!("{}", b))
            .collect::<Vec<String>>()
            .join(", "));

        // generate inputs prototype and serialization
        let circuit_json_str = self.system.read_file_str(circuit_json_path)?;
        let circuit_inputs = self.parse_circuit_inputs(&circuit_json_str)?;
        let public_inputs = circuit_inputs
            .iter()
            .filter(|input| input.visibility == "public")
            .collect::<Vec<&CircuitInputs>>();
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
        let mut context = tera::Context::new();
        context.insert("vk_bytes", &vk_bytes_str);
        context.insert("inputs_prototype", &inputs_prototype_str);
        context.insert("inputs_serialization", &inputs_serialization_str);
        context.insert("circuit_comment", &circuit_comment);

        // add templates
        let mut tera = Tera::default();
        tera.add_raw_template("src/main.rs", include_str!("templates/src/main.rs.tera"))?;
        tera.add_raw_template("src/lib.rs", include_str!("templates/src/lib.rs.tera"))?;
        tera.add_raw_template(".gitignore", include_str!("templates/.gitignore.tera"))?;
        tera.add_raw_template("Cargo.toml", include_str!("templates/Cargo.toml.tera"))?;
        tera.add_raw_template("Cargo.lock", include_str!("templates/Cargo.lock.tera"))?;
        tera.add_raw_template(
            "rust-toolchain.toml",
            include_str!("templates/rust-toolchain.toml.tera"),
        )?;

        Ok(vec![
            ProjectFile {
                path: "src/main.rs".to_string(),
                content: tera.render("src/main.rs", &context)?,
            },
            ProjectFile {
                path: "src/lib.rs".to_string(),
                content: tera.render("src/lib.rs", &context)?,
            },
            ProjectFile {
                path: ".gitignore".to_string(),
                content: tera.render(".gitignore", &context)?,
            },
            ProjectFile {
                path: "Cargo.toml".to_string(),
                content: tera.render("Cargo.toml", &context)?,
            },
            ProjectFile {
                path: "Cargo.lock".to_string(),
                content: tera.render("Cargo.lock", &context)?,
            },
            ProjectFile {
                path: "rust-toolchain.toml".to_string(),
                content: tera.render("rust-toolchain.toml", &context)?,
            },
        ])
    }

    fn parse_circuit_inputs(
        &self,
        circuit_json_str: &str,
    ) -> Result<Vec<CircuitInputs>, Box<dyn std::error::Error>> {
        let circuit_json: serde_json::Value = serde_json::from_str(&circuit_json_str)?;
        let circuit_abi = circuit_json["abi"].clone();
        let circuit_inputs = circuit_abi["parameters"].clone();
        let circuit_inputs = circuit_inputs.as_array().unwrap();
        let circuit_inputs: Vec<CircuitInputs> = circuit_inputs
            .iter()
            .map(|input| CircuitInputs {
                name: input["name"].as_str().unwrap().to_string(),
                visibility: input["visibility"].as_str().unwrap().to_string(),
            })
            .collect::<Vec<CircuitInputs>>();

        Ok(circuit_inputs)
    }
}

use crate::{
    infrastructure::{
        bb::{Bb, TBb},
        nargo::{Nargo, TNargo},
        progress::create_spinner,
        system::{System, TSystem},
    },
    AppContext, AppError,
};
use colored::*;

pub(crate) struct ProveCommand {
    system: Box<dyn TSystem>,
    bb: Box<dyn TBb>,
    nargo: Box<dyn TNargo>,
}

impl Default for ProveCommand {
    fn default() -> Self {
        Self {
            system: Box::new(System),
            bb: Box::new(Bb::default()),
            nargo: Box::new(Nargo::default()),
        }
    }
}

impl ProveCommand {
    pub(crate) async fn run(
        &self,
        _ctx: &AppContext,
        package: Option<String>,
        zk: bool,
    ) -> Result<(), AppError> {
        let root = match package {
            Some(package) => self
                .nargo
                .find_package_root(&package)
                .map_err(|_| AppError::PackageNotFound)?,
            None => self.system.current_dir(),
        };
        let package_name = self
            .nargo
            .read_package_name(&root)
            .map_err(|_| AppError::PackageNotFound)?;

        // All good, let's generate the proof

        let spinner = create_spinner(&format!(
            "⏳ Creating proof for {package_name} at {}...",
            root.display()
        ));

        self.nargo
            .execute(&root, &package_name)
            .map_err(|_| AppError::Other("Failed to execute nargo"))?;
        self.bb
            .prove(&root, &package_name, zk)
            .map_err(|_| AppError::Other("Failed to generate proof"))?;

        spinner.finish_with_message(format!(
            "{} Proof generated at {}\n",
            "✅ Success!".green(),
            root.join("target").join("proof").display()
        ));

        Ok(())
    }
}

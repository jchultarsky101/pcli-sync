use log::{error, trace, warn};
use pcli::model::Model;
use serde_json;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SynchronizerError {
    #[error("deletion error")]
    DeletionError,
    #[error("creation error")]
    UploadError,
    #[error("I/O error")]
    InputOutputError(#[from] std::io::Error),
    #[error("serialization error")]
    SerializationError(#[from] serde_json::Error),
}

pub struct Synchronizer {
    folder_id: u32,
    tenant: String,
    units: String,
}

impl Synchronizer {
    pub fn new(tenant: String, folder_id: u32, units: String) -> Synchronizer {
        Synchronizer {
            tenant,
            folder_id,
            units,
        }
    }

    pub fn init(&self) -> Result<(), SynchronizerError> {
        println!(
            "Initiating new session for tenant {tenant}...",
            tenant = self.tenant.clone()
        );

        let executable = "pcli";
        let subcommand = "invalidate";

        let output = Command::new(executable)
            .arg("--tenant")
            .arg(self.tenant.to_owned())
            .arg(subcommand)
            .output()?;

        trace!("Output: {}", String::from_utf8(output.stdout).unwrap());

        if !output.status.success() {
            error!("Command executed with failing error code {}", output.status);
            Err(SynchronizerError::UploadError)
        } else {
            println!("Session initialized.");
            Ok(())
        }
    }

    fn is_valid_path(&self, path: &Path) -> bool {
        let file_name = path.file_name();
        match file_name {
            Some(file_name) => {
                if file_name
                    .to_os_string()
                    .into_string()
                    .unwrap()
                    .starts_with(".")
                {
                    false
                } else {
                    true
                }
            }
            None => false,
        }
    }

    pub fn upload(&self, path: &Path) -> Result<(), SynchronizerError> {
        if !self.is_valid_path(path) {
            return Ok(());
        }

        println!(
            "Uploading {path} for tenant {tenant} to folder {folder_id}...",
            path = path.as_os_str().to_str().unwrap(),
            tenant = self.tenant.clone(),
            folder_id = self.folder_id,
        );

        let executable = "pcli";
        let subcommand = "upload";

        let output = Command::new(executable)
            .arg("--tenant")
            .arg(self.tenant.to_owned())
            .arg(subcommand)
            .arg("--input")
            .arg(path.as_os_str().to_str().unwrap())
            .arg("--folder")
            .arg(self.folder_id.to_string())
            .arg("--units")
            .arg(self.units.to_owned())
            .output()?;

        trace!("Output: {}", String::from_utf8(output.stdout).unwrap());

        if !output.status.success() {
            error!("Command executed with failing error code {}", output.status);
            Err(SynchronizerError::UploadError)
        } else {
            // parse the output and get the model UUID
            Ok(())
        }
    }

    pub fn delete(&self, path: &Path) -> Result<(), SynchronizerError> {
        if !self.is_valid_path(path) {
            return Ok(());
        }

        println!(
            "Deleting {path} from tenant {tenant} in folder {folder_id}...",
            path = path.as_os_str().to_str().unwrap(),
            tenant = self.tenant.clone(),
            folder_id = self.folder_id,
        );

        let name = path.file_stem().unwrap().to_str().unwrap();

        let executable = "pcli";
        let subcommand = "models";

        let output = Command::new(executable)
            .arg("--tenant")
            .arg(self.tenant.to_owned())
            .arg(subcommand)
            .arg("--folder")
            .arg(self.folder_id.to_string())
            .arg("--search")
            .arg(name)
            .output()?;

        let json = String::from_utf8(output.stdout).unwrap();
        trace!("Output: {}", json);

        if !output.status.success() {
            error!("Command executed with failing error code {}", output.status);
            Err(SynchronizerError::DeletionError)
        } else {
            // parse the output and get the model UUID
            let models: Vec<Model> = serde_json::from_str(&json)?;
            let model = models.first();

            match model {
                Some(model) => {
                    let uuid = model.uuid;
                    trace!("Deleting model {uuid}", uuid = model.uuid);

                    let _ = Command::new(executable)
                        .arg("--tenant")
                        .arg(self.tenant.to_owned())
                        .arg("delete-model")
                        .arg("--uuid")
                        .arg(uuid.to_string())
                        .output()?;
                }
                None => warn!("Model not found!"),
            }

            Ok(())
        }
    }

    pub fn modify(&self, path: &Path) -> Result<(), SynchronizerError> {
        if !self.is_valid_path(path) {
            return Ok(());
        }

        self.delete(path)?;
        self.upload(path)
    }
}

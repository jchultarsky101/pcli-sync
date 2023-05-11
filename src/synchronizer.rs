use dirs::home_dir;
use log::{debug, error, trace};
use pcli::{configuration, service::Api};
use serde_json;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SynchronizerError {
    #[error("creation error")]
    UploadError,
    #[error("I/O error")]
    InputOutputError(#[from] std::io::Error),
    #[error("serialization error")]
    SerializationError(#[from] serde_json::Error),
    #[error("PCLI error")]
    PcliError(#[from] anyhow::Error),
    #[error("invalid home directory")]
    InvalidHomeDirectory,
}

#[derive(Debug)]
pub enum SynchronizerEvent {
    Create(PathBuf),
    Delete(PathBuf),
    Rename(PathBuf),
    //Modify(PathBuf),
}

pub struct Synchronizer {
    pub path: PathBuf,
    pub folder_id: u32,
    pub tenant: String,
    pub units: String,
    pub batch_uuid: Uuid,
    api: Api,
}

impl Synchronizer {
    fn init_api(tenant: String) -> Result<Api, SynchronizerError> {
        let home_directory = home_dir();
        let home_directory = match home_directory {
            Some(dir) => dir,
            None => return Err(SynchronizerError::InvalidHomeDirectory),
        };
        let home_directory = String::from(home_directory.to_str().unwrap());
        let mut default_configuration_file_path = home_directory;
        default_configuration_file_path.push_str("/.pcli.conf");

        let configuration =
            pcli::configuration::initialize(&String::from(default_configuration_file_path))?;
        let api_configuration = configuration::from_client_configuration(&configuration, &tenant)?;

        let api = Api::new(
            api_configuration.base_url,
            tenant.to_owned(),
            api_configuration.access_token,
        );

        Ok(api)
    }

    pub fn new(
        path: PathBuf,
        tenant: String,
        folder_id: u32,
        units: String,
    ) -> Result<Synchronizer, SynchronizerError> {
        Ok(Synchronizer {
            path,
            tenant: tenant.clone(),
            folder_id,
            units,
            batch_uuid: Uuid::new_v4(),
            api: Self::init_api(tenant)?,
        })
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

        if !output.status.success() {
            error!("Command executed with failing error code {}", output.status);
            Err(SynchronizerError::UploadError)
        } else {
            println!("Session initialized.");
            Ok(())
        }
    }

    fn is_valid_path(path: &Path) -> bool {
        let file_name = path.file_name();
        match file_name {
            Some(file_name) => {
                let file_name = file_name.to_str().unwrap();
                !file_name.starts_with(".") && file_name.ends_with(".STL")
            }
            None => false,
        }
    }

    pub fn on_event(&mut self, event: SynchronizerEvent) -> Result<(), SynchronizerError> {
        debug!("Event: {:?}", event);
        match event {
            SynchronizerEvent::Create(path) => self.upload(path.as_path()),
            SynchronizerEvent::Delete(path) => self.delete(path.as_path()),
            //SynchronizerEvent::Modify(path) => self.modify(path.as_path()),
            SynchronizerEvent::Rename(path) => self.rename(path.as_path()),
        }
    }

    fn rename(&mut self, path: &Path) -> Result<(), SynchronizerError> {
        self.upload(path)
    }

    fn upload(&mut self, path: &Path) -> Result<(), SynchronizerError> {
        let path_str = path.as_os_str().to_str().unwrap();
        if !Self::is_valid_path(path) {
            return Ok(());
        }

        debug!("Uploading: {}...", path_str);
        println!("Uploading: {}...", path_str.clone());

        let model = self.api.upload_file(
            self.folder_id,
            path_str.clone(),
            self.batch_uuid.clone(),
            &self.units,
            None,
        )?;

        // validate the upload
        match model {
            Some(model) => {
                trace!("Model uploaded with UUID of {}", model.uuid.to_string());
                let two_seconds = std::time::Duration::from_millis(2000);
                let five_seconds = std::time::Duration::from_millis(5000);
                let start_time = std::time::Instant::now();
                let timeout: u64 = 10000;
                let mut state = model.state.clone();

                std::thread::sleep(five_seconds);
                trace!(
                    "Checking the status for model {}...",
                    model.uuid.to_string()
                );
                while state.ne("finished") && state.ne("failed") && state.ne("missing-parts") {
                    let duration = start_time.elapsed().as_secs();
                    if duration >= timeout {
                        error!("Timeout wile validating model {}", model.uuid.to_string());
                        break;
                    }

                    match self.api.get_model(&model.uuid, false, false) {
                        Ok(verified_model) => {
                            state = verified_model.state.clone();
                        }
                        Err(_) => break,
                    }
                    std::thread::sleep(two_seconds);
                }
                println!("Uploaded model {}", model.uuid.to_string());
            }
            None => (),
        }

        Ok(())
    }

    fn delete(&self, path: &Path) -> Result<(), SynchronizerError> {
        let path_str = path.as_os_str().to_str().unwrap();
        if !Self::is_valid_path(path) {
            return Ok(());
        }

        debug!("Deleting: {}...", path_str);
        println!("Deleting: {}...", path_str.clone());

        let search = path.file_stem().unwrap().to_str().unwrap().to_string();
        trace!(
            "Searching for model by name \"{}\" in folder {}...",
            search.clone(),
            self.folder_id
        );
        let list_of_models =
            self.api
                .list_all_models(vec![self.folder_id], Some(&search), false)?;

        trace!("Found {} model(s)", list_of_models.models.len());
        for model in list_of_models.models {
            println!("Deleting {}...", model.uuid.to_string());
            self.api.delete_model(&model.uuid)?;
            println!("Deleted model {}", model.uuid.to_string());
        }

        Ok(())
    }
}

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SynchronizerError {
    #[error("deletion error")]
    DeletionError,
}

pub struct Synchronizer {
    folder_id: u32,
    tenant: String,
}

impl Synchronizer {
    pub fn new(tenant: String, folder_id: u32) -> Synchronizer {
        Synchronizer { tenant, folder_id }
    }

    pub fn delete(&self, path: &Path) -> Result<(), SynchronizerError> {
        println!("Deleting...");

        Ok(())
    }
}

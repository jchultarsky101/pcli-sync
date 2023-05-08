use clap::Parser;
use log::debug;
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind},
    Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::{Path, PathBuf};
use thiserror::Error;

mod synchronizer;
use synchronizer::{Synchronizer, SynchronizerError};

#[derive(Error, Debug)]
enum PcliSyncError {
    #[error("Invalid directory")]
    InvalidDirectory(String),
    #[error("Synchronization error")]
    SynchronizationError(#[from] SynchronizerError),
    #[error("Directory scan error")]
    NotifyError(#[from] notify::Error),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = Some("Physna file sync. Monitors a directory for changes and synchronizes the contents with Physna."))]
struct Args {
    /// Directory to monitor for changes
    #[arg(short, long)]
    directory: PathBuf,

    /// Physna tenant
    #[arg(short, long)]
    tenant: String,

    /// Physna folder ID
    #[arg(short, long)]
    folder_id: u32,

    /// Unit of measure
    #[arg(short, long)]
    units: String,
}

fn print_banner() {
    println!(
        "{}",
        r#"  _____   _____ _      _____    _____                  "#
    );
    println!(
        "{}",
        r#" |  __ \ / ____| |    |_   _|  / ____|                 "#
    );
    println!(
        "{}",
        r#" | |__) | |    | |      | |   | (___  _   _ _ __   ___ "#
    );
    println!(
        "{}",
        r#" |  ___/| |    | |      | |    \___ \| | | | '_ \ / __|"#
    );
    println!(
        "{}",
        r#" | |    | |____| |____ _| |_   ____) | |_| | | | | (__ "#
    );
    println!(
        "{}",
        r#" |_|     \_____|______|_____| |_____/ \__, |_| |_|\___|"#
    );
    println!(
        "{}",
        r#"                                       __/ |           "#
    );
    println!(
        "{}",
        r#"                                      |___/            "#
    );
    println!();
    println!("Version 0.1.0");
    println!("jchultarsky@physna.com");
    println!();
}

fn main() -> Result<(), PcliSyncError> {
    print_banner();

    let _log_init_result = pretty_env_logger::try_init_timed();

    let args = Args::parse();
    let directory = args.directory;
    let tenant = args.tenant;
    let folder_id = args.folder_id;
    let units = args.units;

    if !directory.is_dir() {
        return Err(PcliSyncError::InvalidDirectory(
            directory.into_os_string().into_string().unwrap(),
        ));
    }

    println!(
        "Watching directory {}... To exit, press Ctrl-C.",
        directory.clone().into_os_string().into_string().unwrap()
    );
    if let Err(e) = watch(directory.as_path(), tenant, folder_id, units) {
        println!("error: {:?}", e)
    }

    Ok(())
}

fn watch<P: AsRef<Path>>(
    path: P,
    tenant: String,
    folder_id: u32,
    units: String,
) -> Result<(), PcliSyncError> {
    let (tx, rx) = std::sync::mpsc::channel();

    let sync = Synchronizer::new(tenant, folder_id, units);
    sync.init()?;

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => match event.kind {
                EventKind::Create(kind) => match kind {
                    CreateKind::File => {
                        debug!("Create file detected");
                        for path in event.paths {
                            sync.upload(&path)?
                        }
                    }
                    _ => (),
                },
                EventKind::Modify(kind) => match kind {
                    ModifyKind::Data(_) => {
                        debug!("Modify data detected");
                        for path in event.paths {
                            sync.modify(&path)?
                        }
                    }
                    _ => (),
                },
                EventKind::Remove(kind) => match kind {
                    RemoveKind::File => {
                        debug!("Delete file detected");
                        for path in event.paths {
                            sync.delete(&path)?
                        }
                    }
                    _ => (),
                },
                _ => println!("Other"),
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

use clap::Parser;
use log::{error, trace};
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
    Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{path::PathBuf, sync::Arc};
use thiserror::Error;

mod synchronizer;
use synchronizer::{Synchronizer, SynchronizerError, SynchronizerEvent};

#[derive(Error, Debug)]
enum PcliSyncError {
    #[error("Invalid directory")]
    InvalidDirectory(String),
    #[error("Synchronization error")]
    SynchronizationError(#[from] SynchronizerError),
    #[error("Directory scan error")]
    NotifyError(#[from] notify::Error),
    #[error("Process was terminated")]
    Terminated,
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
    println!("jchultarsky@physna.com");
    println!();
}

fn main() -> Result<(), PcliSyncError> {
    print_banner();

    let _log_init_result = pretty_env_logger::try_init_timed();

    let args = Args::parse();
    let path = args.directory;
    let path2 = path.to_owned();
    let tenant = args.tenant;
    let folder_id = args.folder_id;
    let units = args.units;
    let path_str = path.clone().into_os_string().into_string().unwrap();

    if !path.clone().is_dir() {
        return Err(PcliSyncError::InvalidDirectory(path_str.clone()));
    }

    println!(
        "Watching directory {}... To exit, press Ctrl-C.",
        path_str.clone(),
    );

    // spawn the event listener thread
    let (sender, receiver) = std::sync::mpsc::channel::<SynchronizerEvent>();
    let handle = std::thread::spawn(move || -> Result<(), SynchronizerError> {
        let mut sync = Box::new(Synchronizer::new(
            path.to_owned(),
            tenant,
            folder_id,
            units,
        )?);
        sync.init()?;

        loop {
            let event = receiver.recv().unwrap();
            sync.on_event(event)?;

            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    // Start the directory watcher
    if let Err(e) = watch(Arc::new(Box::new(sender)), path2) {
        eprintln!("error: {:?}", e);
        ::std::process::exit(exitcode::DATAERR);
    }

    // Join with the main thread to finish all operations in projgress before exiting
    let _ = handle.join();

    Ok(())
}

fn watch<P: AsRef<Box<std::sync::mpsc::Sender<SynchronizerEvent>>>>(
    sender: P,
    path: PathBuf,
) -> Result<(), PcliSyncError> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_path(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => match event.kind {
                EventKind::Create(kind) => match kind {
                    CreateKind::File => {
                        trace!("Create action detected");
                        for path in event.paths {
                            let response = sender
                                .as_ref()
                                .send(SynchronizerEvent::Create(path.clone()));
                            match response {
                                Ok(()) => (),
                                Err(_) => return Err(PcliSyncError::Terminated),
                            }
                        }
                    }
                    _ => (),
                },
                EventKind::Modify(kind) => {
                    trace!("Modify kind is {:?}", kind);
                    match kind {
                        ModifyKind::Data(_) => {
                            trace!("Modify event detected");
                            /*
                            for path in event.paths {
                                sender
                                    .as_ref()
                                    .send(SynchronizerEvent::Modify(path.clone()))
                                    .unwrap();
                            }
                            */
                        }
                        ModifyKind::Name(rename) => match rename {
                            RenameMode::Any => {
                                println!("Rename detected");
                                for path in event.paths {
                                    let response = sender
                                        .as_ref()
                                        .send(SynchronizerEvent::Rename(path.clone()));
                                    match response {
                                        Ok(()) => (),
                                        Err(_) => return Err(PcliSyncError::Terminated),
                                    }
                                }
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                }
                EventKind::Remove(kind) => match kind {
                    RemoveKind::File => {
                        trace!("Delete action detected");
                        for path in event.paths {
                            let response = sender
                                .as_ref()
                                .send(SynchronizerEvent::Delete(path.clone()));
                            match response {
                                Ok(()) => (),
                                Err(_) => return Err(PcliSyncError::Terminated),
                            }
                        }
                    }
                    _ => (),
                },
                _ => println!("Detected unsupported action"),
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

extern crate clap;
extern crate log;

use std::fmt::Display;
use std::fs;
use std::os::unix::prelude::ExitStatusExt;
use std::process;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time;
use std::time::Duration;

use clap::Parser;
use env_logger::Env;
use futures::executor;

use log::error;
use log::info;
use log::warn;

trait UnwrapLog<T> {
    fn unwrap_log(self) -> T;
}

impl<T, E: Display> UnwrapLog<T> for Result<T, E> {
    fn unwrap_log(self) -> T {
        match self {
            Ok(result) => result,
            Err(err) => {
                error!("{}", err);
                process::exit(1);
            }
        }
    }
}

async fn start_watchdog(
    executable: &str, args: &[String], restart_delay: time::Duration, sigterm_recv: Receiver<bool>,
    stdin_file: Option<String>, stdout_file: Option<String>, stderr_file: Option<String>
) {
    let mut command = Command::new(&executable);

    while sigterm_recv.try_recv().or::<()>(Ok(true)).unwrap() {

        let mut proc = command.args(args);

        if let Some(file) = stdin_file.clone() {
            proc = proc.stdin(Stdio::from(fs::File::open(file).unwrap_log()));
        }
        if let Some(file) = stdout_file.clone() {
            proc = proc.stdout(Stdio::from(fs::File::create(file).unwrap_log()));
        }
        if let Some(file) = stderr_file.clone() {
            proc = proc.stderr(Stdio::from(fs::File::create(file).unwrap_log()));
        }

        let mut proc = proc.spawn().unwrap_log();
        info!("starting process {} {:?}", executable, args);
        let exit_status = proc.wait().unwrap_log();

        if exit_status.success() {
            info!("process {} exited normally", proc.id());
            info!("exiting watchdog");
            return;
        }

        match exit_status.code() {
            Some(exit_code) => warn!("process {} exited with code {}", proc.id(), exit_code),
            None => {
                let signal = exit_status.signal().unwrap();
                info!("process {} was terminated by signal {}", proc.id(), signal);
                return;
            }
        }

        if sigterm_recv.try_recv().is_err() {
            break;
        } else {
            thread::sleep(restart_delay);
            info!("restarting after delay");
        }
    }

    info!("watchdog received SIGTERM, exiting");
}

#[derive(Parser)]
#[clap(version)]
struct Args {
    /// Executable file path
    pub executable: String,

    /// Redirect stdin
    #[clap(short = 'i', long)]
    pub stdin: Option<String>,

    /// Redirect stdout
    #[clap(short = 'o', long)]
    pub stdout: Option<String>,

    /// Redirect stderr
    #[clap(short = 'e', long)]
    pub stderr: Option<String>,

    /// Restart delay (millionsecond)
    #[clap(short = 't', long, default_value = "1000")]
    pub delay: u64,

    /// List of arguments to pass to the executable,
    /// seperated by delimitator "--"
    pub args: Vec<String>,
}

fn main() {

    let args = Args::parse();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let (sender, receiver) = channel();

    ctrlc::set_handler(move || {
        sender.send(false).unwrap_log();
    }).is_err().then(|| {
        error!("unable to set signal handler");
    });

    let future = start_watchdog(
        &args.executable,
        &args.args[..],
        Duration::from_millis(args.delay),
        receiver,
        args.stdin,
        args.stdout,
        args.stderr,
    );

    executor::block_on(future);
}

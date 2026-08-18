mod client;
mod common;
mod server;
use std::fs::File;
use std::path::Path;
use std::io::ErrorKind;
use clap::{Parser, Subcommand, Args, ArgGroup};
use std::env;
use regex::Regex;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

const GDBORCH_VERSION: &str = env!("CARGO_PKG_VERSION");

fn set_log_level(level: &str) {
    unsafe { env::set_var("RUST_LOG", level) };
}

#[derive(Parser)]
#[command(
    name = "gdborch",
    version = GDBORCH_VERSION,
    about = "gdb-orchestrator - A modern tool for debugging multiple processes and its threads at the same time.",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run gdborch in client mode
    Client {
        #[command(subcommand)]
        command: ClientCommand,
    },
    /// Run gdborch in server mode
    Server {
        #[command(subcommand)]
        command: ServerCommand,
    }
}


#[derive(Subcommand)]
pub enum ClientCommand {
    /// Run client to manage local sessions
    Local {
        #[command(subcommand)]
        command: ClientLocalCommand,
    },
    /// Run client to manage remote sessions
    Remote {
        #[command(subcommand)]
        command: ClientRemoteCommand,
    }
}

#[derive(Subcommand)]
pub enum ClientLocalCommand {
    /// Start a local session
    Start(ClientLocalStartArgs),

    /// Stop local session(s)
    Stop(ClientLocalStopArgs),

    /// Show local session(s)
    Show(ClientLocalShowArgs),
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
#[command(
    group(
        ArgGroup::new("attach_method")
            .required(true)      // must provide one
            .multiple(false)     // only one allowed
            .args(&["proc_name", "pids"])
    )
)]
pub struct ClientLocalStartArgs {
    /// Verbose Flag
    #[arg(short = 'v', long = "verbose", default_value_t = false, required = false)]
    pub verbose: bool,

    /// Target process name
    #[arg(short = 'n', long = "proc_name", group = "attach_method")]
    pub proc_name: Option<String>,

    /// Target process IDs (comma-separated eg. 1234,5678)
    #[arg(short = 'p', long = "pids", group = "attach_method")]
    pub pids: Option<String>,

    /// Path to GDB script
    #[arg(short = 's', long = "gdb_script", required = true)]
    pub gdb_script: String,
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
#[command(
    group(
        ArgGroup::new("stop_method")
            .required(true)      // must provide one
            .multiple(false)     // only one allowed
            .args(&["session", "all"])
    )
)]
pub struct ClientLocalStopArgs {
    /// Verbose Flag
    #[arg(short = 'v', long = "verbose", default_value_t = false)]
    pub verbose: bool,

    /// Session identifier to stop (eg. 25-12-2023T14-30-00)
    #[arg(short = 's', long = "session", group = "stop_method")]
    pub session: Option<String>,

    /// Stop all sessions
    #[arg(short = 'a', long = "all", group = "stop_method")]
    pub all: bool,

}

#[derive(Args)]
pub struct ClientLocalShowArgs {
    /// Verbose Flag
    #[arg(short = 'v', long = "verbose", default_value_t = false, required = false)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum ClientRemoteCommand {
    /// Start remote client
    Start(ClientRemoteStartArgs),

    /// Stop remote client
    Stop(ClientRemoteStopArgs),

    /// Show sessions of local client
    Show(ClientRemoteShowArgs),
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct ClientRemoteStartArgs {
    /// Verbose Flag
    #[arg(short = 'v', long = "verbose", default_value_t = false, required = false)]
    pub verbose: bool,

    /// Target list of IP:PORT to connect to (eg. 192.168.1.100:5555,192.168.1.100:5556)
    #[arg(short = 'c', long = "connect", required = true)]
    pub connect: String,

    /// Path to GDB script
    #[arg(short = 's', long = "gdb_script", required = true)]
    pub gdb_script: String,
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
#[command(
    group(
        ArgGroup::new("stop_method")
            .required(true)      // must provide one
            .multiple(false)     // only one allowed
            .args(&["session", "all"])
    )
)]
pub struct ClientRemoteStopArgs {
    /// Verbose Flag
    #[arg(short = 'v', long = "verbose", default_value_t = false)]
    pub verbose: bool,

    /// Session identifier to stop (eg. 25-12-2023T14-30-00)
    #[arg(short = 's', long = "session", group = "stop_method")]
    pub session: Option<String>,

    /// Stop all sessions
    #[arg(short = 'a', long = "all", group = "stop_method")]
    pub all: bool,

}

#[derive(Args)]
pub struct ClientRemoteShowArgs {
    /// Verbose Flag
    #[arg(short = 'v', long = "verbose", default_value_t = false, required = false)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum ServerCommand {
    /// Start server
    Start(ServerStartArgs),

    /// Stop server
    Stop(ServerStopArgs),

    /// Show sessions of server
    Show(ServerShowArgs),
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
#[command(
    group(
        ArgGroup::new("attach_method")
            .required(true)      // must provide one
            .multiple(false)     // only one allowed
            .args(&["proc_name", "pids"])
    )
)]
pub struct ServerStartArgs {
    /// Verbose Flag
    #[arg(short = 'v', long = "verbose", default_value_t = false, required = false)]
    pub verbose: bool,

    /// Target process name
    #[arg(short = 'n', long = "proc_name", group = "attach_method")]
    pub proc_name: Option<String>,

    /// Target process IDs (comma-separated eg. 1234,5678)
    #[arg(short = 'p', long = "pids", group = "attach_method")]
    pub pids: Option<String>,

    /// Intial port for gdbserver to try listen on
    #[arg(short = 'l', long = "init_listen_port", default_value = "5555")]
    pub listen_port: u16,

}

#[derive(Args)]
#[command(arg_required_else_help = true)]
#[command(
    group(
        ArgGroup::new("stop_method")
            .required(true)      // must provide one
            .multiple(false)     // only one allowed
            .args(&["session", "all"])
    )
)]
pub struct ServerStopArgs {
    /// Verbose Flag
    #[arg(short = 'v', long = "verbose", default_value_t = false)]
    pub verbose: bool,

    /// Session identifier to stop
    #[arg(short = 's', long = "session", group = "stop_method")]
    pub session: Option<String>,

    /// Stop all sessions
    #[arg(short = 'a', long = "all", group = "stop_method")]
    pub all: bool,
}

#[derive(Args)]
pub struct ServerShowArgs {

    /// Verbose Flag
    #[arg(short = 'v', long = "verbose", default_value_t = false, required = false)]
    pub verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Client { command } => {
            match command {
                ClientCommand::Local { command } => {
                    match command {
                        ClientLocalCommand::Start(local_start_args) => {
                            // Set logging level
                            if local_start_args.verbose {
                                set_log_level("debug");
                            } else {
                                set_log_level("info");
                            }
                            pretty_env_logger::init();

                            if let Some(gdb_script) = Some(&local_start_args.gdb_script) {

                                match File::open(gdb_script) {
                                    Ok(_file) => {
                                        if let Some(proc_name) = local_start_args.proc_name {
                                            client::local::attach_proc_name(&proc_name, Path::new(&gdb_script));
                                        } else if let Some(pids) = local_start_args.pids {
                                            client::local::attach_pids(&pids, Path::new(&gdb_script));
                                        }
                                    },
                                    Err(error) => {
                                        if error.kind() == ErrorKind::NotFound {
                                            error!("File not found: {}", gdb_script);
                                            // Code to create the file would go here
                                        } else {
                                            error!("Error opening file \"{}\": {}", gdb_script, error);
                                        }
                                    }
                                }
                            }
                        }

                        ClientLocalCommand::Stop(local_stop_args) => {
                            // Set logging level
                            if local_stop_args.verbose {
                                set_log_level("debug");
                            } else {
                                set_log_level("info");
                            }
                            pretty_env_logger::init();

                            if local_stop_args.session.is_some() {
                                let re_session = Regex::new(r"^\d{2}-\d{2}-\d{4}T\d{2}-\d{2}-\d{2}$").unwrap();
                                let session = local_stop_args.session.unwrap();
                                if !re_session.is_match(&session) {
                                    error!("Invalid session format. Expected format: <DD>-<MM>-<YYYY>T<hh>-<mm>-<ss>");
                                    return;
                                }
                                client::local::stop_session(&session);
                            } else if local_stop_args.all {
                                client::local::stop_all_sessions();
                            }
                        }

                        ClientLocalCommand::Show(local_show_args) => {
                            // Set logging level
                            if local_show_args.verbose {
                                set_log_level("debug");
                            } else {
                                set_log_level("info");
                            }
                            pretty_env_logger::init();
                            client::local::show_all_sessions();
                        }
                    }
                }
                ClientCommand::Remote { command } => {
                    match command {
                        ClientRemoteCommand::Start(remote_start_args) => { 
                            // Set logging level
                            if remote_start_args.verbose {
                                set_log_level("debug");
                            } else {
                                set_log_level("info");
                            }
                            pretty_env_logger::init();
                            // check remote_start_args.connect format here
                            // It should be a comma separated list of IP:PORT
                            let re_ip_port = Regex::new(r"^((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\.){3}(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d):(6553[0-5]|655[0-2]\d|65[0-4]\d{2}|6[0-4]\d{3}|[1-5]?\d{1,4}|0)$").unwrap();
                            for ip_port in remote_start_args.connect.split(',') {
                                if !re_ip_port.is_match(ip_port) {
                                    error!("Invalid IP:PORT format: {}. Expected format: <IP>:<PORT>", ip_port);
                                    return;
                                }
                            }
                            client::remote::attach_remote(remote_start_args.connect.as_str(), Path::new(&remote_start_args.gdb_script));
                        }
                        ClientRemoteCommand::Stop(remote_stop_args) => { 
                            // Set logging level
                            if remote_stop_args.verbose {
                                set_log_level("debug");
                            } else {
                                set_log_level("info");
                            }
                            pretty_env_logger::init();
                            if remote_stop_args.session.is_some() {
                                let re_session = Regex::new(r"^\d{2}-\d{2}-\d{4}T\d{2}-\d{2}-\d{2}$").unwrap();
                                let session = remote_stop_args.session.unwrap();
                                if !re_session.is_match(&session) {
                                    error!("Invalid session format. Expected format: <DD>-<MM>-<YYYY>T<hh>-<mm>-<ss>");
                                    return;
                                }
                                client::remote::stop_session(&session);
                            } else if remote_stop_args.all {
                                client::remote::stop_all_sessions();
                            }
                        }
                        ClientRemoteCommand::Show(remote_show_args) => { 
                            // Set logging level
                            if remote_show_args.verbose {
                                set_log_level("debug");
                            } else {
                                set_log_level("info");
                            }
                            pretty_env_logger::init();
                            client::remote::show_all_sessions();
                        }
                    }
                }
            }
        }
        Commands::Server { command } => {
            match command {
                ServerCommand::Start(start_args) => {
                    // Implement server start logic here
                    if start_args.verbose {
                        set_log_level("debug");
                    } else {
                        set_log_level("info");
                    }
                    pretty_env_logger::init();
                    if let Some(proc_name) = start_args.proc_name {
                        server::attach_proc_name(Some(&proc_name), start_args.listen_port);
                    } else if let Some(pids) = start_args.pids {
                        server::attach_pids(Some(&pids), start_args.listen_port);
                    }

                }

                ServerCommand::Stop(stop_args) => {
                    if stop_args.verbose {
                        set_log_level("debug");
                    } else {
                        set_log_level("info");
                    }
                    pretty_env_logger::init();
                    if let Some(session) = stop_args.session {
                        let re_session = Regex::new(r"^\d{2}-\d{2}-\d{4}T\d{2}-\d{2}-\d{2}$").unwrap();
                        if !re_session.is_match(&session) {
                            error!("Invalid session format. Expected format: <DD>-<MM>-<YYYY>T<hh>-<mm>-<ss>");
                            return;
                        }
                        server::stop_session(&session);
                    } else if stop_args.all {
                        server::stop_all_sessions();
                    }
                }

                ServerCommand::Show(show_args) => {
                    // Set logging level
                    if show_args.verbose {
                        set_log_level("debug");
                    } else {
                        set_log_level("info");
                    }
                    pretty_env_logger::init();
                    server::show_all_sessions();
                }
            }
        }
    }
}


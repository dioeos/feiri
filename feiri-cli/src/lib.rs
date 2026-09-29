use feiri_ipc::{Action, Query};

#[derive(clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Sub>
}

#[derive(clap::Subcommand, Clone)]
pub enum Sub {
    Msg {
        #[command(subcommand)]
        msg: Msg
    }
}


#[derive(clap::Subcommand, Clone, Debug)]
pub enum Msg {
    Action {
        #[command(subcommand)]
        action: Action,
    },
    Query {
        #[command(subcommand)]
        query: Query
    }
}

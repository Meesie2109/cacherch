use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cacherch",
    version,
    about = "Mini search engine with Redis caching"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn cmd(&self) -> &Commands {
        &self.command
    }
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    Index {
        path: String,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = 30)]
        ttl: usize,
    },
}

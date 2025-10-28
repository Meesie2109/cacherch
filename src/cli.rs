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
        #[arg(long, default_value_t = false)]
        flush_cache: bool,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = 30)]
        ttl: usize,
        #[arg(long, default_value_t = false)]
        flush_cache: bool,
    },
}

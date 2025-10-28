use cacherch::cli::{Cli, Commands};
use cacherch::errors::CacherchError;
use cacherch::indexer::index_dir;
use cacherch::searcher::search_query;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), CacherchError> {
    let cli = Cli::parse();

    match cli.cmd() {
        Commands::Index { path } => index_dir(&path)?,
        Commands::Search { query, ttl } => search_query(&query, &ttl).await?,
    }

    Ok(())
}

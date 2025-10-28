use cacherch::cli::{Cli, Commands};
use cacherch::errors::CacherchError;
use cacherch::indexer::index_dir;
use cacherch::searcher::search_query;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), CacherchError> {
    let cli = Cli::parse();

    match cli.cmd() {
        Commands::Index { path, flush_cache } => index_dir(&path, flush_cache).await?,
        Commands::Search {
            query,
            ttl,
            flush_cache,
        } => search_query(&query, &ttl, &flush_cache).await?,
    }

    Ok(())
}

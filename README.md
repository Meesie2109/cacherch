# cacherch

A small school project featuring a CLI-based search engine. It allows you to add documents (TXT and PDF) and quickly search for content within them. It also caches search results in Redis for faster repeated queries.

## Features

- Index directories recursively (supports `.txt` and `.pdf` files)
- Full-text search with Tantivy
- Redis caching for search results (30-second cache by default)
- CLI interface powered by `clap`
- Pretty logging and error handling

## Installation

1. **Clone the repository:**

   ```bash
    git clone https://github.com/yourusername/cacherch.git
    cd cacherch
   ```

2. Install Rust (if not installed):

   ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Build the project:

   ```bash
    cargo build --release
   ```

4. Make sure Redis is installed and running on your machine (default URL: `redis://127.0.0.1/`)

## Usage

The CLI has two main commands: `index` and `search`.

### Index a directory

Recursively index all supported files in a directory:

```bash
cargo run --release -- index /path/to/documents
```

- Supported file types: .txt, .pdf

### Search for a query

Search for text across all indexed documents:

```bash
cargo run --release -- search "your query here"
```

- Returns the top 5 results by default.
- Uses Redis cache to speed up repeated queries.

Example output:

```bash
[Cache Miss]: Running Tantivy search...
Top 3 results for query 'rust programming':
1. intro.txt (0.87) - /path/to/documents/intro.txt
2. tutorial.pdf (0.65) - /path/to/documents/tutorial.pdf
[Cached results for the coming 30 seconds]
```

## Notes

- If you search again within 30 seconds, results are served from Redis:

  ```bash
  [Cache Hit]
  ```

- The index is stored in ./index by default. You can delete or move it to re-index documents.

## Info

Project: Could be used inside our backend for the project but that backend is currently out of scope the reason i have build this POC
Subject: Improve preformance and optimise workflows under heavy load including APIs and databases
Others: -
Source: (See bottom of the document)
Techniolgies: Rust, Redis, Cargo, Tokio and Tantivy

## Caching

Caching is the process of temporarily storing copies of data in a location that can be accessed faster than the original source.

In computing, caches act as a shortcut between the user and a slower data source (like a disk, a database, or a web API). By storing recently used or frequently requested data in memory, future requests can be served instantly improving speed, reducing server load, and enhancing the overall user experience.
For example:

- Your web browser caches images and scripts so websites load faster the next time.
- A search engine caches query results so repeated searches return instantly.
- In this project, Redis caches search results so the app doesn’t have to run the same text indexing query repeatedly.

### Common types of caching

| Type                    | Description                                         | Example                                         |
| ----------------------- | --------------------------------------------------- | ----------------------------------------------- |
| **Memory Caching**      | Stores data in RAM for fast access                  | Redis, Memcached                                |
| **Disk Caching**        | Saves files or data to disk to avoid re-downloading | Browser cache, OS file cache                    |
| **Application caching** | App level storage for computed or API data          | Django/Flask caching, Rust programs using redis |
| **Distributed Caching** | Shares cache across multiple servers                | Redis Cluster, Hazelcast                        |

### Caching strategies

| Strategy                       | Description                                                                                                                         |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------- |
| **Read-through caching**       | The app checks the cache first. If data is missing, it fetches from the source, then stores it in the cache (used in this project). |
| **Write-through cache**        | Data is written to both the cache and database simultaneously.                                                                      |
| **Cache-aside (lazy loading)** | The app manually controls what gets cached and when it expires.                                                                     |
| **TTL-based cache**            | Cached entries expire automatically after a set time (your app uses this with Redis SETEX).                                         |

### Performance insight

Accessing RAM is roughly 100,000 times faster than accessing a traditional hard drive.
Because of this, caching can reduce response times from milliseconds to microseconds.
In large-scale systems, such as search engines or social media platforms, this difference allows servers to handle millions of requests per second efficiently.

In this project, using Redis as a cache means that once a search query has been performed, the same query can be answered directly from memory without reloading or re-indexing documents from disk. This demonstrates a real-world application of caching to optimize performance.

### Cache invalidation

While caching improves performance, one of the hardest challenges in computer science is cache invalidation keeping cached data consistent with the original source.

If data changes in the underlying database or index, but the cache still holds the old value, users may see stale (outdated) information.

Common strategies to handle this include:

- Time-to-live (TTL): Cached data automatically expires after a set duration (used in your Redis cache).
- Manual invalidation: The application explicitly clears the cache when important updates occur.
- Version-based caching: Each cache entry includes a version number that changes when the data updates.

This implementation uses TTL to ensure that cached search results are automatically refreshed after 30 seconds, balancing freshness and speed.

## Redis

Redis is an in-memory, multi-second database first released in 2009. It was built around the idea that a cache can also act as a durable datastore. Around this time, large web applications such as Twitter were experiencing explosive growth and needed a faster way to deliver data to their end users.

This is where Redis short for `Remote Dictionary Server` came in.

Redis was a game-changing system because it stored data in RAM instead of on disk, allowing read and write operations to happen orders of magnitude faster than traditional databases. Despite being memory-based, Redis can also persist data to disk, which means cached data isn’t always lost when the system restarts.

This hybrid model of speed + durability made Redis one of the most popular technologies for caching, session management, message queues, and even as a lightweight primary database.

## Sources

- What is caching [**LINK**](https://hazelcast.com/foundations/caching/caching/)
- Redis persistence & TTL tutorial [**LINK**](https://redis.io/docs/latest/develop/data-types/strings/#expiration)
- Redis in 100 seconds [**LINK**](https://www.youtube.com/watch?v=G1rOthIU-uo)
- What is redis IBM [**LINK**](https://www.ibm.com/think/topics/redis)

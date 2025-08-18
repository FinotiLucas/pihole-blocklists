# Pihole Blocklist api

A simple Rust-based blocklist api.\
It checks and maintains a list of domains and IPs that should be
blocked.

## Features

-   Fetches blocklists from multiple sources\
-   Stores them locally\
-   Provides a simple HTTP server to serve the blocklist\
-   Can be easily deployed on any Linux server

## How to Use

1.  Clone this repository:

    ``` bash
    git clone https://github.com/finotilucas/pihole-blocklist.git
    cd pihole-blocklist
    ```

2.  Build the project:

    ``` bash
    cargo build --release
    ```

3.  Run the api:

    ``` bash
    ./target/release/pihole-blocklist
    ```

## License

MIT License

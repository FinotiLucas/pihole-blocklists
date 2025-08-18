# Pihole Blocklist API

A simple Rust-based blocklist API.\
It checks and maintains a list of domains and IPs that should be blocked.

## Features

- Fetches blocklists from multiple sources\
- Stores them locally\
- Provides a simple HTTP server to serve the blocklist\
- Can be easily deployed on any Linux server, including ARM devices like Orange Pi Zero 3
- Can be run automatically at system boot using systemd

## How to Use

### 1. Clone this repository:

```bash
git clone https://github.com/finotilucas/pihole-blocklist.git
cd pihole-blocklist
```

### 2. Build the project for your system

- For native Linux (x86_64):

```bash
cargo build --release
```

- For ARM64:

```bash
rustup target add aarch64-unknown-linux-gnu
sudo apt update && sudo apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

### 3. Run the API manually

- Native Linux:

```bash
./target/release/pihole-blocklist
```

- ARM64:

```bash
./target/aarch64-unknown-linux-gnu/release/pihole-blocklist
```

### 4. Run automatically via systemd (recommended for servers)

1. Run the script in the root of the project `run.sh`:

```bash
chmod +x run.sh
```

2. Create the systemd service:

```bash
sudo nano /etc/systemd/system/pihole-blocklists.service
```

3. Paste the following:

```ini
[Unit]
Description=Pi-hole Blocklists Rust Service
After=network.target

[Service]
Type=simple
User=orangepi
WorkingDirectory=/home/orangepi/Github/pihole-blocklists-rs
ExecStart=/home/orangepi/Github/pihole-blocklists-rs/run.sh
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

4. Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable pihole-blocklists
sudo systemctl start pihole-blocklists
```

5. Check the status:

```bash
sudo systemctl status pihole-blocklists
```

Logs can be monitored with:

```bash
journalctl -u pihole-blocklists -f
```

## License

MIT License


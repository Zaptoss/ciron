# Ciron - Process Manager Daemon

A lightweight process manager daemon written in Rust. Ciron allows you to manage, monitor, and automatically restart processes.

Inspired by [Supervisor](https://supervisord.org/) and designed for cloud-native environments with support for microVMs and vsock communication.

## Use Cases

While it may seem counterintuitive to run multiple processes in a single container (containers are typically designed for one process), Ciron excels in specific scenarios:

- **CTF Competitions**: Package complete challenge environments (web server + database + services) in a single container
- **MicroVMs**: Manage multiple services in lightweight VMs like Firecracker with native vsock support for minimal overhead
- **Legacy Applications**: Run traditional applications that bundle multiple services (e.g., Apache + PHP-FPM)

## Components

- **cirond**: The daemon that manages processes
- **cironctl**: CLI client to control the daemon
- **ciron-common**: Shared library with gRPC definitions and utilities

## Building

```bash
cargo build --release
```

## Running

Start the daemon:
```bash
./target/release/cirond -c ciron.toml
```

Control processes:
```bash
./target/release/cironctl -t unix:///tmp/cirond.sock status
./target/release/cironctl -t inet://127.0.0.1:50051 start web
./target/release/cironctl -t vsock://2:50051 stop sleep
```

## Running in Docker

Build the Docker image:
```bash
docker build -t cirond:latest .
```

Run the daemon in a container:
```bash
docker run --rm -p 50051:50051 \
  -v $(pwd)/ciron.docker.toml:/etc/ciron/ciron.toml \
  -v $(pwd)/loop_app.sh:/opt/loop_app.sh \
  cirond:latest
```

Control processes from host:
```bash
./target/debug/cironctl -t inet://127.0.0.1:50051 status
```

## Examples

See the `examples/` directory for complete working examples:

- **[basic](examples/basic/)** - Simple shell commands to get started
- **[nginx-webapp](examples/nginx-webapp/)** - Multi-process setup with Nginx and Python web application

## TODO

- [ ] Add comprehensive logging with log rotation
- [ ] Implement process resource limits (CPU, memory)
- [ ] Add process dependency management
- [ ] Implement health checks for monitored processes
- [ ] Support for process groups
- [ ] Signal handling for graceful shutdown
- [ ] Process output capture and log management
- [ ] Web UI for process monitoring
- [ ] `systemd` integration
- [ ] Configuration hot-reload
- [ ] Add unit and integration tests
- [ ] Create Firecracker/microVM testing environment for vsock
- [ ] Documentation improvements and examples

## Configuration Example
```
log_level = "info"

# Transport configuration
[transport]
enable_unix = true
# Unix socket path (default: /tmp/cirond.sock)
unix_socket_path = "/tmp/cirond.sock"
# Enable inet socket (default: false)
enable_inet = false
# Inet address (default: 127.0.0.1:50051)
inet_address = "127.0.0.1:50051"
# Enable vsock (default: false, Linux only)
enable_vsock = false
# Vsock CID (Context ID) - use 2 for host, or specific VM CID
vsock_cid = 2
# Vsock port
vsock_port = 50051

[program.web]
command = "./webapp"
autostart = true
restart = "always"

[program.web.env]
PORT = "8080"
DEBUG = "true"

[program.sleep]
command = "sleep 5"
autostart = true
restart = "on-failure"
```




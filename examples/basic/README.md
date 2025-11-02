# Basic Example

This example demonstrates basic usage of Ciron with shell commands.

## Running

1. Start the daemon:
```bash
../../target/release/cirond -c ciron.toml
```

2. Check status:
```bash
../../target/release/cironctl -t unix:///tmp/cirond.sock status
```

You should see:
```
Process: hello - Status: RUNNING
Process: counter - Status: RUNNING
```

3. Control processes:
```bash
# Stop a process
../../target/release/cironctl -t unix:///tmp/cirond.sock stop hello

# Check status again
../../target/release/cironctl -t unix:///tmp/cirond.sock status

# Start a process
../../target/release/cironctl -t unix:///tmp/cirond.sock start hello

# Restart a process
../../target/release/cironctl -t unix:///tmp/cirond.sock restart counter
```

## What's Happening

- **hello**: Prints "Hello from Ciron" with timestamp every 2 seconds
- **counter**: Counts from 0 upwards, printing every 3 seconds
- Both processes use shell commands with proper quote handling
- `restart = "always"` means the process will always restart when it exits
- `restart = "on-failure"` means it only restarts if it exits with non-zero code

## Notes

Commands are parsed with proper shell quote handling, so you can use:
- Single quotes: `'text'`
- Double quotes: `"text"`
- Shell variables: `$var` or `$(command)`
- Complex shell syntax

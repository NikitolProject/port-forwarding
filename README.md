# Simple Port Forwarding

### Client and server parts to implement port forwarding between a local server that does not have a white IP address and a remote server that has one

### This implementation may not be the most efficient, so we suggest looking at other similar programs before using this one
---

## Installation
First, you need to install the Rust programming language on your computer in advance

Next, to install the server and client parts, you need to write the following command in each of the folders:

```bash
cargo build
```

Rust will pull all the necessary dependencies by itself, you can find the compiled executables in the target/debug/ directories of each folder

---
## Startup
To run the client/server part you need to run the compiled executable. You can find out the details of possible configuration by writing the -h flag after ./<executable_name>. Also, if you want to see logs during program execution, enter RUST_LOG=<executable_name> before typing ./<executable_name>.
Example:

```bash
RUST_LOG=tcp_repeater_server ./tcp_repeater_server
```
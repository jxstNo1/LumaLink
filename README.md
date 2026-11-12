<p align="center">
  <img src="docs/images/lumalink-logo.png" alt="LumaLink" width="560">
</p>

<h1 align="center">LumaLink</h1>

<p align="center"><strong>Your controller. Your network. Your games.</strong></p>

<p align="center">
  <img src="https://img.shields.io/badge/language-Rust-DEA584?logo=rust&amp;logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/status-early_development-8B5CF6" alt="Early development">
  <img src="https://img.shields.io/badge/server_target-Linux-22D3EE?logo=linux&amp;logoColor=white" alt="Server target: Linux">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT; see NOTICE for exceptions"></a>
</p>

**LumaLink** is a standalone Rust reimplementation of the local
[Stadiacast](https://github.com/jkfran/stadiacast) receiver for the Stadia
Controller's **Wi-Fi mode**. The project was previously called **Stadiacast Rust**.

The goal is to keep Stadia Controllers working on your own network and assign
them to individual gaming PCs through a central server. A small client on each
target PC will expose assigned controllers as virtual gamepads and return
in-game rumble to the controller.

> **Development status:** LumaLink is not yet a working Rust receiver.
> Only the WireHeader2 protocol component has been implemented. Wi-Fi
> connectivity, the server, PC clients, virtual gamepad output, rumble, and
> audio are still planned for the Rust implementation. The existing Python
> receiver remains the reference for protocol compatibility and behavior.

## Project goals

- **Local operation:** Handle controller connections and input on your own network.
- **Standalone Rust implementation:** Run the server and PC clients without an embedded Python receiver or Python subprocesses.
- **Linux as the primary server platform:** Run as a persistent service without a desktop or browser.
- **Multiple controllers and PCs:** Explicitly assign each controller to a target PC.
- **Input and feedback:** Forward buttons, sticks, and triggers to the PC and send rumble back.
- **Reliable connection changes:** Release held inputs when disconnecting or switching target PCs.
- **Wi-Fi firmware compatibility:** Work with the controller's existing network protocol.

These are development goals. They do not imply that all features or client
operating systems are currently supported.

## Current implementation

| Component | Rust implementation status |
| --- | --- |
| WireHeader2 | Packing and parsing implemented; payloads are borrowed from the input buffer without copying |
| Protocol tests | Test cases cover fixed records, malformed lengths, reserved bits, message IDs, and length boundaries |
| RTP, STUN, and input decoding | Planned |
| Receiver CLI, DNS/SNTP, and HTTP/HTTPS | Planned |
| Discovery, signaling, and ICE/DTLS | Planned |
| Controller sessions and reconnection | Planned |
| Paired PC clients and controller assignment | Planned |
| Virtual gamepads and rumble | Planned; operating system backends will be evaluated separately |
| BLE provisioning and headset audio | Scheduled for later stages |

`stadiacast-protocol` currently has no external dependencies. The existing
crate name remains in the source code; **LumaLink** is the new project name.

## Getting started for developers

Rust development currently takes place on the **`rust-port`** branch. You need
Git and a Rust toolchain with Cargo, rustfmt, and Clippy. The workspace does
not yet specify a minimum supported Rust version.

```sh
git clone --branch rust-port https://github.com/jxstNo1/LumaLink.git
cd LumaLink/rust

cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

These commands check the existing protocol component. They do not start a
receiver or connect a controller. The included protocol tests do not require
controller hardware.

There is currently no documented LumaLink launch command or ready-to-use Rust
server package. The upstream project's `pip install stadiacast` instructions
install its Python implementation.

## Planned deployment model

Controllers connect to the local network over Wi-Fi. The central LumaLink
server will manage their sessions and forward input to each assigned PC
client. Games run on the target PCs, where the client creates the virtual
gamepad.

PC clients must be explicitly paired and authenticated. Each controller has
at most one active target PC. If a connection is lost, the client must
independently release buttons and triggers and center the sticks.

See the [server design](docs/rust-server.md) for details. The transport between
the server and PC clients, along with supported client operating systems,
has not yet been finalized.

## Roadmap

1. Port additional protocol components using shared reference fixtures and malformed-input tests.
2. Build a Linux server with configuration, logging, and graceful shutdown.
3. Implement the boot flow, discovery, signaling, and encrypted controller connections.
4. Validate real input, reconnection, and multiple controller sessions.
5. Add paired PC clients, explicit assignment, virtual gamepads, and rumble.
6. Expand Linux deployment, BLE provisioning, and headset audio support.

TLS/DTLS will use maintained libraries chosen to match the controller's
specific protocol requirements.

## Testing and performance

The Python implementation and its fake controller serve as references for
compatibility testing. Real-hardware test reports must identify the firmware
version, host system, and features actually tested.

Rust alone does not guarantee lower latency. Comparisons should use the same
conditions and record p50/p95/p99 processing times, jitter, drops, CPU usage,
and reconnection behavior. For the server-client model, measurements must
include the additional network hop. Physical button-to-screen latency must
be measured separately.

## Documentation and contributing

- [Rust port: implementation status and migration plan](rust/README.md)
- [Server architecture and deployment model](docs/rust-server.md)
- [Python reference protocol documentation](docs/protocol.md)
- [Python reference architecture](docs/architecture.md)
- [Python reference setup](docs/setup.md)
- [Contributing guidelines](CONTRIBUTING.md)
- [Upstream security information](SECURITY.md)
- [Report a bug or suggest a feature](https://github.com/jxstNo1/LumaLink/issues)

Contributions to protocol tests, Linux support, and documented hardware
testing are welcome. Please state whether bug reports concern the Rust port
or the Python reference. Do not include credentials or personal packet
captures in public issues.

## Upstream and license

LumaLink builds on the protocol work of
[Francisco Jimenez Cabrera and the Stadiacast contributors](https://github.com/jkfran/stadiacast).
The Rust port starts from upstream commit
`43cb9cd82a7b89ff0d384bf3472c16cb45bfe227`. Original copyright notices
are preserved.

The code is released under the [MIT License](LICENSE). Schemas recovered
from the controller firmware are subject to the exceptions described in
[NOTICE](NOTICE); translating them into Rust does not change their provenance.

LumaLink is an independent interoperability project and is not affiliated
with Google. Stadia is a trademark of Google LLC. Controller firmware is
not distributed with this project; the planned receiver speaks the existing
network protocol. A controller must be running its Wi-Fi firmware. LumaLink
is not a firmware flashing tool.

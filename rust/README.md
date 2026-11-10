# Stadiacast Rust port

This branch starts an incremental Rust port of the PC-side Stadiacast receiver.
It is not controller firmware, a working Rust receiver, or a demonstrated latency improvement.
The Python receiver remains the interoperability reference.
The deliverable is a standalone Rust implementation: the server and PC clients
must not invoke or embed the Python receiver at runtime. Python is only a reference
and, during development, an optional integration-test peer.

Linux is the primary supported target. The long-term product is a headless,
long-running server, with output adapters separated from controller sessions.
It routes controllers to individual PCs on the local network; a client on each PC
provides virtual gamepad output and returns rumble feedback.
See [server architecture](../docs/rust-server.md) for the intended deployment model.

## Current implementation

`stadiacast-protocol` implements WireHeader2 packing and borrowed-payload parsing,
adapted from `src/stadiacast/rtc/wire.py`. It has no external dependencies.
Its tests cover fixed records, malformed lengths, reserved bits, all 16-bit message ids,
the 24-bit length boundary, and borrowing the input buffer.

```sh
cd rust
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

The initial code was prepared without a Rust toolchain available on the authoring
machine. Compilation, formatting, and tests must be run before merging it.

## Migration order

1. Establish byte-for-byte parity with Python for WireHeader2, RTP, STUN, and input decoding.
   Add shared fixtures and malformed-input coverage; then fuzz the Rust parsers.
2. Add a receiver CLI with loopback-only lab defaults, DNS/SNTP, boot HTTP/HTTPS,
   discovery, GMsg, and protobuf signaling.
3. Implement ICE/DTLS using maintained protocol libraries; validate the specific
   controller profile before choosing dependencies. Do not write custom cryptography.
4. Implement the display state machine and a Linux-first headless server with a
   well-defined output-adapter interface. Add paired PC clients, explicit controller
   assignments, virtual gamepad backends, and safe neutralization on disconnect.
5. Add the game session, rumble/watchdog, multiple controllers, BLE provisioning,
   and finally SRTP/headset audio.
6. Exercise the port against the existing Python fake controller and then real hardware.
   Document firmware version and feature parity for each real-device run.

## Performance acceptance

Measure before changing defaults. Compare Python and Rust on the same Linux machine, network,
controller firmware, and output backend. Record p50/p95/p99 receive-to-gamepad-submit
time, jitter, drops, CPU, and reconnect behavior. Host timestamps do not establish
physical button-to-screen latency; measure that separately with suitable equipment.
Use bounded queues, retain button/trigger transitions, and keep logs/disk I/O out of
the input path. A faster parser does not by itself prove lower controller latency.

## Upstream and attribution

Starting reference: `jkfran/stadiacast` at
`43cb9cd82a7b89ff0d384bf3472c16cb45bfe227`.
Keep `LICENSE`, `NOTICE`, upstream history, and the original attribution.
The firmware-derived schemas listed in `NOTICE` are excluded from upstream's MIT
license; translating their representation does not change their provenance.
No firmware dumps, private keys, or personal packet captures belong in this port.

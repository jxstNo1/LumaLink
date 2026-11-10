# Rust server direction

## Requirements

- Rewrite the server and PC clients in Rust, with no Python runtime dependency,
  embedded Python receiver, or Python subprocess wrapper in the final product.
- Linux is the primary platform for builds, tests, deployment, and performance work.
- Run as a persistent, headless server; no desktop session, browser, or GUI is required.
- A central server on the local network assigns controllers to individual PCs.
- A small agent on each target PC exposes assigned controllers to that PC's games.
- Support multiple controller sessions, reconnects, bounded resource usage, and graceful shutdown.
- Retain protocol compatibility with the original WLAN firmware.
- This document describes the target design. Only WireHeader2 has been ported so far.

## Boundaries

```text
Stadia controllers -- WLAN --> Rust server
                                |-- controller sessions / protocol state
                                |-- input + rumble routing
                                |-- assignments / ownership
                                |-- remote output adapter --> PC client --> virtual pad --> games
                                |<---- rumble feedback <----- PC client <-- game
                                `-- administration / health / metrics
```

The remote adapter and PC client are planned components, not implemented endpoints.
Games run on the assigned PCs; the central server handles controller connectivity and routing.
The client operating systems have not yet been specified. Keep their virtual-device
backends behind an interface (Linux `uinput`, and a separately evaluated Windows backend).
Local game output on the server is optional and secondary.

## Assignment and client protocol

Each controller has at most one active target PC. Authenticate and explicitly pair PC
clients before exposing controllers to them. Clients advertise available gamepad slots;
the server routes input only to the selected owner and accepts rumble only from that owner.
Maintain assignment-generation and sequence numbers so late packets from an old assignment
cannot change the new target's state. Define timestamp semantics without assuming the
server and clients have synchronized clocks.

On reassignment, first release all buttons/triggers and center sticks on the old PC;
then initialize the new owner's state. On a lost connection, the client independently
neutralizes the virtual pad after a bounded timeout and the server stops pending rumble.
Specify reconnect behavior explicitly. Do not silently assign a controller to another PC.

Keep input forwarding independent of administration and persistent storage. Evaluate
an authenticated transport for latency, loss recovery, and head-of-line blocking before
choosing it. Benchmarks must include the additional server-to-PC hop.

## Implementation sequence

1. Protocol crate with byte fixtures and malformed-input tests, shared with the Python reference.
2. Linux server binary, configuration, structured logs, controlled shutdown, and lab loopback listeners.
3. Boot/discovery, ICE/DTLS, real input, session recovery, and multiple controllers.
4. A paired PC client, controller-to-PC assignment, virtual gamepad output, and rumble return path.
   Validate reassignment and client-side neutralization on disconnect before multi-PC use.
5. Linux deployment packaging and operational tests; then provisioning and headset audio.

Choose networking and cryptography dependencies after checking support for the controller's
specific protocol requirements. Reuse maintained TLS/DTLS implementations.

## Linux operations

Plan a dedicated unprivileged service account and systemd unit. Controller discovery requires
specific DNS and service ports; grant only the necessary bind capability when privileged ports
are used. Configure the controller network to use this service's DNS deliberately. Do not
replace a machine's existing resolver or change its routing implicitly.

For local game output, grant access to `/dev/uinput` only to the component that needs it.
The central protocol service should not require broad device access. A separate output process
is an option if privilege separation is needed.

Put persistent configuration under `/etc/stadiacast`, runtime state under `/var/lib/stadiacast`,
and logs on stdout/stderr for journald. These are proposed package defaults, not paths created
by the current code. Support explicit paths for development and containers.

Keep administration on loopback or a Unix socket by default. Remote clients require explicit
pairing/authentication, encrypted transport, controller ownership rules, and a disconnect policy
that releases all held inputs. Never expose the upstream unauthenticated control API to the
internet as the server's public API. Keep metrics and detailed logs out of the input hot path.

Container support is a later deployment option, not a requirement for running the server.
Document DNS, UDP, low-port binding, and optional uinput requirements rather than requiring a
privileged container. Test systemd deployment on a clean Linux host before calling it supported.

## Performance and validation

Start with Linux CI for formatting, clippy, tests, and release builds. Run the protocol tests
without hardware and use the Python fake controller for integration parity. Add real-controller
tests on the target Linux host, covering reconnects, stuck-input prevention, and long sessions.

Compare p50/p95/p99 host processing time and jitter against Python under the same conditions.
For remote output, include the server-to-client hop. Keep physical input-to-screen latency
separate from software timestamps. Rust and an extra server hop do not guarantee a speedup.

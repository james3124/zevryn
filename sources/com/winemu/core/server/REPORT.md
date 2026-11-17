# Report — `com.winemu.core.server` portable Rust model

Scope: dependency-free (std only) Rust modeling of the **portable logic** from
GameHub Lite v5.1.8's `com.winemu.core.server` package. Sockets / unix sockets,
epoll, real IPC, native servers and the `winemu` JNI library are **out of scope**
(see bottom).

## Files written (write area only)

| File | Purpose |
| --- | --- |
| `mod.rs` | Module root; declares submodules and re-exports all public types. |
| `request_codes.rs` | `RequestCode` enums per subsystem with exact i32 codes + `from_code`/`code`. |
| `server_message.rs` | `ServerMessage` (code + payload bytes) with code-byte-first encode/decode. |
| `plugin.rs` | `PluginState`, `PluginKind`, `ProgramPlugin` descriptor, ordered startup list, socket paths. |
| `winmonitor.rs` | WinMonitor data beans as plain owned structs. |
| `REPORT.md` | This report. |

## Exact RequestCodes sources

In the decompiled reference every `RequestCodes.java` is an **empty abstract
class** — the `static final int` constants were inlined by javac, so literal
values were recovered from the request handlers, which are listed as the
authoritative source for each enum.

| Enum (Rust) | Exact source file(s) the codes were recovered from | Codes |
| --- | --- | --- |
| `AlsaRequestCode` | `alsaserver/ALSARequestHandler.java` (`switch(e())` cases 0–7); operations in `alsaserver/ALSAClient.java` | 0 Reset, 1 Play, 2 Stop, 3 Pause, 4 Setup, 5 Write, 6 Flush, 7 GetPosition |
| `WmRequestCode` | `wm/WMRequestHandler.java` (`e() == 1`); callback in `wm/WMCallback.java` | 1 FireCallback (only handled code) |
| `PerfRequestCode` | `perf/PerfRequestHandler.java` (`e() == 10 / == 11`); listener in `perf/PerfEventListener.java` | 10 ReportMetrics, 11 UpdateValue |
| `RenderDocRequestCode` | `renderdoc/RenderDocRequestHandler.java` (`e() != 10` early return; responds `i(0)`) | 10 Ping |
| `SysvRequestCode` | `sysvshm/SysVSHMRequestHandler.java` (`bE == 0/1/2/3`); operations in `sysvshm/SysVSharedMemory.java` | 0 Create, 1 GetFd, 2 Delete, 3 GetSize |
| `WinMonitorRequestCode` | `winmonitor/WinMonitorClient.java`, `winmonitor/CommandRequest.java`, response beans | **No numeric codes exist** — commands are named strings serialized as JSON over UDP; variants are descriptive labels inferred from the response beans, not source constants |

The empty-inlined `RequestCodes.java` files that were checked (all confirmed
empty `abstract class`):
`alsaserver/RequestCodes.java`, `wm/RequestCodes.java`, `perf/RequestCodes.java`,
`renderdoc/RequestCodes.java`, `sysvshm/RequestCodes.java`.

## Framing decision (`server_message.rs`)

The request handlers read a **single code byte first** (`SocketInputStream.e()`)
and then subsystem-specific payload; guards like `socketInputStream.a() < 5`
require at least 5 buffered bytes before parsing (code + a 4-byte int in some
cases). There is **no whole-message length-prefix framing** in the source.
Per the task's conditional, a length-prefixed framing was therefore **not**
implemented; the model stores only `code: i32` and `payload: Vec<u8>`, with
`encode()`/`decode()` producing/consuming the observed `[code byte][payload]`
shape. Handlers' 5-byte minimum is exposed as `MIN_BUFFERED_BYTES`.

## Plugin ordering source

The ordered startup list (`PLUGIN_STARTUP_ORDER`) is taken from
`controller/ProgramController.java`, method `i(String)`, which registers into
`ProgramEnvironment` (a `CopyOnWriteArrayList`, so insertion order is start
order via `ProgramEnvironment.h()`):

1. `PerfPlugin` (`/tmp/.perf/V0`)
2. `WMPlugin` (`/tmp/.wm/V0`)
3. sound backend — `PulseAudioPlugin` (`/tmp/.sound/PS0`) when config mapping == 1, else `ALSAServerPlugin` (`/tmp/.sound/AS0`)
4. `RenderDocPlugin` (`/tmp/.rdc/V0`, when enabled)
5. `SysVSharedMemoryPlugin` (`/tmp/.sysvshm/SM0`)
6. `NetworkInfoUpdatePlugin`
7. `ProgramLauncher` (one or more: jwm / main program / steam)

Socket paths are the exact `UnixSocketConfig.a(root, path)` arguments in that
file. `ProgramPlugin` lifecycle maps `a()` -> `Started`, `b()` -> `Stopped`,
with `Loaded` as the constructed state.

## WinMonitor beans

Plain structs modeled from `winmonitor/*.java`: `ProcessInfo`, `ThreadInfo`,
`StackFrame`, `ThreadDetailInfo`, `CommandRequest`, and the four response beans
(`KillProcessResponse`, `ProcessListResponse`, `ThreadListResponse`,
`ThreadStackResponse`). Field names/order come from each decompiled class's
constructor/`toString()` (e.g. `ProcessInfo(name, pid, path)`).

## Out of scope (deliberately not modeled)

- Sockets and unix sockets: `socket/Client.java`, `ClientSocket.java`,
  `ConnectionHandler.java`, `RequestHandler.java`, `SocketInputStream.java`,
  `SocketOutputStream.java`, `SocketStreamLock.java`, `UnixSocketConfig.java`,
  `XConnectorEpoll.java`, `XServer.java`.
- Real IPC / server threads / epoll event loops and connection handlers
  (`*ConnectionHandler.java`, `XConnectorEpoll`).
- Native servers and JNI: the `winemu` native library (`System.loadLibrary`),
  `SysVSharedMemory` native methods (`createMemoryFd`, `mapSHMSegment`, ...),
  `ALSAClient` native down-mixers and Android `AudioTrack`.
- Android platform code: `Context`, `ConnectivityManager`/broadcast receivers,
  process spawning (`ProcessHelper`), Gson JSON serialization, UDP
  `DatagramSocket` transport.
- Other servers not requested: `log/LogHttpServer.java`,
  `tabtip/TabTipServer.java`.

## Tests

`#[cfg(test)]` unit tests live in each module and cover: exact code values and
`from_code` round-trips for every subsystem, the no-numeric-codes fact for
WinMonitor, `[code byte][payload]` encode/decode, plugin lifecycle
transitions, exact socket paths, plugin startup ordering, and WinMonitor bean
field construction. Run with `cargo test` once this module is included in the
crate (this agent did not modify `app/src/lib.rs` or `Cargo.toml`).

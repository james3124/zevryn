//! Numeric request/command codes used by the server subsystems.
//!
//! IMPORTANT SOURCE NOTE
//! ---------------------
//! In the decompiled reference every `RequestCodes.java`
//! (`alsaserver/RequestCodes.java`, `wm/RequestCodes.java`,
//! `perf/RequestCodes.java`, `renderdoc/RequestCodes.java`,
//! `sysvshm/RequestCodes.java`) is an **empty abstract class**. The Java
//! compiler inlined the `static final int` constants into the call sites, so
//! the literal numeric values must be recovered from the `*RequestHandler.java`
//! switch/`if` statements. Each enum below documents the exact source file the
//! values were recovered from. All codes are wire-level `byte`/`i32` values.
//!
//! The common request header is a **single code byte** read first by every
//! handler (`SocketInputStream.e()`), followed by subsystem-specific payload.
//! Handlers buffer-check `a() < 5` before parsing (code byte + at least a
//! 4-byte int). There is no whole-message length-prefix framing in the source.

/// ALSA audio server codes.
///
/// Source: `alsaserver/ALSARequestHandler.java`, method `a(Client)`,
/// `switch (socketInputStreamB.e())` cases 0..=7. The byte selects an
/// `ALSAClient` operation (`alsaserver/ALSAClient.java`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlsaRequestCode {
    /// case 0: `ALSAClient.h()` - unmap SHM buffer and release/flush AudioTrack.
    Reset = 0,
    /// case 1: `ALSAClient.n()` - start playback (AudioTrack.play()).
    Play = 1,
    /// case 2: `ALSAClient.o()` - stop playback and flush.
    Stop = 2,
    /// case 3: `ALSAClient.e()` - pause playback.
    Pause = 3,
    /// case 4: configure channels/data-type/sample-rate/frame-count, then
    /// create + map the shared-memory audio buffer (`ALSARequestHandler.b`).
    Setup = 4,
    /// case 5: write audio data (`ALSAClient.p(ByteBuffer)`), payload length is
    /// the int read after the code.
    Write = 5,
    /// case 6: `ALSAClient.a()` - flush AudioTrack.
    Flush = 6,
    /// case 7: `ALSAClient.f()` - query playback position, returns an int.
    GetPosition = 7,
}

impl AlsaRequestCode {
    /// Recover a code from its on-wire value, or `None` if unknown.
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Reset),
            1 => Some(Self::Play),
            2 => Some(Self::Stop),
            3 => Some(Self::Pause),
            4 => Some(Self::Setup),
            5 => Some(Self::Write),
            6 => Some(Self::Flush),
            7 => Some(Self::GetPosition),
            _ => None,
        }
    }

    /// The on-wire i32 value.
    pub fn code(self) -> i32 {
        self as i32
    }
}

/// Window-manager server codes.
///
/// Source: `wm/WMRequestHandler.java`, method `a(Client)`: the code byte is
/// compared to `1`; when it matches, `WMPlugin.c().b()` is invoked, which is
/// the callback held in `wm/WMCallback.java` (`void b()`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WmRequestCode {
    /// code 1: fire the registered WM callback (`WMCallback.b()`). Code 1 is
    /// the only code handled; all others are ignored by the source logic.
    FireCallback = 1,
}

impl WmRequestCode {
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            1 => Some(Self::FireCallback),
            _ => None,
        }
    }

    pub fn code(self) -> i32 {
        self as i32
    }
}

/// Performance-monitoring server codes.
///
/// Source: `perf/PerfRequestHandler.java`, method `a(Client)`, which compares
/// the code byte to `10` and `11`. The listener interface is
/// `perf/PerfEventListener.java`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PerfRequestCode {
    /// code 10: after skipping a long, reads three length-prefixed strings and
    /// two ints, then calls `PerfEventListener.d(pid, s1, s2, s3, i1, i2)`.
    ReportMetrics = 10,
    /// code 11: reads a float and calls `PerfEventListener.a(pid, value)`.
    UpdateValue = 11,
}

impl PerfRequestCode {
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            10 => Some(Self::ReportMetrics),
            11 => Some(Self::UpdateValue),
            _ => None,
        }
    }

    pub fn code(self) -> i32 {
        self as i32
    }
}

/// RenderDoc server codes.
///
/// Source: `renderdoc/RenderDocRequestHandler.java`, method `a(Client)`: only
/// code `10` is acted upon; the handler replies with an int `0` (`i(0)`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderDocRequestCode {
    /// code 10: ping/status request; server writes a zero int back.
    Ping = 10,
}

impl RenderDocRequestCode {
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            10 => Some(Self::Ping),
            _ => None,
        }
    }

    pub fn code(self) -> i32 {
        self as i32
    }
}

/// System-V shared-memory server codes.
///
/// Source: `sysvshm/SysVSHMRequestHandler.java`, method `a(Client)`,
/// `if (bE == 0/1/2/3) { ... }`. Operations delegate to
/// `sysvshm/SysVSharedMemory.java`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SysvRequestCode {
    /// code 0: `SysVSharedMemory.get(long size)` - create a segment, return id.
    Create = 0,
    /// code 1: `SysVSharedMemory.getFd(int id)` - return the segment fd.
    GetFd = 1,
    /// code 2: `SysVSharedMemory.delete(int id)` - delete a segment.
    Delete = 2,
    /// code 3: `SysVSharedMemory.getSize(int id)` - return segment size.
    GetSize = 3,
}

impl SysvRequestCode {
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Create),
            1 => Some(Self::GetFd),
            2 => Some(Self::Delete),
            3 => Some(Self::GetSize),
            _ => None,
        }
    }

    pub fn code(self) -> i32 {
        self as i32
    }
}

/// WinMonitor command identifiers.
///
/// SOURCE NOTE: unlike the five subsystems above, the decompiled
/// `winmonitor/` package contains **no numeric request codes**. Commands are
/// *named strings* serialized as JSON by Gson and sent over a UDP
/// `DatagramSocket` (`winmonitor/WinMonitorClient.java`: fields `Gson` and
/// `DatagramSocket`, 5000 ms timeout). `winmonitor/CommandRequest.java` carries
/// a `String command` field. Therefore there is no i32 table to recover here.
///
/// The variants below are the logical commands inferred from the response bean
/// classes present in the source (`ProcessListResponse`, `ThreadListResponse`,
/// `ThreadStackResponse`, `KillProcessResponse`). Their `&str` names are
/// descriptive labels, **not** literal constants found in the decompiled Java.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WinMonitorRequestCode {
    /// Corresponds to `ProcessListResponse` / `ProcessInfo`.
    GetProcessList,
    /// Corresponds to `ThreadListResponse` / `ThreadInfo`.
    GetThreadList,
    /// Corresponds to `ThreadStackResponse` / `ThreadDetailInfo`.
    GetThreadStack,
    /// Corresponds to `KillProcessResponse`.
    KillProcess,
}

/// WinMonitor does not use numeric wire codes in the reference source.
pub const WINMONITOR_HAS_NUMERIC_CODES: bool = false;

impl WinMonitorRequestCode {
    /// Descriptive command label (inferred from the response bean names).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GetProcessList => "getProcessList",
            Self::GetThreadList => "getThreadList",
            Self::GetThreadStack => "getThreadStack",
            Self::KillProcess => "killProcess",
        }
    }

    /// Always `None`: the source defines no numeric code for WinMonitor.
    pub fn from_code(_code: i32) -> Option<Self> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alsa_codes_match_source_switch_cases() {
        assert_eq!(AlsaRequestCode::Reset.code(), 0);
        assert_eq!(AlsaRequestCode::Play.code(), 1);
        assert_eq!(AlsaRequestCode::Stop.code(), 2);
        assert_eq!(AlsaRequestCode::Pause.code(), 3);
        assert_eq!(AlsaRequestCode::Setup.code(), 4);
        assert_eq!(AlsaRequestCode::Write.code(), 5);
        assert_eq!(AlsaRequestCode::Flush.code(), 6);
        assert_eq!(AlsaRequestCode::GetPosition.code(), 7);
        for code in 0..=7 {
            assert_eq!(AlsaRequestCode::from_code(code).map(|c| c.code()), Some(code));
        }
        assert_eq!(AlsaRequestCode::from_code(8), None);
        assert_eq!(AlsaRequestCode::from_code(-1), None);
    }

    #[test]
    fn wm_code_matches_source() {
        assert_eq!(WmRequestCode::FireCallback.code(), 1);
        assert_eq!(WmRequestCode::from_code(1), Some(WmRequestCode::FireCallback));
        assert_eq!(WmRequestCode::from_code(0), None);
        assert_eq!(WmRequestCode::from_code(2), None);
    }

    #[test]
    fn perf_codes_match_source() {
        assert_eq!(PerfRequestCode::ReportMetrics.code(), 10);
        assert_eq!(PerfRequestCode::UpdateValue.code(), 11);
        assert_eq!(PerfRequestCode::from_code(10), Some(PerfRequestCode::ReportMetrics));
        assert_eq!(PerfRequestCode::from_code(11), Some(PerfRequestCode::UpdateValue));
        assert_eq!(PerfRequestCode::from_code(12), None);
    }

    #[test]
    fn renderdoc_code_matches_source() {
        assert_eq!(RenderDocRequestCode::Ping.code(), 10);
        assert_eq!(RenderDocRequestCode::from_code(10), Some(RenderDocRequestCode::Ping));
        assert_eq!(RenderDocRequestCode::from_code(9), None);
    }

    #[test]
    fn sysv_codes_match_source() {
        assert_eq!(SysvRequestCode::Create.code(), 0);
        assert_eq!(SysvRequestCode::GetFd.code(), 1);
        assert_eq!(SysvRequestCode::Delete.code(), 2);
        assert_eq!(SysvRequestCode::GetSize.code(), 3);
        for code in 0..=3 {
            assert_eq!(SysvRequestCode::from_code(code).map(|c| c.code()), Some(code));
        }
        assert_eq!(SysvRequestCode::from_code(4), None);
    }

    #[test]
    fn winmonitor_has_no_numeric_codes() {
        assert!(!WINMONITOR_HAS_NUMERIC_CODES);
        assert_eq!(WinMonitorRequestCode::from_code(10), None);
        assert_eq!(WinMonitorRequestCode::GetProcessList.as_str(), "getProcessList");
    }
}

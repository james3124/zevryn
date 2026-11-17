//! WinMonitor data beans, modeled as plain owned structs.
//!
//! SOURCE: `winmonitor/*.java` in the decompiled reference. The originals are
//! Kotlin data classes (decompiled to final Java classes with `equals`,
//! `hashCode` and a descriptive `toString`), serialized as JSON via Gson over
//! a UDP `DatagramSocket`. Here they are simple Rust structs with the same
//! fields, derived from each class's `toString()` / field list:
//!
//! - `ProcessInfo(name, pid, path)`  -> f20836a/f20837b/f20838c
//! - `ThreadInfo(threadId, ownerPid, priority, name)` -> f20850a/f20851b/f20852c/f20853d
//! - `StackFrame(address, module, symbol)` -> f20842a/f20843b/f20844c
//! - `ThreadDetailInfo(threadId, ownerPid, priority, name, stackFrames)`
//! - `CommandRequest(command, pid, threadId)`
//! - responses `*Response(status, data/message)` with a `message` label.

use std::fmt;

/// `winmonitor/CommandRequest.java`: a named command with optional pid/thread id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRequest {
    pub command: String,
    /// Nullable in the source (`Integer`).
    pub pid: Option<i32>,
    /// Nullable in the source (`Long`).
    pub thread_id: Option<i64>,
}

/// `winmonitor/ProcessInfo.java`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: i32,
    pub path: String,
}

/// `winmonitor/ThreadInfo.java`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadInfo {
    pub thread_id: i64,
    pub owner_pid: i64,
    pub priority: i32,
    pub name: String,
}

/// `winmonitor/StackFrame.java`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    pub address: String,
    pub module: String,
    pub symbol: String,
}

/// `winmonitor/ThreadDetailInfo.java`: a thread plus its captured stack frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadDetailInfo {
    pub thread_id: i64,
    pub owner_pid: i64,
    pub priority: i32,
    pub name: String,
    pub stack_frames: Vec<StackFrame>,
}

/// `winmonitor/KillProcessResponse.java`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KillProcessResponse {
    pub status: String,
    pub success: Option<bool>,
    pub message: Option<String>,
}

/// `winmonitor/ProcessListResponse.java`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessListResponse {
    pub status: String,
    pub data: Vec<ProcessInfo>,
    pub message: Option<String>,
}

/// `winmonitor/ThreadListResponse.java`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadListResponse {
    pub status: String,
    pub data: Vec<ThreadInfo>,
    pub message: Option<String>,
}

/// `winmonitor/ThreadStackResponse.java`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadStackResponse {
    pub status: String,
    pub data: Option<ThreadDetailInfo>,
    pub message: Option<String>,
}

impl fmt::Display for ProcessInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProcessInfo(name={}, pid={}, path={})", self.name, self.pid, self.path)
    }
}

impl fmt::Display for ThreadInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ThreadInfo(threadId={}, ownerPid={}, priority={}, name={})",
            self.thread_id, self.owner_pid, self.priority, self.name
        )
    }
}

impl fmt::Display for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StackFrame(address={}, module={}, symbol={})",
            self.address, self.module, self.symbol
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_info_fields() {
        let p = ProcessInfo {
            name: "steam.exe".into(),
            pid: 42,
            path: "C:\\steam\\steam.exe".into(),
        };
        assert_eq!(p.pid, 42);
        assert_eq!(p.to_string(), "ProcessInfo(name=steam.exe, pid=42, path=C:\\steam\\steam.exe)");
    }

    #[test]
    fn thread_info_fields() {
        let t = ThreadInfo {
            thread_id: 7,
            owner_pid: 42,
            priority: 5,
            name: "main".into(),
        };
        assert_eq!(t.thread_id, 7);
        assert_eq!(t.owner_pid, 42);
        assert!(t.to_string().starts_with("ThreadInfo(threadId=7"));
    }

    #[test]
    fn stack_frames_nest_in_detail_info() {
        let frames = vec![StackFrame {
            address: "0x1000".into(),
            module: "steam.exe".into(),
            symbol: "main".into(),
        }];
        let d = ThreadDetailInfo {
            thread_id: 7,
            owner_pid: 42,
            priority: 5,
            name: "main".into(),
            stack_frames: frames.clone(),
        };
        assert_eq!(d.stack_frames, frames);
        assert_eq!(d.stack_frames[0].symbol, "main");
    }

    #[test]
    fn responses_hold_expected_payloads() {
        let pl = ProcessListResponse {
            status: "ok".into(),
            data: vec![],
            message: None,
        };
        assert_eq!(pl.status, "ok");
        let ks = ThreadStackResponse {
            status: "ok".into(),
            data: None,
            message: Some("no such thread".into()),
        };
        assert!(ks.data.is_none());
        assert_eq!(ks.message.as_deref(), Some("no such thread"));
    }

    #[test]
    fn command_request_optional_fields() {
        let c = CommandRequest {
            command: "getProcessList".into(),
            pid: None,
            thread_id: Some(9),
        };
        assert_eq!(c.pid, None);
        assert_eq!(c.thread_id, Some(9));
    }
}

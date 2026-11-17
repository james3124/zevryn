//! Rust model of the portable part of `com.winemu.core.utils.PortUtils`
//! (GameHub Lite v5.1.8).
//!
//! Java `PortUtils.a()` does `new DatagramSocket(0).getLocalPort()` — it asks
//! the OS for an ephemeral port. Real sockets are out of scope here, so this
//! module models the **range** semantics used by callers: a half-open/inclusive
//! port range plus an ordered "find first free port" scan against a set of
//! already-used ports. No `std::net` socket is ever created.

/// Default scan range used by callers that look for a free local port.
pub const DEFAULT_PORT_START: u16 = 20_000;
pub const DEFAULT_PORT_END: u16 = 30_000;

/// An inclusive range of TCP/UDP port numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

impl PortRange {
    /// Build a validated range. Ports must be non-zero (port 0 means
    /// "let the OS choose" and is not a usable fixed port) and `start <= end`.
    pub fn new(start: u16, end: u16) -> Option<Self> {
        if start == 0 || end == 0 || start > end {
            None
        } else {
            Some(Self { start, end })
        }
    }

    /// The default range `20_000..=30_000`.
    pub fn default_range() -> Self {
        Self {
            start: DEFAULT_PORT_START,
            end: DEFAULT_PORT_END,
        }
    }

    /// Whether `port` lies inside this inclusive range.
    pub fn contains(&self, port: u16) -> bool {
        (self.start..=self.end).contains(&port)
    }

    /// Number of ports in the range (always >= 1 for a valid [`PortRange`]).
    pub fn len(&self) -> usize {
        (self.end - self.start) as usize + 1
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    /// Iterate ports in ascending order.
    pub fn iter(&self) -> impl Iterator<Item = u16> {
        self.start..=self.end
    }

    /// `PortUtils.a()` analogue: return the first port in the range not present
    /// in `used`, or `None` when every port is taken. Ports outside the range
    /// in `used` are ignored.
    pub fn first_free(&self, used: &[u16]) -> Option<u16> {
        self.iter().find(|p| !used.contains(p))
    }

    /// The next free port strictly after `after`, wrapping is **not** performed
    /// (mirrors a single scan of the range).
    pub fn next_free_after(&self, after: u16, used: &[u16]) -> Option<u16> {
        self.iter().find(|p| *p > after && !used.contains(p))
    }
}

/// Convenience free function: first free port in the default range.
pub fn find_free_port(used: &[u16]) -> Option<u16> {
    PortRange::default_range().first_free(used)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_validation() {
        assert_eq!(PortRange::new(0, 100), None);
        assert_eq!(PortRange::new(100, 0), None);
        assert_eq!(PortRange::new(200, 100), None);
        let r = PortRange::new(100, 100).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r.start, 100);
    }

    #[test]
    fn contains_and_len() {
        let r = PortRange::new(1000, 1003).unwrap();
        assert!(r.contains(1000));
        assert!(r.contains(1003));
        assert!(!r.contains(999));
        assert!(!r.contains(1004));
        assert_eq!(r.len(), 4);
        assert_eq!(r.iter().collect::<Vec<_>>(), vec![1000, 1001, 1002, 1003]);
    }

    #[test]
    fn first_free_skips_used_ports() {
        let r = PortRange::new(5000, 5003).unwrap();
        assert_eq!(r.first_free(&[]), Some(5000));
        assert_eq!(r.first_free(&[5000, 5002]), Some(5001));
        assert_eq!(r.first_free(&[4999, 5000, 5001, 5002, 5003, 6000]), None);
        // used ports outside the range don't matter
        assert_eq!(r.first_free(&[4999, 6000]), Some(5000));
    }

    #[test]
    fn next_free_after() {
        let r = PortRange::new(10, 13).unwrap();
        let used = [11];
        assert_eq!(r.next_free_after(9, &used), Some(10));
        assert_eq!(r.next_free_after(10, &used), Some(12));
        assert_eq!(r.next_free_after(13, &used), None);
    }

    #[test]
    fn default_range_and_helper() {
        let r = PortRange::default_range();
        assert_eq!(r.start, DEFAULT_PORT_START);
        assert_eq!(r.end, DEFAULT_PORT_END);
        assert_eq!(find_free_port(&[DEFAULT_PORT_START]), Some(DEFAULT_PORT_START + 1));
    }
}

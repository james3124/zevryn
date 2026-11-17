//! Pure helpers modeled on `com.winemu.core.utils.BatteryUtils`
//! (GameHub Lite v5.1.8).
//!
//! The Android `BatteryManager`/`Context.registerReceiver` calls are out of
//! scope. The numeric transforms in `a()`, `b()` and `c()` are pure and kept
//! here so callers can normalize raw battery readings without a device.

/// `BatteryUtils.a(BatteryManager)` — normalize a raw
/// `BATTERY_PROPERTY_CURRENT_NOW` reading (in microamps, discharge negative).
/// The Java code negates it, then either scales by 1000 or returns as-is
/// depending on magnitude. `raw` is the raw `getIntProperty(2)` value.
pub fn normalize_current(raw_property: i32) -> f32 {
    let mut value = -raw_property as f32;
    if value < 0.0 {
        // Java: if abs(value/1000) < 1 (i.e. |value| < 1000) multiply by 1000.
        if (value / 1000.0).abs() < 1.0 {
            value *= 1000.0;
        }
    } else if value.abs() <= 100_000.0 {
        return value;
    }
    value / 1000.0
}

/// `BatteryUtils.b(Context)` — voltage: a value > 20 is treated as millivolts
/// and divided by 1000; otherwise it is already volts.
pub fn voltage_from_extra(raw: i32) -> f32 {
    let value = raw as f32;
    if value > 20.0 {
        value / 1000.0
    } else {
        value
    }
}

/// `BatteryUtils.c(Context)` — charging state. Java returns
/// `!(plugged == 0)`, where `plugged == 0` means "not plugged".
pub fn is_charging(plugged: i32) -> bool {
    plugged != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_normalization() {
        // negative property => positive discharge current
        assert_eq!(normalize_current(-500_000), 500.0);
        assert_eq!(normalize_current(500_000), -500.0);
        // small negative result (< 1000) is scaled by 1000 then divided -> same
        assert_eq!(normalize_current(5), -5.0);
        // magnitude <= 100000 positive returns as-is
        assert_eq!(normalize_current(-50_000), 50_000.0);
    }

    #[test]
    fn voltage_scaling() {
        assert_eq!(voltage_from_extra(3800), 3.8);
        assert_eq!(voltage_from_extra(-1), -1.0);
        assert_eq!(voltage_from_extra(12), 12.0);
    }

    #[test]
    fn charging_state() {
        assert!(is_charging(1)); // USB/AC
        assert!(!is_charging(0));
    }
}

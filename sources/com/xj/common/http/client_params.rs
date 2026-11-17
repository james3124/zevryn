//! Portable model of `com.xj.common.http.ClientParams.b()`.
//!
//! The interceptor inserts the result of `ClientParams.b()` into every signed
//! request under the key `clientparams`. It is a pipe-separated string built
//! from a `LinkedHashMap` of device/app fields (order preserved). Android-only
//! values are represented as caller-supplied strings with empty defaults.

/// A single ordered `clientparams` value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientParams {
    pub client_version: String, // 客户端版本 = AppUtils.e()
    pub android_version: String, // 安卓版本
    pub language: String,        // 语言
    pub model: String,           // 机型 = Build.MODEL
    pub resolution: String,      // 分辨率 = "W * H"
    pub protocol_version: String, // 协议版本
    pub channel: String,          // 渠道编号 = protocol + "_" + channel_no
    pub brand: String,            // 品牌 = Build.BRAND
    pub app_package: String,      // APP对应包名
    pub gpu: String,              // GPU
    pub cpu: String,              // CPU
}

impl Default for ClientParams {
    fn default() -> Self {
        Self {
            client_version: String::new(),
            android_version: String::new(),
            language: String::new(),
            model: String::new(),
            resolution: String::new(),
            protocol_version: String::new(),
            channel: String::new(),
            brand: String::new(),
            app_package: String::new(),
            gpu: String::new(),
            cpu: String::new(),
        }
    }
}

impl ClientParams {
    /// Build the ordered `|`-joined value exactly like `ClientParams.b()`:
    /// 19 fields, with the 10 "unknown" device fields always empty strings,
    /// followed by app package, GPU and CPU.
    pub fn value(&self) -> String {
        let fields: [&str; 19] = [
            &self.client_version,
            &self.android_version,
            &self.language,
            &self.model,
            &self.resolution,
            &self.protocol_version,
            &self.channel,
            &self.brand,
            "", // 生产厂家
            "", // MAC地址
            "", // IMEI
            "", // 地址位置
            "", // 手柄名称
            "", // 键盘名称
            "", // 鼠标名称
            "", // 固件版本
            &self.app_package,
            &self.gpu,
            &self.cpu,
        ];
        fields.join("|")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_nineteen_fields() {
        let p = ClientParams {
            client_version: "5.1.8".into(),
            language: "zh".into(),
            ..Default::default()
        };
        let v = p.value();
        assert_eq!(v.split('|').count(), 19);
        assert_eq!(v.split('|').next(), Some("5.1.8"));
        assert!(v.contains("|zh|"));
    }

    #[test]
    fn default_is_all_empty_but_has_delimiters() {
        let v = ClientParams::default().value();
        assert_eq!(v.split('|').count(), 19);
        assert!(v.chars().all(|c| c == '|'));
    }
}

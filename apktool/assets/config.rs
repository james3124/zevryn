//! Parser for the bundled Aria downloader configuration (`aria_config.xml`).
//!
//! Mirrored implementation of the core crate's `src/config.rs`. The real asset
//! uses the flat `<key value="..." />` form (with XML comments), but the parser
//! also accepts the plain `<key>...</key>` text form. No XML library is used;
//! only a small `find`-based subset is needed for this fixed config file.

/// Download settings read from `aria_config.xml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AriaConfig {
    pub max_speed: Option<i64>,
    pub thread_num: Option<i32>,
    pub re_try_num: Option<i32>,
    pub re_try_interval: Option<i64>,
    pub connect_time_out: Option<i64>,
    pub io_time_out: Option<i64>,
    pub use_block: Option<bool>,
    pub use_head_request: Option<bool>,
    pub net_check: Option<bool>,
    pub log_level: Option<i32>,
    pub max_task_num: Option<i32>,
}

impl AriaConfig {
    /// Extract scalar values by tag name, ignoring XML attributes and comments.
    pub fn parse(xml: &str) -> Self {
        let get = |key: &str| tag_value(xml, key);
        Self {
            max_speed: get("maxSpeed").and_then(|v| v.parse().ok()),
            thread_num: get("threadNum").and_then(|v| v.parse().ok()),
            re_try_num: get("reTryNum").and_then(|v| v.parse().ok()),
            re_try_interval: get("reTryInterval").and_then(|v| v.parse().ok()),
            connect_time_out: get("connectTimeOut").and_then(|v| v.parse().ok()),
            io_time_out: get("iOTimeOut").and_then(|v| v.parse().ok()),
            use_block: get("useBlock").and_then(|v| parse_bool(&v)),
            use_head_request: get("useHeadRequest").and_then(|v| parse_bool(&v)),
            net_check: get("netCheck").and_then(|v| parse_bool(&v)),
            log_level: get("logLevel").and_then(|v| v.parse().ok()),
            max_task_num: get("maxTaskNum").and_then(|v| v.parse().ok()),
        }
    }
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.trim() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

fn tag_value(xml: &str, key: &str) -> Option<String> {
    let open = format!("<{key}");
    let start = xml.find(&open)?;
    let tag_end = xml[start..].find('>')? + start;
    let tag = &xml[start..tag_end];

    // Form used by the real aria_config.xml: <key value="..." />
    if let Some(v) = attr_value(tag, "value") {
        return Some(v);
    }

    // Text form: <key>...</key>
    let close = format!("</{key}>");
    let body_start = tag_end + 1;
    let end = xml[body_start..].find(&close)? + body_start;
    Some(xml[body_start..end].trim().to_owned())
}

fn attr_value(tag: &str, name: &str) -> Option<String> {
    let pat = format!("{name}=\"");
    let start = tag.find(&pat)? + pat.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFERENCE_ASSET: &str = "/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/apktool/assets/aria_config.xml";
    const LOCAL_ASSET: &str = "/home/Project/Zevryn/app/apktool/assets/aria_config.xml";

    fn read_config() -> Option<String> {
        std::fs::read_to_string(REFERENCE_ASSET)
            .or_else(|_| std::fs::read_to_string(LOCAL_ASSET))
            .ok()
    }

    #[test]
    fn parses_attribute_form() {
        // This is the form used by the real aria_config.xml.
        let xml = r#"<aria><app><logLevel value="1" /><netCheck value="true" /></app>
            <download><useHeadRequest value="false" /><maxSpeed value="0" />
            <useBlock value="true" /><threadNum value="3" /><maxTaskNum value="2" />
            <reTryNum value="3" /><reTryInterval value="5000" />
            <connectTimeOut value="5000" /><iOTimeOut value="10000" /></download></aria>"#;
        let c = AriaConfig::parse(xml);
        assert_eq!(c.max_speed, Some(0));
        assert_eq!(c.thread_num, Some(3));
        assert_eq!(c.re_try_num, Some(3));
        assert_eq!(c.re_try_interval, Some(5000));
        assert_eq!(c.connect_time_out, Some(5000));
        assert_eq!(c.io_time_out, Some(10000));
        assert_eq!(c.use_block, Some(true));
        assert_eq!(c.use_head_request, Some(false));
        assert_eq!(c.net_check, Some(true));
        assert_eq!(c.log_level, Some(1));
        assert_eq!(c.max_task_num, Some(2));
    }

    #[test]
    fn parses_text_form() {
        let xml = r#"<aria><download><maxSpeed>1024</maxSpeed><threadNum>5</threadNum>
            <reTryNum>2</reTryNum><reTryInterval>1000</reTryInterval>
            <connectTimeOut>3000</connectTimeOut><iOTimeOut>20000</iOTimeOut>
            <useBlock>false</useBlock><useHeadRequest>1</useHeadRequest>
            <netCheck>0</netCheck><logLevel>2</logLevel><maxTaskNum>4</maxTaskNum>
            </download></aria>"#;
        let c = AriaConfig::parse(xml);
        assert_eq!(c.max_speed, Some(1024));
        assert_eq!(c.thread_num, Some(5));
        assert_eq!(c.re_try_num, Some(2));
        assert_eq!(c.re_try_interval, Some(1000));
        assert_eq!(c.connect_time_out, Some(3000));
        assert_eq!(c.io_time_out, Some(20000));
        assert_eq!(c.use_block, Some(false));
        assert_eq!(c.use_head_request, Some(true));
        assert_eq!(c.net_check, Some(false));
        assert_eq!(c.log_level, Some(2));
        assert_eq!(c.max_task_num, Some(4));
    }

    #[test]
    fn missing_and_malformed_values_are_none() {
        let c = AriaConfig::parse("<aria><download><threadNum>oops</threadNum></download></aria>");
        assert_eq!(c.thread_num, None);
        assert_eq!(c.max_speed, None);
        assert_eq!(c.use_block, None);
    }

    #[test]
    fn parses_real_config_when_present() {
        let Some(xml) = read_config() else { return; };
        let c = AriaConfig::parse(&xml);
        assert_eq!(c.thread_num, Some(3), "threadNum missing from real config");
        assert_eq!(c.max_speed, Some(0));
        assert_eq!(c.re_try_num, Some(3));
        assert_eq!(c.re_try_interval, Some(5000));
        assert_eq!(c.connect_time_out, Some(5000));
        assert_eq!(c.io_time_out, Some(10000));
        assert_eq!(c.use_block, Some(true));
        assert_eq!(c.use_head_request, Some(false));
        assert_eq!(c.net_check, Some(true));
        assert_eq!(c.log_level, Some(1));
        assert_eq!(c.max_task_num, Some(2));
    }
}

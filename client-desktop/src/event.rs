use chrono::{Local, SecondsFormat, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::browser::BrowserContext;
use eyes_on_me_shared::PresenceState;

const KNOWN_WORK_APPS: &[&str] = &[
    "code",
    "cursor",
    "idea",
    "pycharm",
    "webstorm",
    "clion",
    "goland",
    "rustrover",
    "terminal",
    "wezterm",
    "powershell",
    "cmd",
    "windows terminal",
    "alacritty",
    "warp",
    "zellij",
    "tmux",
    "datagrip",
    "dbeaver",
    "postman",
    "insomnia",
    "tableplus",
    "notion",
    "obsidian",
    "figma",
    "draw.io",
    "slack",
    "discord",
    "telegram",
    "wechat",
    "chrome",
    "google chrome",
    "msedge",
    "microsoft edge",
    "firefox",
    "safari",
    "arc",
    "brave",
    "vivaldi",
    "opera",
];

const KNOWN_SYSTEM_APPS: &[&str] = &["system.idle", "system.locked", "system.unknown"];
const KNOWN_COMM_APPS: &[&str] = &["slack", "discord", "telegram", "wechat", "qq", "feishu"];
const KNOWN_BROWSER_IDS: &[&str] = &[
    "chrome",
    "chromium",
    "firefox",
    "safari",
    "arc",
    "brave",
    "vivaldi",
    "opera",
    "edge",
    "msedge",
    "qqbrowser",
    "360se",
    "360chrome",
    "sogouexplorer",
    "zen",
    "orion",
];

#[derive(Debug, Clone, Serialize)]
pub struct ActivityEnvelope {
    #[serde(rename = "type")]
    pub message_type: &'static str,
    pub payload: ActivityPayload,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityPayload {
    pub event_id: String,
    pub ts: String,
    pub device_id: String,
    pub agent_name: String,
    pub platform: &'static str,
    pub kind: &'static str,
    pub app: AppInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<BrowserContext>,
    pub presence: PresenceState,
    pub source: &'static str,
    pub summary: ActivitySummary,
    pub meta: ActivityMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub pid: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySummary {
    pub current_label: String,
    pub status_line: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_key: Option<String>,
    pub app_key: String,
    pub app_group: String,
    pub presence: &'static str,
    pub is_browser: bool,
    pub is_active: bool,
    pub is_idle: bool,
    pub is_locked: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityMeta {
    pub app_key: String,
    pub app_display_name: String,
    pub app_group: String,
    pub is_browser: bool,
    pub is_idle: bool,
    pub is_locked: bool,
    pub is_active: bool,
    pub presence_label: &'static str,
    pub activity_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_or_page_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_page_title: Option<String>,
    pub local_date: String,
    pub local_time: String,
    pub local_datetime: String,
}

impl ActivityEnvelope {
    pub fn activity(
        device_id: &str,
        agent_name: &str,
        platform: &'static str,
        source: &'static str,
        kind: &'static str,
        app: AppInfo,
        window_title: Option<String>,
        browser: Option<BrowserContext>,
        presence: PresenceState,
    ) -> Self {
        Self {
            message_type: "activity",
            payload: normalized_payload(
                device_id,
                agent_name,
                platform,
                source,
                kind,
                app,
                window_title,
                browser,
                presence,
            ),
        }
    }
}

fn normalized_payload(
    device_id: &str,
    agent_name: &str,
    platform: &'static str,
    source: &'static str,
    kind: &'static str,
    app: AppInfo,
    window_title: Option<String>,
    browser: Option<BrowserContext>,
    presence: PresenceState,
) -> ActivityPayload {
    let (app, window_title, browser) = normalize_payload_fields(app, window_title, browser);
    let summary = build_summary(&app, window_title.as_deref(), browser.as_ref(), presence);
    let meta = build_meta(&app, window_title.as_deref(), browser.as_ref(), presence);

    ActivityPayload {
        event_id: new_event_id(),
        ts: now_rfc3339(),
        device_id: normalize_device_id(device_id),
        agent_name: normalize_agent_name(agent_name),
        platform,
        kind,
        app,
        window_title,
        browser,
        presence,
        source,
        summary,
        meta,
    }
}

fn normalize_payload_fields(
    app: AppInfo,
    window_title: Option<String>,
    browser: Option<BrowserContext>,
) -> (AppInfo, Option<String>, Option<BrowserContext>) {
    let mut app = sanitize_app(app);
    let window_title = sanitize_window_title(window_title);
    let mut browser = sanitize_browser(browser);

    if app.title.is_none() {
        app.title = window_title.clone();
    }
    fill_browser_page_title(&mut browser, window_title.as_deref());

    (app, window_title, browser)
}

fn sanitize_app(mut app: AppInfo) -> AppInfo {
    app.id = app.id.trim().to_string();
    app.name = app.name.trim().to_string();
    app.title = app.title.and_then(non_empty_string);
    app
}

fn sanitize_window_title(window_title: Option<String>) -> Option<String> {
    window_title.and_then(non_empty_string)
}

fn sanitize_browser(browser: Option<BrowserContext>) -> Option<BrowserContext> {
    browser.map(|mut context| {
        context.family = context.family.trim().to_string();
        context.name = context.name.trim().to_string();
        context.source = context.source.trim().to_string();
        context.page_title = context.page_title.and_then(non_empty_string);
        context.domain = context.domain.and_then(non_empty_lowercase_string);

        let blocked_url = match context.url.as_deref() {
            Some(url) => {
                url.starts_with("file://")
                    || url.starts_with("about:")
                    || url.starts_with("chrome://")
                    || url.starts_with("edge://")
            }
            None => false,
        };

        if blocked_url {
            context.url = None;
        } else {
            context.url = context.url.and_then(non_empty_string);
        }

        context
    })
}

fn fill_browser_page_title(browser: &mut Option<BrowserContext>, window_title: Option<&str>) {
    if let Some(context) = browser {
        if context.page_title.is_none() {
            context.page_title = window_title.and_then(|value| non_empty_string(value.to_string()));
        }
    }
}

fn build_summary(
    app: &AppInfo,
    window_title: Option<&str>,
    browser: Option<&BrowserContext>,
    presence: PresenceState,
) -> ActivitySummary {
    ActivitySummary {
        current_label: activity_label(app, window_title, browser, presence),
        status_line: summary_status_line(app, window_title, browser, presence),
        detail_line: summary_detail_line(app, window_title, browser, presence),
        domain_key: domain_key(browser),
        page_key: page_key(browser, window_title),
        app_key: app_key(app),
        app_group: app_group(app, browser, presence),
        presence: presence_label(presence),
        is_browser: is_browser_app(app, browser),
        is_active: presence == PresenceState::Active,
        is_idle: presence == PresenceState::Idle,
        is_locked: presence == PresenceState::Locked,
    }
}

fn build_meta(
    app: &AppInfo,
    window_title: Option<&str>,
    browser: Option<&BrowserContext>,
    presence: PresenceState,
) -> ActivityMeta {
    let now_local = Local::now();

    ActivityMeta {
        app_key: app_key(app),
        app_display_name: app.name.clone(),
        app_group: app_group(app, browser, presence),
        is_browser: is_browser_app(app, browser),
        is_idle: presence == PresenceState::Idle,
        is_locked: presence == PresenceState::Locked,
        is_active: presence == PresenceState::Active,
        presence_label: presence_label(presence),
        activity_label: activity_label(app, window_title, browser, presence),
        window_or_page_title: window_or_page_title(window_title, browser),
        browser_family: browser.map(|context| context.family.clone()),
        browser_name: browser.map(|context| context.name.clone()),
        browser_domain: browser.and_then(|context| context.domain.clone()),
        browser_url: browser.and_then(|context| context.url.clone()),
        browser_page_title: browser.and_then(|context| context.page_title.clone()),
        local_date: now_local.format("%Y-%m-%d").to_string(),
        local_time: now_local.format("%H:%M:%S").to_string(),
        local_datetime: now_local.format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

fn summary_status_line(
    app: &AppInfo,
    window_title: Option<&str>,
    browser: Option<&BrowserContext>,
    presence: PresenceState,
) -> String {
    if presence == PresenceState::Locked {
        return "屏幕已锁定".to_string();
    }
    if presence == PresenceState::Idle {
        return "当前空闲中".to_string();
    }

    if let Some(browser) = browser {
        if let Some(domain) = browser.domain.as_deref() {
            return format!("正在使用 {} 浏览 {}", app.name, domain);
        }
        if let Some(page_title) = browser.page_title.as_deref() {
            return format!("正在使用 {} 查看 {}", app.name, page_title);
        }
    }

    if let Some(title) = clean_str(window_title) {
        return format!("正在使用 {} · {}", app.name, title);
    }

    format!("正在使用 {}", app.name)
}

fn summary_detail_line(
    app: &AppInfo,
    window_title: Option<&str>,
    browser: Option<&BrowserContext>,
    presence: PresenceState,
) -> Option<String> {
    if presence == PresenceState::Locked {
        return Some("屏幕已锁定".to_string());
    }
    if presence == PresenceState::Idle {
        return Some("当前空闲中".to_string());
    }

    if let Some(browser) = browser {
        if let Some(page_title) = browser.page_title.as_deref() {
            return Some(page_title.to_string());
        }
        if let Some(domain) = browser.domain.as_deref() {
            return Some(domain.to_string());
        }
    }

    clean_str(window_title).map(|title| {
        if title == app.name {
            title.to_string()
        } else {
            format!("{} · {}", app.name, title)
        }
    })
}

fn activity_label(
    app: &AppInfo,
    window_title: Option<&str>,
    browser: Option<&BrowserContext>,
    presence: PresenceState,
) -> String {
    if presence == PresenceState::Locked {
        return "Locked Screen".to_string();
    }
    if presence == PresenceState::Idle {
        return "Idle".to_string();
    }

    if let Some(browser) = browser {
        if let Some(domain) = browser.domain.as_deref() {
            return format!("{} · {}", app.name, domain);
        }
        if let Some(page_title) = browser.page_title.as_deref() {
            return format!("{} · {}", app.name, page_title);
        }
    }

    if let Some(title) = clean_str(window_title) {
        return format!("{} · {}", app.name, title);
    }

    app.name.clone()
}

fn window_or_page_title(window_title: Option<&str>, browser: Option<&BrowserContext>) -> Option<String> {
    browser
        .and_then(|context| context.page_title.clone())
        .or_else(|| clean_str(window_title).map(ToString::to_string))
}

fn domain_key(browser: Option<&BrowserContext>) -> Option<String> {
    browser
        .and_then(|context| context.domain.clone())
        .and_then(non_empty_lowercase_string)
}

fn page_key(browser: Option<&BrowserContext>, window_title: Option<&str>) -> Option<String> {
    browser
        .and_then(|context| context.url.clone())
        .or_else(|| browser.and_then(|context| context.page_title.clone()))
        .or_else(|| clean_str(window_title).map(ToString::to_string))
        .and_then(non_empty_string)
}

fn app_key(app: &AppInfo) -> String {
    if !app.id.trim().is_empty() {
        app.id.clone()
    } else {
        app.name.trim().to_ascii_lowercase()
    }
}

fn presence_label(presence: PresenceState) -> &'static str {
    match presence {
        PresenceState::Active => "active",
        PresenceState::Idle => "idle",
        PresenceState::Locked => "locked",
    }
}

fn app_group(app: &AppInfo, browser: Option<&BrowserContext>, presence: PresenceState) -> String {
    if presence == PresenceState::Locked {
        return "locked".to_string();
    }
    if presence == PresenceState::Idle {
        return "idle".to_string();
    }
    if browser.is_some() {
        return "browser".to_string();
    }

    let app_id = app.id.to_ascii_lowercase();
    let app_name = app.name.to_ascii_lowercase();

    if KNOWN_SYSTEM_APPS.iter().any(|candidate| app_id.contains(candidate)) {
        return "system".to_string();
    }
    if KNOWN_COMM_APPS
        .iter()
        .any(|candidate| app_id.contains(candidate) || app_name.contains(candidate))
    {
        return "communication".to_string();
    }
    if KNOWN_WORK_APPS
        .iter()
        .any(|candidate| app_id.contains(candidate) || app_name.contains(candidate))
    {
        return "work".to_string();
    }

    "app".to_string()
}

fn is_browser_app(app: &AppInfo, browser: Option<&BrowserContext>) -> bool {
    if browser.is_some() {
        return true;
    }

    let app_id = app.id.to_ascii_lowercase();
    let app_name = app.name.to_ascii_lowercase();
    KNOWN_BROWSER_IDS
        .iter()
        .any(|candidate| app_id.contains(candidate) || app_name.contains(candidate))
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn new_event_id() -> String {
    Uuid::new_v4().to_string()
}

fn normalize_agent_name(agent_name: &str) -> String {
    agent_name.trim().to_string()
}

fn normalize_device_id(device_id: &str) -> String {
    device_id.trim().to_string()
}

fn clean_str<'a>(value: Option<&'a str>) -> Option<&'a str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn non_empty_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn non_empty_lowercase_string(value: String) -> Option<String> {
    non_empty_string(value).map(|value| value.to_ascii_lowercase())
}

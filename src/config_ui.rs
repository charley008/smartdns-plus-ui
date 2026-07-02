use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MANAGED_BEGIN: &str = "# BEGIN smartdns-plus-ui managed";
const MANAGED_END: &str = "# END smartdns-plus-ui managed";
const BASIC_FILE_NAME: &str = "plus-ui-basic.conf";
const UPSTREAMS_FILE_NAME: &str = "plus-ui-upstreams.conf";
const RULES_FILE_NAME: &str = "plus-ui-rules.conf";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BeginnerConfig {
    pub server_name: String,
    pub binds: Vec<String>,
    pub cache_size: String,
    pub prefetch_domain: bool,
    pub serve_expired: bool,
    pub serve_expired_ttl: String,
    pub response_mode: String,
    pub speed_check_mode: String,
    pub dualstack_ip_selection: bool,
    pub log_level: String,
    pub mdns_lookup: bool,
    pub audit_enable: bool,
    pub upstreams: Vec<UpstreamConfig>,
    #[serde(default)]
    pub extras: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpstreamConfig {
    pub kind: String,
    pub value: String,
    pub groups: Vec<String>,
    pub exclude_default_group: bool,
    pub proxy_name: String,
    pub bootstrap_dns: String,
    pub host_ip: String,
    pub subnet: String,
    pub fallback: bool,
    pub options: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveConfigRequest {
    pub beginner: BeginnerConfig,
    pub rules_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationItem {
    pub key: String,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub caution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFileInfo {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoutingRuleItem {
    pub rule_type: String,
    pub target: String,
    pub value: String,
    pub source: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleSetAsset {
    pub name: String,
    pub set_type: String,
    pub file: String,
    pub source: String,
    pub rule_count: usize,
    pub sample_items: Vec<String>,
    pub referenced_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleSetFileDetail {
    pub name: String,
    pub set_type: String,
    pub file: String,
    pub source: String,
    pub exists: bool,
    pub rule_count: usize,
    pub filtered_count: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
    pub items: Vec<String>,
    pub sample_items: Vec<String>,
    pub referenced_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleGroupAsset {
    pub name: String,
    pub inherit: String,
    pub source: String,
    pub include_file: String,
    pub matchers: Vec<String>,
    pub rule_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientRuleAsset {
    pub matcher: String,
    pub group_name: String,
    pub source: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IpRuleAsset {
    pub rule_type: String,
    pub target: String,
    pub options: String,
    pub group_name: String,
    pub source: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IpSetAsset {
    pub name: String,
    pub file: String,
    pub source: String,
    pub rule_count: usize,
    pub sample_items: Vec<String>,
    pub referenced_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub path: String,
    pub modified_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveRuleSetFileRequest {
    pub add_items: Vec<String>,
    pub remove_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManagedRuleGroup {
    pub name: String,
    pub inherit: String,
    pub matchers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManagedClientRule {
    pub matcher: String,
    pub group_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManagedIpRule {
    pub rule_type: String,
    pub target: String,
    pub options: String,
    pub group_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManagedRoutingItem {
    pub rule_type: String,
    pub target: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManagedDomainSet {
    pub name: String,
    pub file: String,
    pub set_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManagedIpSet {
    pub name: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManagedAssets {
    pub rule_groups: Vec<ManagedRuleGroup>,
    pub client_rules: Vec<ManagedClientRule>,
    pub ip_rules: Vec<ManagedIpRule>,
    pub routing_items: Vec<ManagedRoutingItem>,
    pub domain_sets: Vec<ManagedDomainSet>,
    pub ip_sets: Vec<ManagedIpSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveManagedAssetsRequest {
    pub rule_groups: Vec<ManagedRuleGroup>,
    pub client_rules: Vec<ManagedClientRule>,
    pub ip_rules: Vec<ManagedIpRule>,
    pub routing_items: Vec<ManagedRoutingItem>,
    pub domain_sets: Vec<ManagedDomainSet>,
    pub ip_sets: Vec<ManagedIpSet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub ok: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOverview {
    pub config_file: String,
    pub rules_dir: String,
    pub managed_block_present: bool,
    pub beginner: BeginnerConfig,
    pub main_config_text: String,
    pub managed_basic_text: String,
    pub managed_upstreams_text: String,
    pub managed_rules_text: String,
    pub detected_rules_text: String,
    pub routing_items: Vec<RoutingRuleItem>,
    pub rule_set_assets: Vec<RuleSetAsset>,
    pub rule_group_assets: Vec<RuleGroupAsset>,
    pub client_rule_assets: Vec<ClientRuleAsset>,
    pub ip_rule_assets: Vec<IpRuleAsset>,
    pub ip_set_assets: Vec<IpSetAsset>,
    pub explanations: Vec<ExplanationItem>,
    pub recommendations: Vec<String>,
    pub rule_files: Vec<RuleFileInfo>,
    pub validation: ValidationResult,
    pub managed_assets: ManagedAssets,
    pub working_config_text: String,
}

pub fn config_file_from_settings(config_file: Option<String>) -> String {
    config_file.unwrap_or_else(|| "/etc/smartdns/smartdns.conf".to_string())
}

pub fn rules_dir_from_settings(rules_dir: Option<String>) -> String {
    rules_dir.unwrap_or_else(|| "/etc/smartdns/rules".to_string())
}

pub fn read_overview(config_file: &str, rules_dir: &str) -> io::Result<ConfigOverview> {
    let main_config_text = read_text_or_default(config_file)?;
    let managed_block_present = main_config_text.contains(MANAGED_BEGIN);
    let managed_dir = managed_conf_dir(rules_dir);
    let managed_basic_text = read_text_or_default(&path_join(&managed_dir, BASIC_FILE_NAME))?;
    let managed_upstreams_text =
        read_text_or_default(&path_join(&managed_dir, UPSTREAMS_FILE_NAME))?;
    let managed_rules_text = read_text_or_default(&path_join(&managed_dir, RULES_FILE_NAME))?;
    let config_sources = collect_config_sources(config_file)?;
    let routing_items = parse_routing_items(&config_sources);
    let rule_set_assets = build_rule_set_assets(&routing_items);
    let rule_group_assets = build_rule_group_assets(&config_sources);
    let client_rule_assets = build_client_rule_assets(&config_sources);
    let ip_rule_assets = build_ip_rule_assets(&config_sources);
    let ip_set_assets = build_ip_set_assets(&config_sources, &rule_group_assets, &ip_rule_assets);
    let detected_rules_text = render_detected_rules_text(&routing_items);
    let managed_assets = parse_managed_assets_from_text(&managed_rules_text);
    let working_config_text =
        render_effective_config_preview(&main_config_text, rules_dir).unwrap_or_default();

    let aggregate = if !working_config_text.trim().is_empty() {
        working_config_text.clone()
    } else if !managed_basic_text.trim().is_empty() || !managed_upstreams_text.trim().is_empty() {
        format!("{}\n{}", managed_basic_text, managed_upstreams_text)
    } else {
        main_config_text.clone()
    };

    let beginner = parse_beginner_config(&aggregate);
    let validation = validate_beginner_config(&beginner, &managed_rules_text);

    Ok(ConfigOverview {
        config_file: config_file.to_string(),
        rules_dir: rules_dir.to_string(),
        managed_block_present,
        beginner: beginner.clone(),
        main_config_text,
        managed_basic_text,
        managed_upstreams_text,
        managed_rules_text: managed_rules_text.clone(),
        detected_rules_text,
        routing_items,
        rule_set_assets,
        rule_group_assets,
        client_rule_assets,
        ip_rule_assets,
        ip_set_assets,
        explanations: explanation_items(),
        recommendations: recommendations_for(&beginner),
        rule_files: vec![
            rule_file_info(&managed_dir, BASIC_FILE_NAME)?,
            rule_file_info(&managed_dir, UPSTREAMS_FILE_NAME)?,
            rule_file_info(&managed_dir, RULES_FILE_NAME)?,
        ],
        validation,
        managed_assets,
        working_config_text,
    })
}

pub fn save_beginner_config(
    config_file: &str,
    rules_dir: &str,
    request: &SaveConfigRequest,
) -> io::Result<ValidationResult> {
    let validation = validate_beginner_config(&request.beginner, &request.rules_text);
    if !validation.ok {
        return Ok(validation);
    }

    let basic_text = render_basic_config(&request.beginner);
    let upstreams_text = render_upstreams_config(&request.beginner);
    let rules_text = normalize_rules_text(&request.rules_text);

    let managed_dir = managed_conf_dir(rules_dir);
    write_text_atomic(&path_join(&managed_dir, BASIC_FILE_NAME), &basic_text)?;
    write_text_atomic(&path_join(&managed_dir, UPSTREAMS_FILE_NAME), &upstreams_text)?;
    write_text_atomic(&path_join(&managed_dir, RULES_FILE_NAME), &rules_text)?;

    let current = read_text_or_default(config_file)?;
    let new_main = ensure_managed_block(&current, rules_dir);
    if new_main != current {
        backup_file(config_file)?;
        write_text_atomic(config_file, &new_main)?;
    }

    Ok(validation)
}

/// Apply include config: ensure main config still references managed include files.
pub fn apply_working_config(config_file: &str, rules_dir: &str) -> io::Result<()> {
    let current = read_text_or_default(config_file)?;
    let new_main = ensure_managed_block(&current, rules_dir);
    if new_main != current {
        backup_file(config_file)?;
        write_text_atomic(config_file, &new_main)?;
    }
    Ok(())
}

pub fn list_backups(config_file: &str) -> io::Result<Vec<BackupInfo>> {
    let config_path = PathBuf::from(config_file);
    let parent = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let file_name = config_path
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("smartdns.conf");
    let prefix = format!("{}.smartdns-plus-ui.bak.", file_name);
    let mut backups = Vec::new();

    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let path = entry.path();
        let name = match path.file_name().and_then(|x| x.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if !name.starts_with(&prefix) {
            continue;
        }
        let modified_unix = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        backups.push(BackupInfo {
            path: path.to_string_lossy().to_string(),
            modified_unix,
        });
    }

    backups.sort_by(|a, b| b.modified_unix.cmp(&a.modified_unix));
    Ok(backups)
}

pub fn restore_backup(config_file: &str, backup_path: &str) -> io::Result<()> {
    let backup_text = fs::read_to_string(backup_path)?;
    backup_file(config_file)?;
    write_text_atomic(config_file, &backup_text)
}

pub fn read_rule_set_detail(
    config_file: &str,
    name: &str,
    query: Option<&str>,
    offset: usize,
    limit: usize,
) -> io::Result<Option<RuleSetFileDetail>> {
    let config_sources = collect_config_sources(config_file)?;
    let routing_items = parse_routing_items(&config_sources);
    let rule_set_assets = build_rule_set_assets(&routing_items);

    let Some(asset) = rule_set_assets.into_iter().find(|candidate| candidate.name == name) else {
        return Ok(None);
    };

    let all_items = read_rule_set_items(&asset.file)?;
    let query = query.unwrap_or("").trim().to_lowercase();
    let filtered: Vec<String> = if query.is_empty() {
        all_items.clone()
    } else {
        all_items
            .iter()
            .filter(|item| item.to_lowercase().contains(&query))
            .cloned()
            .collect()
    };
    let limit = limit.max(1);
    let items: Vec<String> = filtered.iter().skip(offset).take(limit).cloned().collect();

    Ok(Some(RuleSetFileDetail {
        name: asset.name,
        set_type: asset.set_type,
        file: asset.file.clone(),
        source: asset.source,
        exists: Path::new(&asset.file).exists(),
        rule_count: all_items.len(),
        filtered_count: filtered.len(),
        offset,
        limit,
        has_more: offset.saturating_add(items.len()) < filtered.len(),
        items,
        sample_items: asset.sample_items,
        referenced_by: asset.referenced_by,
    }))
}

pub fn save_rule_set_file(
    config_file: &str,
    name: &str,
    request: &SaveRuleSetFileRequest,
) -> io::Result<RuleSetFileDetail> {
    let detail = read_rule_set_detail(config_file, name, None, 0, 200)?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("rule set `{}` not found", name),
        )
    })?;

    if detail.file.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("rule set `{}` has no backing file", name),
        ));
    }

    let mut items = read_rule_set_items(&detail.file).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("read rule set file {} failed: {}", detail.file, err),
        )
    })?;
    let remove_items = normalize_rule_set_items(&request.remove_items);
    let add_items = normalize_rule_set_items(&request.add_items);

    if !remove_items.is_empty() {
        items.retain(|item| !remove_items.contains(item));
    }
    items.extend(add_items);
    items = dedupe_rule_set_items(items);

    backup_file(&detail.file).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("backup rule set file {} failed: {}", detail.file, err),
        )
    })?;
    write_text_atomic(&detail.file, &render_rule_set_items(&items)).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("write rule set file {} failed: {}", detail.file, err),
        )
    })?;

    read_rule_set_detail(config_file, name, None, 0, 200)?.ok_or_else(|| {
        io::Error::new(
            ErrorKind::NotFound,
            format!("rule set `{}` disappeared after save", name),
        )
    })
}

fn rule_file_info(rules_dir: &str, name: &str) -> io::Result<RuleFileInfo> {
    let path = path_join(rules_dir, name);
    Ok(RuleFileInfo {
        name: name.to_string(),
        exists: Path::new(&path).exists(),
        content: read_text_or_default(&path)?,
        path,
    })
}

fn read_text_or_default(path: &str) -> io::Result<String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(err) => Err(err),
    }
}

#[derive(Debug, Clone)]
struct ConfigSource {
    path: String,
    text: String,
    implicit_group: Option<String>,
}

fn path_join(dir: &str, file_name: &str) -> String {
    Path::new(dir).join(file_name).to_string_lossy().to_string()
}

fn managed_conf_dir(rules_dir: &str) -> String {
    let rules_path = Path::new(rules_dir);
    let parent = rules_path.parent().unwrap_or_else(|| Path::new(rules_dir));
    parent.join("conf.d").to_string_lossy().to_string()
}

fn write_text_atomic(path: &str, content: &str) -> io::Result<()> {
    let target = Path::new(path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = format!("{}.tmp", path);
    fs::write(&tmp, content)?;
    if let Err(err) = fs::rename(&tmp, path) {
        if err.kind() == io::ErrorKind::PermissionDenied {
            // Some bind-mounted filesystems reject atomic rename-overwrite even
            // though direct writes succeed. Fall back to a non-atomic overwrite
            // so the UI can still save managed files in Docker-on-WSL setups.
            fs::write(path, content)?;
            let _ = fs::remove_file(&tmp);
        } else {
            let _ = fs::remove_file(&tmp);
            return Err(err);
        }
    }
    Ok(())
}

fn backup_file(path: &str) -> io::Result<()> {
    let target = Path::new(path);
    if !target.exists() {
        return Ok(());
    }
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let file_name = target
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("smartdns.conf");
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup_name = format!("{}.smartdns-plus-ui.bak.{}", file_name, ts);
    let backup_path = parent.join(backup_name);
    fs::copy(target, backup_path)?;
    Ok(())
}

fn normalize_rules_text(text: &str) -> String {
    let text = text.replace("\r\n", "\n");
    if text.trim().is_empty() {
        return default_rules_text();
    }
    if text.ends_with('\n') {
        text
    } else {
        format!("{}\n", text)
    }
}

fn render_basic_config(config: &BeginnerConfig) -> String {
    let mut lines = Vec::new();
    lines.push("# ═══════════════════════════════════════════".to_string());
    lines.push("#  SmartDNS 基础配置".to_string());
    lines.push("#  由 SmartDNS Plus UI 生成".to_string());
    lines.push("# ═══════════════════════════════════════════".to_string());
    lines.push(String::new());

    // Service
    if !config.server_name.trim().is_empty() {
        lines.push(format!("server-name {}", config.server_name.trim()));
    }
    let binds = if config.binds.is_empty() {
        vec!["[::]:53".to_string()]
    } else {
        config.binds.clone()
    };
    for bind in binds {
        let trimmed = bind.trim();
        if !trimmed.is_empty() {
            lines.push(format!("bind {}", trimmed));
        }
    }
    // bind-tcp from extras, render right after bind
    if let Some(bt) = config.extras.get("bind-tcp") {
        let v = bt.trim();
        if !v.is_empty() {
            lines.push(format!("bind-tcp {}", v));
        }
    }

    // Cache & performance
    lines.push(String::new());
    lines.push("# ── 缓存 ──".to_string());
    push_kv(&mut lines, "cache-size", &config.cache_size, "32768");
    push_bool(&mut lines, "prefetch-domain", config.prefetch_domain);
    push_bool(&mut lines, "serve-expired", config.serve_expired);
    push_kv(&mut lines, "serve-expired-ttl", &config.serve_expired_ttl, "86400");

    // Response & speed
    lines.push(String::new());
    lines.push("# ── 响应与测速 ──".to_string());
    push_kv(&mut lines, "response-mode", &config.response_mode, "first-ping");
    push_kv(&mut lines, "speed-check-mode", &config.speed_check_mode, "ping,tcp:80,tcp:443");
    push_bool(&mut lines, "dualstack-ip-selection", config.dualstack_ip_selection);

    // Logging & audit
    lines.push(String::new());
    lines.push("# ── 日志与审计 ──".to_string());
    push_kv(&mut lines, "log-level", &config.log_level, "info");
    push_bool(&mut lines, "mdns-lookup", config.mdns_lookup);
    push_bool(&mut lines, "audit-enable", config.audit_enable);

    // Extras
    let skip_prefixes = ["smartdns-plus-ui.", "smartdns-ui."];
    let skip_exact = ["plugin", "data-dir", "bind-tcp"];
    let mut extra_keys: Vec<&String> = config.extras.keys().collect();
    extra_keys.sort();
    let has_extras = extra_keys.iter().any(|k| {
        !skip_prefixes.iter().any(|p| k.starts_with(p)) && !skip_exact.contains(&k.as_str())
    });
    if has_extras {
        lines.push(String::new());
        lines.push("# ── 其他设置 ──".to_string());
        for key in extra_keys {
            if skip_prefixes.iter().any(|p| key.starts_with(p)) { continue; }
            if skip_exact.contains(&key.as_str()) { continue; }
            if let Some(value) = config.extras.get(key) {
                let v = value.trim();
                if !v.is_empty() {
                    lines.push(format!("{} {}", key, v));
                }
            }
        }
    }

    lines.push(String::new());
    lines.join("\n")
}

fn render_upstreams_config(config: &BeginnerConfig) -> String {
    let mut lines = Vec::new();
    lines.push("# ═══════════════════════════════════════════".to_string());
    lines.push("#  上游 DNS 服务器".to_string());
    lines.push("# ═══════════════════════════════════════════".to_string());
    lines.push(String::new());

    let upstreams = if config.upstreams.is_empty() {
        default_upstreams()
    } else {
        config.upstreams.clone()
    };

    // Group by first group name
    let mut grouped: BTreeMap<String, Vec<&UpstreamConfig>> = BTreeMap::new();
    let mut ungrouped = Vec::new();
    for u in &upstreams {
        if u.value.trim().is_empty() { continue; }
        if let Some(first) = u.groups.first() {
            grouped.entry(first.clone()).or_default().push(u);
        } else {
            ungrouped.push(u);
        }
    }

    for (gname, servers) in &grouped {
        lines.push(format!("# ── 分组: {} ──", gname));
        for u in servers {
            let key = upstream_key(&u.kind);
            let opts = render_upstream_options(u);
            if opts.trim().is_empty() {
                lines.push(format!("{} {}", key, u.value.trim()));
            } else {
                lines.push(format!("{} {} {}", key, u.value.trim(), opts));
            }
        }
        lines.push(String::new());
    }

    if !ungrouped.is_empty() {
        lines.push("# ── 默认组（未分组）──".to_string());
        for u in &ungrouped {
            let key = upstream_key(&u.kind);
            let opts = render_upstream_options(u);
            if opts.trim().is_empty() {
                lines.push(format!("{} {}", key, u.value.trim()));
            } else {
                lines.push(format!("{} {} {}", key, u.value.trim(), opts));
            }
        }
        lines.push(String::new());
    }

    lines.push(String::new());
    lines.join("\n")
}

fn upstream_key(kind: &str) -> &str {
    match kind.trim() {
        "server" | "udp" => "server",
        "server-tcp" | "tcp" => "server-tcp",
        "server-tls" | "tls" => "server-tls",
        "server-https" | "https" => "server-https",
        "server-quic" | "quic" => "server-quic",
        "server-http3" | "http3" | "h3" => "server-http3",
        _ => "server",
    }
}

fn render_upstream_options(upstream: &UpstreamConfig) -> String {
    let mut parts = Vec::new();
    for group in &upstream.groups {
        let trimmed = group.trim();
        if !trimmed.is_empty() {
            parts.push(format!("-group {}", trimmed));
        }
    }
    if upstream.exclude_default_group {
        parts.push("-exclude-default-group".to_string());
    }
    if !upstream.proxy_name.trim().is_empty() {
        parts.push(format!("-proxy {}", upstream.proxy_name.trim()));
    }
    if !upstream.bootstrap_dns.trim().is_empty() {
        parts.push(format!("-bootstrap-dns {}", upstream.bootstrap_dns.trim()));
    }
    if !upstream.host_ip.trim().is_empty() {
        parts.push(format!("-host-ip {}", upstream.host_ip.trim()));
    }
    if !upstream.subnet.trim().is_empty() {
        parts.push(format!("-subnet {}", upstream.subnet.trim()));
    }
    if upstream.fallback {
        parts.push("-fallback".to_string());
    }
    if !upstream.options.trim().is_empty() {
        parts.push(upstream.options.trim().to_string());
    }
    parts.join(" ")
}

fn ensure_managed_block(main_config_text: &str, rules_dir: &str) -> String {
    let managed_dir = managed_conf_dir(rules_dir);
    let mut include_lines = Vec::new();
    include_lines.push(MANAGED_BEGIN.to_string());
    include_lines.push(format!("conf-file {}", path_join(&managed_dir, BASIC_FILE_NAME)));
    include_lines.push(format!("conf-file {}", path_join(&managed_dir, UPSTREAMS_FILE_NAME)));
    include_lines.push(format!("conf-file {}", path_join(&managed_dir, RULES_FILE_NAME)));
    include_lines.push(MANAGED_END.to_string());
    let block = include_lines.join("\n");

    if let Some(start) = main_config_text.find(MANAGED_BEGIN) {
        if let Some(end_rel) = main_config_text[start..].find(MANAGED_END) {
            let end = start + end_rel + MANAGED_END.len();
            let before = &main_config_text[..start];
            let after = if end < main_config_text.len() {
                &main_config_text[end..]
            } else {
                ""
            };
            let mut merged = before.trim_end().to_string();
            if !merged.is_empty() {
                merged.push_str("\n\n");
            }
            merged.push_str(&block);
            if !after.trim().is_empty() {
                merged.push('\n');
                if !after.starts_with('\n') {
                    merged.push('\n');
                }
                merged.push_str(after);
            } else {
                merged.push('\n');
            }
            return merged;
        }
    }

    let mut merged = main_config_text.trim_end().to_string();
    if !merged.is_empty() {
        merged.push_str("\n\n");
    }
    merged.push_str(&block);
    merged.push('\n');
    merged
}

fn render_effective_config_preview(main_config_text: &str, rules_dir: &str) -> io::Result<String> {
    let managed_dir = managed_conf_dir(rules_dir);
    let basic = read_text_or_default(&path_join(&managed_dir, BASIC_FILE_NAME))?;
    let upstreams = read_text_or_default(&path_join(&managed_dir, UPSTREAMS_FILE_NAME))?;
    let rules = read_text_or_default(&path_join(&managed_dir, RULES_FILE_NAME))?;

    let mut lines = Vec::new();
    let mut in_managed_block = false;

    for line in main_config_text.lines() {
        let trimmed = line.trim();
        if trimmed == MANAGED_BEGIN {
            in_managed_block = true;
            lines.push(line.to_string());

            if !basic.trim().is_empty() {
                lines.push(String::new());
                lines.extend(basic.lines().map(|item| item.to_string()));
            }
            if !upstreams.trim().is_empty() {
                if !basic.trim().is_empty() {
                    lines.push(String::new());
                }
                lines.extend(upstreams.lines().map(|item| item.to_string()));
            }
            if !rules.trim().is_empty() {
                if !basic.trim().is_empty() || !upstreams.trim().is_empty() {
                    lines.push(String::new());
                }
                lines.extend(rules.lines().map(|item| item.to_string()));
            }
            continue;
        }

        if in_managed_block {
            if trimmed == MANAGED_END {
                lines.push(line.to_string());
                in_managed_block = false;
            }
            continue;
        }

        lines.push(line.to_string());
    }

    if !lines.is_empty() {
        lines.push(String::new());
    }
    Ok(lines.join("\n"))
}

fn parse_beginner_config(text: &str) -> BeginnerConfig {
    let mut cfg = BeginnerConfig {
        cache_size: "32768".to_string(),
        serve_expired_ttl: "86400".to_string(),
        response_mode: "first-ping".to_string(),
        speed_check_mode: "ping,tcp:80,tcp:443".to_string(),
        log_level: "info".to_string(),
        prefetch_domain: true,
        serve_expired: true,
        dualstack_ip_selection: true,
        ..BeginnerConfig::default()
    };

    for line in logical_lines(text) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let key = parts.next().unwrap_or("").trim();
        let value = parts.next().unwrap_or("").trim();
        match key {
            "server-name" => cfg.server_name = value.to_string(),
            "bind" => cfg.binds.push(value.to_string()),
            "cache-size" => cfg.cache_size = value.to_string(),
            "prefetch-domain" => cfg.prefetch_domain = parse_yesno(value, true),
            "serve-expired" => cfg.serve_expired = parse_yesno(value, true),
            "serve-expired-ttl" => cfg.serve_expired_ttl = value.to_string(),
            "response-mode" => cfg.response_mode = value.to_string(),
            "speed-check-mode" => cfg.speed_check_mode = value.to_string(),
            "dualstack-ip-selection" => cfg.dualstack_ip_selection = parse_yesno(value, true),
            "log-level" => cfg.log_level = value.to_string(),
            "mdns-lookup" => cfg.mdns_lookup = parse_yesno(value, false),
            "audit-enable" => cfg.audit_enable = parse_yesno(value, false),
            "server" | "server-tcp" | "server-tls" | "server-https" | "server-quic" | "server-http3" | "server-h3" => {
                let mut value_parts = value.splitn(2, char::is_whitespace);
                let server_value = value_parts.next().unwrap_or("").trim().to_string();
                let server_opts = value_parts.next().unwrap_or("").trim().to_string();
                cfg.upstreams.push(parse_upstream_config(key, &server_value, &server_opts));
            }
            _ => {
                // Collect unknown directives into extras (covers ALL official config)
                if !value.is_empty() {
                    cfg.extras.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    cfg
}

fn collect_config_sources(config_file: &str) -> io::Result<Vec<ConfigSource>> {
    let mut items = Vec::new();
    let mut visited = HashSet::new();
    collect_config_source_recursive(Path::new(config_file), None, &mut visited, &mut items)?;
    Ok(items)
}

fn collect_config_source_recursive(
    path: &Path,
    implicit_group: Option<String>,
    visited: &mut HashSet<String>,
    items: &mut Vec<ConfigSource>,
) -> io::Result<()> {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let visit_key = format!(
        "{}::{}",
        canonical.to_string_lossy(),
        implicit_group.clone().unwrap_or_default()
    );
    if !visited.insert(visit_key) {
        return Ok(());
    }

    let text = read_text_or_default(&canonical.to_string_lossy())?;
    items.push(ConfigSource {
        path: canonical.to_string_lossy().to_string(),
        text: text.clone(),
        implicit_group: implicit_group.clone(),
    });

    for line in logical_lines(&text) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let key = parts.next().unwrap_or("").trim();
        let value = parts.next().unwrap_or("").trim();
        if key != "conf-file" || value.is_empty() {
            continue;
        }

        let include_file = value.split_whitespace().next().unwrap_or("").trim();
        let include_group = parse_named_option(value, "-group");
        let include_path = resolve_include_path(&canonical, include_file);
        if include_path.exists() {
            collect_config_source_recursive(&include_path, include_group, visited, items)?;
        }
    }

    Ok(())
}

fn resolve_include_path(base_file: &Path, include_value: &str) -> PathBuf {
    let include_path = PathBuf::from(include_value);
    if include_path.is_absolute() {
        include_path
    } else {
        base_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(include_path)
    }
}

fn logical_lines(text: &str) -> Vec<String> {
    let mut merged = Vec::new();
    let mut current = String::new();

    for raw_line in text.lines() {
        let trimmed_end = raw_line.trim_end();
        let trimmed = trimmed_end.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed_end.ends_with('\\') {
            let piece = trimmed_end.trim_end_matches('\\').trim_end();
            if !piece.is_empty() {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(piece.trim());
            }
            continue;
        }

        if !trimmed.is_empty() {
            if !current.is_empty() {
                current.push(' ');
                current.push_str(trimmed);
                merged.push(current.trim().to_string());
                current.clear();
            } else {
                merged.push(trimmed.to_string());
            }
        } else if !current.is_empty() {
            merged.push(current.trim().to_string());
            current.clear();
        }
    }

    if !current.is_empty() {
        merged.push(current.trim().to_string());
    }

    merged
}

fn parse_routing_items(sources: &[ConfigSource]) -> Vec<RoutingRuleItem> {
    let mut items = Vec::new();

    for source in sources {
        for line in logical_lines(&source.text) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let mut parts = trimmed.splitn(2, char::is_whitespace);
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();
            if value.is_empty() {
                continue;
            }

            match key {
                "nameserver" => {
                    let (target, server_group) = split_target_value(value);
                    items.push(RoutingRuleItem {
                        rule_type: "nameserver".to_string(),
                        target,
                        value: server_group,
                        source: source.path.clone(),
                        note: describe_target_hint(value, "把域名或规则集导向指定上游组"),
                    });
                }
                "domain-rules" => {
                    let (target, options) = split_target_options(value);
                    items.push(RoutingRuleItem {
                        rule_type: "domain-rules".to_string(),
                        target,
                        value: options,
                        source: source.path.clone(),
                        note: "给域名或规则集附加行为，例如禁缓存、禁 IPv6、改测速".to_string(),
                    });
                }
                "address" => {
                    let (target, address) = split_target_value(value);
                    items.push(RoutingRuleItem {
                        rule_type: "address".to_string(),
                        target,
                        value: address,
                        source: source.path.clone(),
                        note: "给域名或规则集直接返回固定地址或空地址".to_string(),
                    });
                }
                "cname" => {
                    let (target, cname) = split_target_value(value);
                    items.push(RoutingRuleItem {
                        rule_type: "cname".to_string(),
                        target,
                        value: cname,
                        source: source.path.clone(),
                        note: "把查询结果改写为指定 CNAME".to_string(),
                    });
                }
                "domain-set" => {
                    if let Some(name) = parse_named_option(value, "-name") {
                        let set_type =
                            parse_named_option(value, "-type").unwrap_or_else(|| "list".to_string());
                        let file = parse_named_option(value, "-file").unwrap_or_default();
                        items.push(RoutingRuleItem {
                            rule_type: "domain-set".to_string(),
                            target: name,
                            value: format!("type={} file={}", set_type, file),
                            source: source.path.clone(),
                            note: "定义一个可被 nameserver 或 domain-rules 引用的域名集合".to_string(),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    items
}

fn split_target_value(value: &str) -> (String, String) {
    let trimmed = value.trim();
    if let Some(rest) = trimmed.strip_prefix('/') {
        if let Some(pos) = rest.rfind('/') {
            let target = format!("/{}", &rest[..pos]);
            let tail = rest[pos + 1..].trim();
            return (target, tail.to_string());
        }
    }

    split_target_options(value)
}

fn split_target_options(value: &str) -> (String, String) {
    let trimmed = value.trim();
    if let Some(rest) = trimmed.strip_prefix('/') {
        if let Some(pos) = rest.find("/ ") {
            let target = format!("/{}", &rest[..pos]);
            let tail = rest[pos + 2..].trim();
            return (target, tail.to_string());
        }
    }

    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let first = parts.next().unwrap_or("").trim().to_string();
    let second = parts.next().unwrap_or("").trim().to_string();
    (first, second)
}

fn parse_named_option(value: &str, name: &str) -> Option<String> {
    let tokens: Vec<&str> = value.split_whitespace().collect();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index] == name {
            return tokens.get(index + 1).map(|v| (*v).to_string());
        }
        index += 1;
    }
    None
}

fn parse_domain_set_value(value: &str) -> (String, String) {
    let tokens: Vec<&str> = value.split_whitespace().collect();
    let mut set_type = String::from("list");
    let mut file = String::new();
    let mut index = 0usize;
    while index < tokens.len() {
        match tokens[index] {
            "type=" => {}
            token if token.starts_with("type=") => {
                set_type = token.trim_start_matches("type=").to_string();
            }
            token if token.starts_with("file=") => {
                file = token.trim_start_matches("file=").to_string();
            }
            _ => {}
        }
        index += 1;
    }
    (set_type, file)
}

fn build_rule_set_assets(items: &[RoutingRuleItem]) -> Vec<RuleSetAsset> {
    let mut assets = Vec::new();

    for item in items.iter().filter(|x| x.rule_type == "domain-set") {
        let (set_type, file) = parse_domain_set_value(&item.value);
        let (rule_count, sample_items) = inspect_rule_set_file(&file);
        let referenced_by = items
            .iter()
            .filter(|candidate| candidate.rule_type != "domain-set")
            .filter_map(|candidate| {
                let expected = format!("domain-set:{}", item.target);
                if candidate.target.contains(&expected) {
                    Some(format!(
                        "{} -> {}{}",
                        candidate.rule_type,
                        candidate.target,
                        if candidate.value.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", candidate.value)
                        }
                    ))
                } else {
                    None
                }
            })
            .collect();

        assets.push(RuleSetAsset {
            name: item.target.clone(),
            set_type,
            file,
            source: item.source.clone(),
            rule_count,
            sample_items,
            referenced_by,
        });
    }

    assets
}

fn inspect_rule_set_file(path: &str) -> (usize, Vec<String>) {
    if path.trim().is_empty() {
        return (0, Vec::new());
    }

    let items = match read_rule_set_items(path) {
        Ok(items) => items,
        Err(_) => return (0, Vec::new()),
    };
    let sample_items = items.iter().take(5).cloned().collect();
    (items.len(), sample_items)
}

fn read_rule_set_items(path: &str) -> io::Result<Vec<String>> {
    let text = read_text_or_default(path)?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect())
}

fn normalize_rule_set_items(items: &[String]) -> Vec<String> {
    items
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty() && !item.starts_with('#'))
        .map(|item| item.to_string())
        .collect()
}

fn dedupe_rule_set_items(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for item in items {
        if seen.insert(item.clone()) {
            deduped.push(item);
        }
    }
    deduped
}

fn render_rule_set_items(items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }

    let mut text = items.join("\n");
    text.push('\n');
    text
}

#[derive(Debug, Clone)]
struct PendingGroupAsset {
    name: String,
    inherit: String,
    source: String,
    include_file: String,
    matchers: Vec<String>,
    rule_count: usize,
}

fn build_rule_group_assets(sources: &[ConfigSource]) -> Vec<RuleGroupAsset> {
    let mut assets = Vec::new();

    for source in sources {
        for line in logical_lines(&source.text) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut parts = trimmed.splitn(2, char::is_whitespace);
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();

            match key {
                "group-begin" => {
                    let name = value.split_whitespace().next().unwrap_or("").trim().to_string();
                    let inherit = parse_named_option(value, "-inherit").unwrap_or_default();
                    assets.push(RuleGroupAsset {
                        name,
                        inherit,
                        source: source.path.clone(),
                        include_file: String::new(),
                        matchers: Vec::new(),
                        rule_count: 0,
                    });
                }
                "conf-file" => {
                    if let Some(group_name) = parse_named_option(value, "-group") {
                        let include_file = value.split_whitespace().next().unwrap_or("").trim().to_string();
                        assets.push(RuleGroupAsset {
                            name: group_name,
                            inherit: String::new(),
                            source: source.path.clone(),
                            include_file,
                            matchers: Vec::new(),
                            rule_count: 0,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    let mut realized = Vec::new();
    let mut stack: Vec<PendingGroupAsset> = Vec::new();

    for source in sources {
        if let Some(group_name) = &source.implicit_group {
            stack.push(PendingGroupAsset {
                name: group_name.clone(),
                inherit: String::new(),
                source: source.path.clone(),
                include_file: source.path.clone(),
                matchers: Vec::new(),
                rule_count: 0,
            });
        }

        for line in logical_lines(&source.text) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut parts = trimmed.splitn(2, char::is_whitespace);
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();

            match key {
                "group-begin" => {
                    let name = value.split_whitespace().next().unwrap_or("").trim().to_string();
                    let inherit = parse_named_option(value, "-inherit").unwrap_or_default();
                    stack.push(PendingGroupAsset {
                        name,
                        inherit,
                        source: source.path.clone(),
                        include_file: String::new(),
                        matchers: Vec::new(),
                        rule_count: 0,
                    });
                }
                "group-match" => {
                    if let Some(group) = stack.last_mut() {
                        group.matchers.push(value.to_string());
                    }
                }
                "group-end" => {
                    if let Some(group) = stack.pop() {
                        realized.push(RuleGroupAsset {
                            name: group.name,
                            inherit: group.inherit,
                            source: group.source,
                            include_file: group.include_file,
                            matchers: group.matchers,
                            rule_count: group.rule_count,
                        });
                    }
                }
                _ => {
                    if let Some(group) = stack.last_mut() {
                        group.rule_count += 1;
                    }
                }
            }
        }

        if source.implicit_group.is_some() {
            if let Some(group) = stack.pop() {
                realized.push(RuleGroupAsset {
                    name: group.name,
                    inherit: group.inherit,
                    source: group.source,
                    include_file: group.include_file,
                    matchers: group.matchers,
                    rule_count: group.rule_count,
                });
            }
        }
    }

    for dynamic in realized {
        if let Some(asset) = assets.iter_mut().find(|candidate| {
            candidate.name == dynamic.name
                && candidate.source == dynamic.source
                && candidate.include_file == dynamic.include_file
        }) {
            asset.matchers = dynamic.matchers;
            asset.rule_count = dynamic.rule_count;
            if asset.inherit.is_empty() {
                asset.inherit = dynamic.inherit;
            }
        } else {
            assets.push(dynamic);
        }
    }

    assets
}

fn build_client_rule_assets(sources: &[ConfigSource]) -> Vec<ClientRuleAsset> {
    let mut assets = Vec::new();

    for source in sources {
        let mut group_stack: Vec<String> = Vec::new();
        if let Some(group_name) = &source.implicit_group {
            group_stack.push(group_name.clone());
        }

        for line in logical_lines(&source.text) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut parts = trimmed.splitn(2, char::is_whitespace);
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();

            match key {
                "group-begin" => {
                    let name = value.split_whitespace().next().unwrap_or("").trim().to_string();
                    group_stack.push(name);
                }
                "group-end" => {
                    group_stack.pop();
                }
                "client-rules" => {
                    assets.push(ClientRuleAsset {
                        matcher: value.to_string(),
                        group_name: group_stack.last().cloned().unwrap_or_default(),
                        source: source.path.clone(),
                        note: describe_client_matcher(value),
                    });
                }
                _ => {}
            }
        }
    }

    assets
}

fn describe_client_matcher(value: &str) -> String {
    if value.contains(':') && value.matches(':').count() >= 5 {
        "按客户端 MAC 地址匹配".to_string()
    } else {
        "按客户端 IP 或网段匹配".to_string()
    }
}

fn build_ip_rule_assets(sources: &[ConfigSource]) -> Vec<IpRuleAsset> {
    let mut assets = Vec::new();

    for source in sources {
        let mut group_stack: Vec<String> = Vec::new();
        if let Some(group_name) = &source.implicit_group {
            group_stack.push(group_name.clone());
        }

        for line in logical_lines(&source.text) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut parts = trimmed.splitn(2, char::is_whitespace);
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();

            match key {
                "group-begin" => {
                    let name = value.split_whitespace().next().unwrap_or("").trim().to_string();
                    group_stack.push(name);
                }
                "group-end" => {
                    group_stack.pop();
                }
                "ip-rules" => {
                    let (target, options) = split_target_options(value);
                    assets.push(IpRuleAsset {
                        rule_type: "ip-rules".to_string(),
                        target,
                        options,
                        group_name: group_stack.last().cloned().unwrap_or_default(),
                        source: source.path.clone(),
                        note: "对指定 IP、网段或 ip-set 应用 IP 规则选项".to_string(),
                    });
                }
                "whitelist-ip" | "blacklist-ip" | "ignore-ip" | "bogus-nxdomain" | "ip-alias" => {
                    let (target, options) = split_target_options(value);
                    assets.push(IpRuleAsset {
                        rule_type: key.to_string(),
                        target,
                        options,
                        group_name: group_stack.last().cloned().unwrap_or_default(),
                        source: source.path.clone(),
                        note: describe_ip_rule_type(key),
                    });
                }
                _ => {}
            }
        }
    }

    assets
}

fn describe_ip_rule_type(rule_type: &str) -> String {
    match rule_type {
        "whitelist-ip" => "只接受白名单范围内的返回 IP".to_string(),
        "blacklist-ip" => "丢弃命中黑名单范围的返回 IP".to_string(),
        "ignore-ip" => "忽略指定 IP 或 IP 集合".to_string(),
        "bogus-nxdomain" => "命中假冒地址时返回 SOA".to_string(),
        "ip-alias" => "把命中的 IP 映射到别名地址".to_string(),
        _ => "IP 规则".to_string(),
    }
}

fn build_ip_set_assets(
    sources: &[ConfigSource],
    rule_groups: &[RuleGroupAsset],
    ip_rules: &[IpRuleAsset],
) -> Vec<IpSetAsset> {
    let mut assets = Vec::new();

    for source in sources {
        for line in logical_lines(&source.text) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut parts = trimmed.splitn(2, char::is_whitespace);
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();

            if key != "ip-set" {
                continue;
            }

            if let Some(name) = parse_named_option(value, "-name") {
                let file = parse_named_option(value, "-file").unwrap_or_default();
                let (rule_count, sample_items) = inspect_rule_set_file(&file);
                let mut referenced_by = Vec::new();
                let marker = format!("ip-set:{}", name);

                for group in rule_groups {
                    for matcher in &group.matchers {
                        if matcher.contains(&marker) {
                            referenced_by.push(format!("group-match -> {} ({})", group.name, matcher));
                        }
                    }
                }

                for item in ip_rules {
                    if item.target.contains(&marker) || item.options.contains(&marker) {
                        referenced_by.push(format!(
                            "{} -> {}{}",
                            item.rule_type,
                            item.target,
                            if item.options.is_empty() {
                                String::new()
                            } else {
                                format!(" ({})", item.options)
                            }
                        ));
                    }
                }

                assets.push(IpSetAsset {
                    name,
                    file,
                    source: source.path.clone(),
                    rule_count,
                    sample_items,
                    referenced_by,
                });
            }
        }
    }

    assets
}

fn describe_target_hint(value: &str, fallback: &str) -> String {
    if value.contains("domain-set:") {
        "把一个域名集合整体导向指定上游组".to_string()
    } else if value.contains("/.cn/") || value.contains("/.cn/") {
        "把命中的顶级域名或模式导向指定上游组".to_string()
    } else {
        fallback.to_string()
    }
}

fn render_detected_rules_text(items: &[RoutingRuleItem]) -> String {
    if items.is_empty() {
        return "当前没有从主配置或 conf-file 中检测到分流规则。".to_string();
    }

    let mut lines = vec![
        "# Detected by smartdns-plus-ui".to_string(),
        "# 下面是从当前主配置和 conf-file 中识别出来的现有规则，只读展示，不会直接覆盖原文件。".to_string(),
        String::new(),
    ];

    for item in items {
        let value = if item.value.trim().is_empty() {
            String::new()
        } else {
            format!(" {}", item.value.trim())
        };
        lines.push(format!("{} {}{}", item.rule_type, item.target, value));
        lines.push(format!("# source: {}", item.source));
        if !item.note.trim().is_empty() {
            lines.push(format!("# note: {}", item.note));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

fn parse_upstream_config(kind: &str, value: &str, options: &str) -> UpstreamConfig {
    let mut cfg = UpstreamConfig {
        kind: kind.to_string(),
        value: value.to_string(),
        ..UpstreamConfig::default()
    };

    let mut passthrough = Vec::new();
    let tokens: Vec<&str> = options.split_whitespace().collect();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = tokens[index];
        match token {
            "-g" | "-group" => {
                if let Some(next) = tokens.get(index + 1) {
                    cfg.groups.push((*next).to_string());
                    index += 2;
                    continue;
                }
            }
            "-e" | "-exclude-default-group" => {
                cfg.exclude_default_group = true;
                index += 1;
                continue;
            }
            "-proxy" => {
                if let Some(next) = tokens.get(index + 1) {
                    cfg.proxy_name = (*next).to_string();
                    index += 2;
                    continue;
                }
            }
            "-b" | "-bootstrap-dns" => {
                if let Some(next) = tokens.get(index + 1) {
                    cfg.bootstrap_dns = (*next).to_string();
                    index += 2;
                    continue;
                }
            }
            "-host-ip" => {
                if let Some(next) = tokens.get(index + 1) {
                    cfg.host_ip = (*next).to_string();
                    index += 2;
                    continue;
                }
            }
            "-subnet" => {
                if let Some(next) = tokens.get(index + 1) {
                    cfg.subnet = (*next).to_string();
                    index += 2;
                    continue;
                }
            }
            "-fallback" => {
                cfg.fallback = true;
                index += 1;
                continue;
            }
            _ => {}
        }

        passthrough.push(token.to_string());
        if let Some(next) = tokens.get(index + 1) {
            if !next.starts_with('-') {
                passthrough.push((*next).to_string());
                index += 2;
            } else {
                index += 1;
            }
        } else {
            index += 1;
        }
    }

    cfg.options = passthrough.join(" ");
    cfg
}

fn parse_yesno(value: &str, default_value: bool) -> bool {
    if value.eq_ignore_ascii_case("yes") || value.eq_ignore_ascii_case("true") || value == "1" {
        return true;
    }
    if value.eq_ignore_ascii_case("no") || value.eq_ignore_ascii_case("false") || value == "0" {
        return false;
    }
    default_value
}

fn push_kv(lines: &mut Vec<String>, key: &str, value: &str, default_value: &str) {
    let actual = if value.trim().is_empty() {
        default_value
    } else {
        value.trim()
    };
    lines.push(format!("{} {}", key, actual));
}

fn push_bool(lines: &mut Vec<String>, key: &str, value: bool) {
    lines.push(format!("{} {}", key, if value { "yes" } else { "no" }));
}

fn validate_beginner_config(config: &BeginnerConfig, rules_text: &str) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if config.binds.iter().all(|x| x.trim().is_empty()) {
        warnings.push("未设置 bind，将自动使用 [::]:53。".to_string());
    }

    if config.upstreams.iter().all(|x| x.value.trim().is_empty()) {
        errors.push("至少需要配置一个上游 DNS 服务器。".to_string());
    }

    if config.response_mode.trim().is_empty() {
        errors.push("response-mode 不能为空。".to_string());
    }

    if config.speed_check_mode.trim().is_empty() {
        warnings.push("speed-check-mode 为空时将退回默认测速策略。".to_string());
    }

    if config.cache_size.trim().is_empty() {
        warnings.push("cache-size 为空时将使用默认值 32768。".to_string());
    }

    if config.serve_expired && config.serve_expired_ttl.trim().is_empty() {
        warnings.push("已启用 serve-expired，但未指定 TTL，将使用默认值 86400。".to_string());
    }

    let mut group_depth = 0i32;
    for (index, line) in rules_text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("group-begin ") {
            group_depth += 1;
        } else if trimmed == "group-end" {
            group_depth -= 1;
        }
        if group_depth < 0 {
            errors.push(format!("规则文件第 {} 行出现多余的 group-end。", index + 1));
            group_depth = 0;
        }
    }
    if group_depth > 0 {
        errors.push("规则文件中的 group-begin / group-end 未闭合。".to_string());
    }

    ValidationResult {
        ok: errors.is_empty(),
        errors,
        warnings,
    }
}

fn explanation_items() -> Vec<ExplanationItem> {
    vec![
        ExplanationItem {
            key: "bind".to_string(),
            title: "监听地址".to_string(),
            description: "决定 SmartDNS 在哪些 IP 和端口上接受 DNS 请求。".to_string(),
            recommendation: "家庭环境通常保持 53 端口即可。".to_string(),
            caution: "如果端口被别的 DNS 服务占用，SmartDNS 将无法启动。".to_string(),
        },
        ExplanationItem {
            key: "prefetch-domain".to_string(),
            title: "预取缓存".to_string(),
            description: "在缓存快过期时提前刷新热门域名，减少下一次访问等待。".to_string(),
            recommendation: "新手建议开启，提高连续访问体验。".to_string(),
            caution: "会略微增加上游查询次数。".to_string(),
        },
        ExplanationItem {
            key: "serve-expired".to_string(),
            title: "过期缓存兜底".to_string(),
            description: "上游 DNS 抖动时，先返回旧缓存结果，减少网页打不开。".to_string(),
            recommendation: "家庭网络建议开启，稳定性更好。".to_string(),
            caution: "极少数场景下结果可能不是最新。".to_string(),
        },
        ExplanationItem {
            key: "response-mode".to_string(),
            title: "返回策略".to_string(),
            description: "控制 SmartDNS 按测速结果还是按响应速度返回 IP。".to_string(),
            recommendation: "默认推荐 first-ping，兼顾体验与稳定性。".to_string(),
            caution: "乱改可能导致解析结果不符合预期。".to_string(),
        },
        ExplanationItem {
            key: "speed-check-mode".to_string(),
            title: "测速方式".to_string(),
            description: "决定 SmartDNS 用 ping、TCP 端口或组合方式判断哪个 IP 更快。".to_string(),
            recommendation: "默认使用 ping,tcp:80,tcp:443。".to_string(),
            caution: "纯 ping 有时不能真实反映网页访问速度。".to_string(),
        },
        ExplanationItem {
            key: "dualstack-ip-selection".to_string(),
            title: "双栈优化".to_string(),
            description: "当目标同时有 IPv4 和 IPv6 时，帮助挑选更合适的返回结果。".to_string(),
            recommendation: "有 IPv6 网络时建议开启。".to_string(),
            caution: "如果你的 IPv6 网络不稳定，可能需要关闭。".to_string(),
        },
        ExplanationItem {
            key: "upstreams".to_string(),
            title: "上游 DNS".to_string(),
            description: "SmartDNS 会向这些 DNS 服务器发起查询，再把更快的结果返回给客户端。".to_string(),
            recommendation: "至少配置两个不同来源的上游，提高可用性。".to_string(),
            caution: "只配一个上游时，一旦它异常就容易影响解析。".to_string(),
        },
    ]
}

fn recommendations_for(config: &BeginnerConfig) -> Vec<String> {
    let mut items = Vec::new();
    if config.upstreams.len() < 2 {
        items.push("建议至少配置两个上游 DNS，提高容错能力。".to_string());
    }
    if !config.prefetch_domain {
        items.push("你关闭了预取缓存；如果更看重体感速度，建议开启 prefetch-domain。".to_string());
    }
    if !config.serve_expired {
        items.push("你关闭了过期缓存兜底；遇到上游 DNS 抖动时，网页可能更容易短暂打不开。".to_string());
    }
    if config.log_level.eq_ignore_ascii_case("debug") {
        items.push("当前日志级别为 debug，适合排错，但长期运行可能产生更多日志。".to_string());
    }
    if items.is_empty() {
        items.push("当前配置看起来比较稳妥，适合家庭和新手环境。".to_string());
    }
    items
}

fn default_upstreams() -> Vec<UpstreamConfig> {
    vec![
        UpstreamConfig {
            kind: "server-https".to_string(),
            value: "https://dns.alidns.com/dns-query".to_string(),
            groups: vec!["cn".to_string()],
            options: String::new(),
            note: "国内 HTTPS DNS".to_string(),
            ..UpstreamConfig::default()
        },
        UpstreamConfig {
            kind: "server-https".to_string(),
            value: "https://cloudflare-dns.com/dns-query".to_string(),
            groups: vec!["global".to_string()],
            fallback: true,
            options: String::new(),
            note: "海外备用 HTTPS DNS".to_string(),
            ..UpstreamConfig::default()
        },
    ]
}

pub fn default_rules_text() -> String {
    let mut sections = BTreeMap::new();
    sections.insert(
        "block",
        vec![
            "# 这里可以添加你想屏蔽的域名".to_string(),
            "# address /ads.example.com/#".to_string(),
        ],
    );
    sections.insert(
        "split",
        vec![
            "# 这里可以添加你想分流的域名".to_string(),
            "# nameserver /example.com/office".to_string(),
            "# domain-rules /example.com/ -speed-check-mode ping".to_string(),
        ],
    );

    let mut lines = vec![
        "# Generated by smartdns-plus-ui".to_string(),
        "# 这份文件专门放图形界面管理的规则。".to_string(),
        "# 如果你还不熟悉 SmartDNS，建议先从少量规则开始。".to_string(),
        String::new(),
    ];

    for (name, section_lines) in sections {
        lines.push(format!("# [{}]", name));
        lines.extend(section_lines);
        lines.push(String::new());
    }

    lines.join("\n")
}

pub fn default_beginner_config() -> BeginnerConfig {
    BeginnerConfig {
        server_name: "smartdns".to_string(),
        binds: vec!["[::]:53".to_string()],
        cache_size: "32768".to_string(),
        prefetch_domain: true,
        serve_expired: true,
        serve_expired_ttl: "86400".to_string(),
        response_mode: "first-ping".to_string(),
        speed_check_mode: "ping,tcp:80,tcp:443".to_string(),
        dualstack_ip_selection: true,
        log_level: "info".to_string(),
        mdns_lookup: false,
        audit_enable: false,
        upstreams: default_upstreams(),
        extras: HashMap::new(),
    }
}

// ── Managed Assets ──────────────────────────────────────────────────────────

pub fn parse_managed_assets_from_text(text: &str) -> ManagedAssets {
    let mut rule_groups: Vec<ManagedRuleGroup> = Vec::new();
    let mut client_rules: Vec<ManagedClientRule> = Vec::new();
    let mut ip_rules: Vec<ManagedIpRule> = Vec::new();
    let mut routing_items: Vec<ManagedRoutingItem> = Vec::new();
    let mut domain_sets: Vec<ManagedDomainSet> = Vec::new();
    let mut ip_sets: Vec<ManagedIpSet> = Vec::new();

    let mut in_group: Option<ManagedRuleGroup> = None;

    for line in logical_lines(text) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let key = parts.next().unwrap_or("").trim();
        let value = parts.next().unwrap_or("").trim();

        match key {
            "group-begin" => {
                let name = value.split_whitespace().next().unwrap_or("").trim().to_string();
                let inherit = parse_named_option(value, "-inherit").unwrap_or_default();
                in_group = Some(ManagedRuleGroup {
                    name,
                    inherit,
                    matchers: Vec::new(),
                });
            }
            "group-match" => {
                if let Some(ref mut group) = in_group {
                    group.matchers.push(value.to_string());
                }
            }
            "group-end" => {
                if let Some(group) = in_group.take() {
                    rule_groups.push(group);
                }
            }
            "client-rules" => {
                let group_name = in_group
                    .as_ref()
                    .map(|group| group.name.clone())
                    .unwrap_or_default();
                client_rules.push(ManagedClientRule {
                    matcher: value.to_string(),
                    group_name,
                });
            }
            "ip-rules" | "whitelist-ip" | "blacklist-ip" | "ignore-ip"
            | "bogus-nxdomain" | "ip-alias" => {
                let (target, options) = split_target_options(value);
                let group_name = in_group
                    .as_ref()
                    .map(|group| group.name.clone())
                    .unwrap_or_default();
                ip_rules.push(ManagedIpRule {
                    rule_type: key.to_string(),
                    target,
                    options,
                    group_name,
                });
            }
            "nameserver" | "domain-rules" | "address" | "cname" => {
                let (target, value) = split_target_value(value);
                routing_items.push(ManagedRoutingItem {
                    rule_type: key.to_string(),
                    target,
                    value,
                });
            }
            "domain-set" => {
                let name = parse_named_option(value, "-name").unwrap_or_default();
                let file = parse_named_option(value, "-file").unwrap_or_default();
                let set_type = parse_named_option(value, "-type").unwrap_or_else(|| "list".to_string());
                if !name.is_empty() {
                    domain_sets.push(ManagedDomainSet { name, file, set_type });
                }
            }
            "ip-set" => {
                let name = parse_named_option(value, "-name").unwrap_or_default();
                let file = parse_named_option(value, "-file").unwrap_or_default();
                if !name.is_empty() {
                    ip_sets.push(ManagedIpSet { name, file });
                }
            }
            _ => {}
        }
    }

    if let Some(group) = in_group {
        rule_groups.push(group);
    }

    ManagedAssets {
        rule_groups,
        client_rules,
        ip_rules,
        routing_items,
        domain_sets,
        ip_sets,
    }
}

pub fn render_managed_assets_text(assets: &ManagedAssets) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("# Generated by smartdns-plus-ui".to_string());
    lines.push("# 管理规则组的图形化编辑结果。".to_string());
    lines.push(String::new());

    if !assets.domain_sets.is_empty() {
        lines.push("# ── 域名集合 ──".to_string());
        for ds in &assets.domain_sets {
            lines.push(format!(
                "domain-set -name {} -type {} -file {}",
                ds.name, ds.set_type, ds.file
            ));
        }
        lines.push(String::new());
    }

    if !assets.ip_sets.is_empty() {
        lines.push("# ── IP 集合 ──".to_string());
        for ips in &assets.ip_sets {
            lines.push(format!("ip-set -name {} -file {}", ips.name, ips.file));
        }
        lines.push(String::new());
    }

    if !assets.rule_groups.is_empty() {
        lines.push("# ── 规则组 ──".to_string());
        for group in &assets.rule_groups {
            if group.inherit.is_empty() {
                lines.push(format!("group-begin {}", group.name));
            } else {
                lines.push(format!(
                    "group-begin {} -inherit {}",
                    group.name, group.inherit
                ));
            }
            for matcher in &group.matchers {
                lines.push(format!("group-match {}", matcher));
            }
            lines.push("group-end".to_string());
            lines.push(String::new());
        }
    }

    if !assets.client_rules.is_empty() {
        lines.push("# ── 客户端规则 ──".to_string());
        let mut by_group: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for rule in &assets.client_rules {
            by_group
                .entry(rule.group_name.clone())
                .or_default()
                .push(rule.matcher.clone());
        }
        for (group_name, matchers) in &by_group {
            for matcher in matchers {
                lines.push(format!("client-rules {}", matcher));
            }
            if !group_name.is_empty() {
                lines.push(format!("# (属于规则组: {})", group_name));
            }
        }
        lines.push(String::new());
    }

    if !assets.ip_rules.is_empty() {
        lines.push("# ── IP 规则 ──".to_string());
        for rule in &assets.ip_rules {
            if rule.options.is_empty() {
                lines.push(format!("{} {}", rule.rule_type, rule.target));
            } else {
                lines.push(format!("{} {} {}", rule.rule_type, rule.target, rule.options));
            }
            if !rule.group_name.is_empty() {
                lines.push(format!("# (规则组: {})", rule.group_name));
            }
        }
        lines.push(String::new());
    }

    if !assets.routing_items.is_empty() {
        lines.push("# ── 分流规则 ──".to_string());
        for item in &assets.routing_items {
            if item.value.is_empty() {
                lines.push(format!("{} {}", item.rule_type, item.target));
            } else {
                lines.push(format!("{} {} {}", item.rule_type, item.target, item.value));
            }
        }
        lines.push(String::new());
    }

    if lines.len() <= 3 {
        lines.push("# 还没有任何托管规则。".to_string());
        lines.push("# 你可以通过图形界面新增规则组、客户端规则、IP 规则或分流规则。".to_string());
    }

    lines.push(String::new());
    lines.join("\n")
}

pub fn save_managed_assets(
    config_file: &str,
    rules_dir: &str,
    request: &SaveManagedAssetsRequest,
) -> io::Result<ValidationResult> {
    let managed_dir = managed_conf_dir(rules_dir);
    fs::create_dir_all(&managed_dir)?;

    let assets = ManagedAssets {
        rule_groups: request.rule_groups.clone(),
        client_rules: request.client_rules.clone(),
        ip_rules: request.ip_rules.clone(),
        routing_items: request.routing_items.clone(),
        domain_sets: request.domain_sets.clone(),
        ip_sets: request.ip_sets.clone(),
    };

    let validation = validate_managed_assets(&assets);
    if !validation.ok {
        return Ok(validation);
    }

    let rules_text = render_managed_assets_text(&assets);
    write_text_atomic(&path_join(&managed_dir, RULES_FILE_NAME), &rules_text)?;

    backup_file(config_file)?;
    let main_config_text = read_text_or_default(config_file)?;
    let new_main_config = ensure_managed_block(&main_config_text, rules_dir);
    write_text_atomic(config_file, &new_main_config)?;

    Ok(validation)
}

fn validate_managed_assets(assets: &ManagedAssets) -> ValidationResult {
    let mut errors = Vec::new();
    let warnings = Vec::new();

    // Check duplicate group names
    let mut seen = HashSet::new();
    for group in &assets.rule_groups {
        if !seen.insert(group.name.clone()) {
            errors.push(format!("规则组名称重复：{}", group.name));
        }
    }

    // Check invalid IP/CIDR patterns in client rules
    for rule in &assets.client_rules {
        let matcher = rule.matcher.trim();
        if matcher.is_empty() {
            errors.push("客户端规则不能为空。".to_string());
        }
    }

    // Check IP rule targets
    for rule in &assets.ip_rules {
        if rule.target.trim().is_empty() {
            errors.push(format!("{} 规则的目标不能为空。", rule.rule_type));
        }
    }

    // Check routing targets
    for item in &assets.routing_items {
        if item.target.trim().is_empty() {
            errors.push(format!("{} 规则的目标不能为空。", item.rule_type));
        }
    }

    ValidationResult {
        ok: errors.is_empty(),
        errors,
        warnings,
    }
}

// ── IP Set detail (mirrors domain-set) ──────────────────────────────────────

pub fn read_ip_set_detail(
    config_file: &str,
    name: &str,
    query: Option<&str>,
    offset: usize,
    limit: usize,
) -> io::Result<Option<RuleSetFileDetail>> {
    let config_sources = collect_config_sources(config_file)?;
    let _routing_items = parse_routing_items(&config_sources);
    let rule_groups = build_rule_group_assets(&config_sources);
    let ip_rules = build_ip_rule_assets(&config_sources);
    let ip_set_assets = build_ip_set_assets(&config_sources, &rule_groups, &ip_rules);

    let Some(asset) = ip_set_assets
        .into_iter()
        .find(|candidate| candidate.name == name)
    else {
        return Ok(None);
    };

    let all_items = read_rule_set_items(&asset.file)?;
    let query = query.unwrap_or("").trim().to_lowercase();
    let filtered: Vec<String> = if query.is_empty() {
        all_items.clone()
    } else {
        all_items
            .iter()
            .filter(|item| item.to_lowercase().contains(&query))
            .cloned()
            .collect()
    };
    let limit_val = limit.max(1);
    let items: Vec<String> = filtered
        .iter()
        .skip(offset)
        .take(limit_val)
        .cloned()
        .collect();

    Ok(Some(RuleSetFileDetail {
        name: asset.name,
        set_type: "ip-set".to_string(),
        file: asset.file.clone(),
        source: asset.source,
        exists: Path::new(&asset.file).exists(),
        rule_count: all_items.len(),
        filtered_count: filtered.len(),
        offset,
        limit: limit_val,
        has_more: offset.saturating_add(items.len()) < filtered.len(),
        items,
        sample_items: asset.sample_items,
        referenced_by: asset.referenced_by,
    }))
}

pub fn save_ip_set_file(
    config_file: &str,
    name: &str,
    request: &SaveRuleSetFileRequest,
) -> io::Result<RuleSetFileDetail> {
    let detail =
        read_ip_set_detail(config_file, name, None, 0, 200)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("ip set `{}` not found", name),
            )
        })?;

    if detail.file.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("ip set `{}` has no backing file", name),
        ));
    }

    let mut items = read_rule_set_items(&detail.file).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("read ip set file {} failed: {}", detail.file, err),
        )
    })?;
    let remove_items = normalize_rule_set_items(&request.remove_items);
    let add_items = normalize_rule_set_items(&request.add_items);

    if !remove_items.is_empty() {
        items.retain(|item| !remove_items.contains(item));
    }
    items.extend(add_items);
    items = dedupe_rule_set_items(items);

    backup_file(&detail.file).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("backup ip set file {} failed: {}", detail.file, err),
        )
    })?;
    write_text_atomic(&detail.file, &render_rule_set_items(&items)).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("write ip set file {} failed: {}", detail.file, err),
        )
    })?;

    read_ip_set_detail(config_file, name, None, 0, 200)?.ok_or_else(|| {
        io::Error::new(
            ErrorKind::NotFound,
            format!("ip set `{}` disappeared after save", name),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn reads_and_saves_rule_set_detail() {
        let temp = tempdir().unwrap();
        let rules_dir = temp.path().join("rules");
        fs::create_dir_all(&rules_dir).unwrap();

        let set_file = rules_dir.join("direct.txt");
        fs::write(&set_file, "# comment\nexample.com\nexample.org\n").unwrap();

        let config_file = temp.path().join("smartdns.conf");
        fs::write(
            &config_file,
            format!(
                "domain-set -name direct -file {}\n",
                set_file.to_string_lossy()
            ),
        )
        .unwrap();

        let detail = read_rule_set_detail(&config_file.to_string_lossy(), "direct", None, 0, 200)
            .unwrap()
            .unwrap();
        assert_eq!(detail.rule_count, 2);
        assert_eq!(detail.items, vec!["example.com", "example.org"]);

        let saved = save_rule_set_file(
            &config_file.to_string_lossy(),
            "direct",
            &SaveRuleSetFileRequest {
                add_items: vec![
                    "example.net".to_string(),
                    " ".to_string(),
                    "example.edu".to_string(),
                ],
                remove_items: vec!["example.com".to_string()],
            },
        )
        .unwrap();

        assert_eq!(saved.rule_count, 3);
        assert_eq!(
            fs::read_to_string(&set_file).unwrap(),
            "example.org\nexample.net\nexample.edu\n"
        );
    }
}

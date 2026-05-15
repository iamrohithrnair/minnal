//! Compatibility surface for the removed Mermaid renderer.
//!
//! Minnal no longer renders Mermaid diagrams or depends on the old renderer crate.
//! These no-op helpers keep older UI call sites compiling while returning empty
//! diagram/render state.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::GenericImageView as _;
use ratatui::prelude::{Buffer, Line, Rect, Span, Style};
use serde::Serialize;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

const IMAGE_MARKER_PREFIX: &str = "\x00MINNAL_IMAGE:";
const IMAGE_MARKER_SUFFIX: &str = "\x00";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiagramInfo {
    pub hash: u64,
    pub width: u32,
    pub height: u32,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessMemorySnapshot {
    pub rss_bytes: Option<u64>,
    pub peak_rss_bytes: Option<u64>,
    pub virtual_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum RenderResult {
    Image {
        hash: u64,
        path: PathBuf,
        width: u32,
        height: u32,
    },
    Error(String),
}

#[derive(Debug, Clone, Copy)]
pub enum ProtocolType {
    Disabled,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MermaidDebugStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub deferred_enqueued: u64,
    pub deferred_deduped: u64,
    pub deferred_superseded: u64,
    pub deferred_worker_renders: u64,
    pub deferred_worker_skips: u64,
    pub deferred_epoch_bumps: u64,
    pub render_success: u64,
    pub render_errors: u64,
    pub last_render_ms: Option<f32>,
    pub last_parse_ms: Option<f32>,
    pub last_layout_ms: Option<f32>,
    pub last_svg_ms: Option<f32>,
    pub last_png_ms: Option<f32>,
    pub last_error: Option<String>,
    pub last_hash: Option<String>,
    pub last_nodes: Option<usize>,
    pub last_edges: Option<usize>,
    pub last_content_len: Option<usize>,
    pub image_state_hits: u64,
    pub image_state_misses: u64,
    pub skipped_renders: u64,
    pub fit_state_reuse_hits: u64,
    pub fit_protocol_rebuilds: u64,
    pub viewport_state_reuse_hits: u64,
    pub viewport_protocol_rebuilds: u64,
    pub clear_operations: u64,
    pub last_image_render_ms: Option<f32>,
    pub cache_entries: usize,
    pub cache_dir: Option<String>,
    pub protocol: Option<String>,
    pub render_size_backend: &'static str,
    pub last_png_width: Option<u32>,
    pub last_png_height: Option<u32>,
    pub last_measured_width: Option<u32>,
    pub last_measured_height: Option<u32>,
    pub last_viewbox_width: Option<u32>,
    pub last_viewbox_height: Option<u32>,
    pub last_target_width: Option<u32>,
    pub last_target_height: Option<u32>,
    pub deferred_pending: usize,
    pub deferred_epoch: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MermaidCacheEntry {
    pub hash: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MermaidMemoryProfile {
    pub process_rss_bytes: Option<u64>,
    pub process_peak_rss_bytes: Option<u64>,
    pub process_virtual_bytes: Option<u64>,
    pub render_cache_entries: usize,
    pub render_cache_limit: usize,
    pub render_cache_metadata_estimate_bytes: u64,
    pub image_state_entries: usize,
    pub image_state_limit: usize,
    pub image_state_protocol_min_estimate_bytes: u64,
    pub source_cache_entries: usize,
    pub source_cache_limit: usize,
    pub source_cache_decoded_estimate_bytes: u64,
    pub active_diagrams: usize,
    pub active_diagrams_limit: usize,
    pub cache_disk_png_files: usize,
    pub cache_disk_png_bytes: u64,
    pub cache_disk_limit_bytes: u64,
    pub cache_disk_max_age_secs: u64,
    pub mermaid_working_set_estimate_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MermaidMemoryBenchmark {
    pub iterations: usize,
    pub errors: usize,
    pub before: MermaidMemoryProfile,
    pub after: MermaidMemoryProfile,
    pub rss_delta_bytes: Option<i64>,
    pub working_set_delta_bytes: i64,
    pub peak_rss_bytes: Option<u64>,
    pub peak_working_set_estimate_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MermaidTimingSummary {
    pub avg_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MermaidDebugStatsDelta {
    pub image_state_hits: u64,
    pub image_state_misses: u64,
    pub skipped_renders: u64,
    pub fit_state_reuse_hits: u64,
    pub fit_protocol_rebuilds: u64,
    pub viewport_state_reuse_hits: u64,
    pub viewport_protocol_rebuilds: u64,
    pub clear_operations: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MermaidFlickerBenchmark {
    pub protocol_supported: bool,
    pub protocol: Option<String>,
    pub steps: usize,
    pub changed_viewports: usize,
    pub fit_frames: usize,
    pub viewport_frames: usize,
    pub fit_timing: MermaidTimingSummary,
    pub viewport_timing: MermaidTimingSummary,
    pub deltas: MermaidDebugStatsDelta,
    pub viewport_protocol_rebuild_rate: f64,
    pub fit_protocol_rebuild_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageStateInfo {
    pub hash: String,
    pub resize_mode: String,
    pub last_area: Option<String>,
    pub last_viewport: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScrollFrameInfo {
    pub frame: usize,
    pub y_offset: i32,
    pub visible_rows: u16,
    pub rendered: bool,
    pub resize_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScrollTestResult {
    pub hash: String,
    pub frames_rendered: usize,
    pub resize_mode_changes: usize,
    pub skipped_renders: u64,
    pub render_calls: Vec<ScrollFrameInfo>,
    pub stable: bool,
    pub border_rendered: bool,
}

#[derive(Debug, Clone)]
struct RegisteredImage {
    path: Option<PathBuf>,
    width: u32,
    height: u32,
}

static ACTIVE_DIAGRAMS: LazyLock<Mutex<Vec<DiagramInfo>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
static REGISTERED_IMAGES: LazyLock<Mutex<HashMap<u64, RegisteredImage>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static VIDEO_EXPORT_MODE: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

pub fn install_minnal_mermaid_hooks() {}

pub fn set_log_hooks(_info: fn(&str), _warn: fn(&str)) {}

pub fn set_render_completed_hook(_hook: fn()) {}

pub fn set_memory_snapshot_hook(_hook: fn() -> ProcessMemorySnapshot) {}

pub fn init_picker() {}

pub fn protocol_type() -> Option<ProtocolType> {
    None
}

pub fn image_protocol_available() -> bool {
    false
}

pub fn get_font_size() -> Option<(u16, u16)> {
    Some((8, 16))
}

pub fn is_video_export_mode() -> bool {
    VIDEO_EXPORT_MODE.lock().map(|mode| *mode).unwrap_or(false)
}

pub fn set_video_export_mode(enabled: bool) {
    if let Ok(mut mode) = VIDEO_EXPORT_MODE.lock() {
        *mode = enabled;
    }
}

pub fn is_mermaid_lang(_lang: &str) -> bool {
    false
}

pub fn current_preferred_aspect_ratio_bucket() -> Option<u16> {
    None
}

pub fn preferred_aspect_ratio_bucket(_ratio: Option<f32>) -> Option<u16> {
    None
}

pub fn with_preferred_aspect_ratio<R>(_ratio: Option<f32>, f: impl FnOnce() -> R) -> R {
    f()
}

pub fn deferred_render_epoch() -> u64 {
    0
}

pub fn active_diagram_count() -> usize {
    get_active_diagrams().len()
}

pub fn get_active_diagrams() -> Vec<DiagramInfo> {
    ACTIVE_DIAGRAMS
        .lock()
        .map(|items| items.clone())
        .unwrap_or_default()
}

pub fn register_active_diagram(hash: u64, width: u32, height: u32, label: Option<String>) {
    if let Ok(mut diagrams) = ACTIVE_DIAGRAMS.lock() {
        if !diagrams.iter().any(|diagram| diagram.hash == hash) {
            diagrams.push(DiagramInfo {
                hash,
                width,
                height,
                label,
            });
        }
    }
}

pub fn set_streaming_preview_diagram(hash: u64, width: u32, height: u32, label: Option<String>) {
    register_active_diagram(hash, width, height, label);
}

pub fn clear_active_diagrams() {
    if let Ok(mut diagrams) = ACTIVE_DIAGRAMS.lock() {
        diagrams.clear();
    }
}

pub fn clear_streaming_preview_diagram() {}

pub fn snapshot_active_diagrams() -> Vec<DiagramInfo> {
    get_active_diagrams()
}

pub fn restore_active_diagrams(snapshot: Vec<DiagramInfo>) {
    if let Ok(mut diagrams) = ACTIVE_DIAGRAMS.lock() {
        *diagrams = snapshot;
    }
}

pub fn clear_image_state() {}

pub fn clear_cache() -> Result<(), String> {
    clear_active_diagrams();
    if let Ok(mut images) = REGISTERED_IMAGES.lock() {
        images.clear();
    }
    Ok(())
}

pub fn get_cached_path(hash: u64) -> Option<PathBuf> {
    REGISTERED_IMAGES
        .lock()
        .ok()?
        .get(&hash)
        .and_then(|image| image.path.clone())
}

pub fn get_cached_png(hash: u64) -> Option<(PathBuf, u32, u32)> {
    let image = REGISTERED_IMAGES.lock().ok()?.get(&hash)?.clone();
    image.path.map(|path| (path, image.width, image.height))
}

pub fn register_external_image(path: &Path, width: u32, height: u32) -> u64 {
    let hash = stable_hash(&(path.to_string_lossy(), width, height));
    if let Ok(mut images) = REGISTERED_IMAGES.lock() {
        images.insert(
            hash,
            RegisteredImage {
                path: Some(path.to_path_buf()),
                width,
                height,
            },
        );
    }
    hash
}

pub fn register_inline_image(media_type: &str, data: &str) -> Option<(u64, u32, u32)> {
    if !media_type.starts_with("image/") {
        return None;
    }
    let bytes = BASE64.decode(data).ok()?;
    let decoded = image::load_from_memory(&bytes).ok()?;
    let (width, height) = decoded.dimensions();
    let hash = stable_hash(&(media_type, data.len(), &bytes[..bytes.len().min(256)]));
    if let Ok(mut images) = REGISTERED_IMAGES.lock() {
        images.insert(
            hash,
            RegisteredImage {
                path: None,
                width,
                height,
            },
        );
    }
    Some((hash, width, height))
}

pub fn render_image_widget(
    _hash: u64,
    _area: Rect,
    _buf: &mut Buffer,
    _centered: bool,
    _crop_top: bool,
) -> u16 {
    0
}

pub fn render_image_widget_scale(
    _hash: u64,
    _area: Rect,
    _buf: &mut Buffer,
    _crop_top: bool,
) -> u16 {
    0
}

pub fn render_image_widget_viewport(
    _hash: u64,
    _area: Rect,
    _buf: &mut Buffer,
    _scroll_x: i32,
    _scroll_y: i32,
    _zoom_percent: u8,
    _crop_top: bool,
) -> u16 {
    0
}

pub fn render_image_widget_viewport_precise(
    _hash: u64,
    _area: Rect,
    _buf: &mut Buffer,
    _scroll_x: i32,
    _scroll_y: i32,
    _zoom_percent: u16,
    _crop_top: bool,
) -> u16 {
    0
}

pub fn diagram_placeholder_lines(width: u32, height: u32) -> Vec<Line<'static>> {
    vec![Line::from(Span::styled(
        format!("diagram rendering disabled ({width}x{height})"),
        Style::default(),
    ))]
}

pub fn write_video_export_marker(_hash: u64, _area: Rect, _buf: &mut Buffer) {}

pub fn image_widget_placeholder_markdown(hash: u64) -> String {
    format!("{IMAGE_MARKER_PREFIX}{hash:016x}{IMAGE_MARKER_SUFFIX}\n")
}

pub fn parse_image_placeholder(line: &Line<'_>) -> Option<u64> {
    let content = line.spans.first()?.content.as_ref();
    if !content.starts_with(IMAGE_MARKER_PREFIX) || !content.ends_with(IMAGE_MARKER_SUFFIX) {
        return None;
    }
    let start = IMAGE_MARKER_PREFIX.len();
    let end = content.len().saturating_sub(IMAGE_MARKER_SUFFIX.len());
    u64::from_str_radix(&content[start..end], 16).ok()
}

pub fn render_mermaid_untracked(_content: &str, _terminal_width: Option<u16>) -> RenderResult {
    RenderResult::Error("Mermaid rendering is disabled".to_string())
}

pub fn debug_stats() -> MermaidDebugStats {
    MermaidDebugStats {
        render_size_backend: "disabled",
        protocol: None,
        ..MermaidDebugStats::default()
    }
}

pub fn reset_debug_stats() {}

pub fn debug_stats_json() -> Option<serde_json::Value> {
    serde_json::to_value(debug_stats()).ok()
}

pub fn debug_cache() -> Vec<MermaidCacheEntry> {
    Vec::new()
}

pub fn debug_image_state() -> Vec<ImageStateInfo> {
    Vec::new()
}

pub fn debug_memory_profile() -> MermaidMemoryProfile {
    MermaidMemoryProfile {
        active_diagrams: active_diagram_count(),
        ..MermaidMemoryProfile::default()
    }
}

pub fn debug_memory_benchmark(iterations: usize) -> MermaidMemoryBenchmark {
    let profile = debug_memory_profile();
    MermaidMemoryBenchmark {
        iterations,
        errors: 0,
        before: profile.clone(),
        after: profile,
        rss_delta_bytes: Some(0),
        working_set_delta_bytes: 0,
        peak_rss_bytes: None,
        peak_working_set_estimate_bytes: 0,
    }
}

pub fn debug_flicker_benchmark(steps: usize) -> MermaidFlickerBenchmark {
    let timing = MermaidTimingSummary {
        avg_ms: 0.0,
        p50_ms: 0.0,
        p95_ms: 0.0,
        p99_ms: 0.0,
        max_ms: 0.0,
    };
    MermaidFlickerBenchmark {
        protocol_supported: false,
        protocol: None,
        steps,
        changed_viewports: 0,
        fit_frames: 0,
        viewport_frames: 0,
        fit_timing: timing.clone(),
        viewport_timing: timing,
        deltas: MermaidDebugStatsDelta::default(),
        viewport_protocol_rebuild_rate: 0.0,
        fit_protocol_rebuild_rate: 0.0,
    }
}

pub fn debug_test_scroll(_content: Option<&str>) -> ScrollTestResult {
    ScrollTestResult {
        hash: "disabled".to_string(),
        frames_rendered: 0,
        resize_mode_changes: 0,
        skipped_renders: 0,
        render_calls: Vec::new(),
        stable: true,
        border_rendered: false,
    }
}

fn stable_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

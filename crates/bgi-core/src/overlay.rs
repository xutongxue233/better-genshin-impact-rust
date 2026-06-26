use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MaskWindowConfig {
    pub directions_enabled: bool,
    pub display_recognition_results_on_mask: bool,
    pub mask_enabled: bool,
    pub show_log_box: bool,
    pub show_status: bool,
    pub uid_cover_enabled: bool,
    pub show_fps: bool,
    pub show_overlay_metrics: bool,
    pub overlay_metric_items: BTreeMap<String, bool>,
    pub text_opacity: f64,
    pub overlay_layout_edit_enabled: bool,
    pub log_text_box_left_ratio: f64,
    pub log_text_box_top_ratio: f64,
    pub log_text_box_width_ratio: f64,
    pub log_text_box_height_ratio: f64,
    pub status_list_left_ratio: f64,
    pub status_list_top_ratio: f64,
    pub status_list_width_ratio: f64,
    pub status_list_height_ratio: f64,
    pub metrics_left_ratio: f64,
    pub metrics_top_ratio: f64,
    pub metrics_width_ratio: f64,
    pub metrics_height_ratio: f64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OverlayMetricItem {
    GameFps,
    ProcessingCost,
    PeakProcessingCost,
    CaptureCost,
    TriggerCost,
    SkippedTicks,
    GpuUsage,
    CpuUsage,
    MemoryUsage,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OverlayMetricDescriptor {
    pub key: &'static str,
    pub display_name: &'static str,
    pub tooltip: &'static str,
    pub enabled_by_default: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OverlayLayoutRect {
    pub left_ratio: f64,
    pub top_ratio: f64,
    pub width_ratio: f64,
    pub height_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MaskWindowState {
    pub mask_enabled: bool,
    pub show_log_box: bool,
    pub show_status: bool,
    pub display_recognition_results_on_mask: bool,
    pub directions_enabled: bool,
    pub uid_cover_enabled: bool,
    pub show_fps: bool,
    pub show_overlay_metrics: bool,
    pub text_opacity: f64,
    pub overlay_layout_edit_enabled: bool,
    pub log_box_layout: OverlayLayoutRect,
    pub status_layout: OverlayLayoutRect,
    pub metrics_layout: OverlayLayoutRect,
    pub metrics: Vec<OverlayMetricDescriptor>,
    pub enabled_metric_keys: Vec<String>,
    pub migrated_legacy_metrics_layout: bool,
}

impl Default for MaskWindowConfig {
    fn default() -> Self {
        Self {
            directions_enabled: false,
            display_recognition_results_on_mask: true,
            mask_enabled: true,
            show_log_box: true,
            show_status: true,
            uid_cover_enabled: false,
            show_fps: false,
            show_overlay_metrics: false,
            overlay_metric_items: overlay_metric_defaults(),
            text_opacity: 1.0,
            overlay_layout_edit_enabled: false,
            log_text_box_left_ratio: 20.0 / 1920.0,
            log_text_box_top_ratio: 822.0 / 1080.0,
            log_text_box_width_ratio: 480.0 / 1920.0,
            log_text_box_height_ratio: 188.0 / 1080.0,
            status_list_left_ratio: 20.0 / 1920.0,
            status_list_top_ratio: 790.0 / 1080.0,
            status_list_width_ratio: 480.0 / 1920.0,
            status_list_height_ratio: 24.0 / 1080.0,
            metrics_left_ratio: 20.0 / 1920.0,
            metrics_top_ratio: 744.0 / 1080.0,
            metrics_width_ratio: 477.0 / 1920.0,
            metrics_height_ratio: 58.0 / 1080.0,
            extra: Map::new(),
        }
    }
}

impl OverlayMetricItem {
    pub fn key(self) -> &'static str {
        match self {
            Self::GameFps => "GameFps",
            Self::ProcessingCost => "ProcessingCost",
            Self::PeakProcessingCost => "PeakProcessingCost",
            Self::CaptureCost => "CaptureCost",
            Self::TriggerCost => "TriggerCost",
            Self::SkippedTicks => "SkippedTicks",
            Self::GpuUsage => "GpuUsage",
            Self::CpuUsage => "CpuUsage",
            Self::MemoryUsage => "MemoryUsage",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::GameFps => "游戏帧率",
            Self::ProcessingCost => "处理耗时",
            Self::PeakProcessingCost => "峰值耗时",
            Self::CaptureCost => "截图耗时",
            Self::TriggerCost => "触发器耗时",
            Self::SkippedTicks => "跳过次数",
            Self::GpuUsage => "显卡占用",
            Self::CpuUsage => "CPU占用",
            Self::MemoryUsage => "内存占用",
        }
    }

    pub fn tooltip(self) -> &'static str {
        match self {
            Self::GameFps => "游戏当前渲染帧率。",
            Self::ProcessingCost => "BetterGI 每轮截图、识别和触发器处理的总耗时。",
            Self::PeakProcessingCost => "最近 5 秒内 BetterGI 单轮处理耗时的峰值。",
            Self::CaptureCost => "单次获取游戏画面的耗时。",
            Self::TriggerCost => "本轮实际执行触发器的总耗时。",
            Self::SkippedTicks => "上一轮未结束导致本秒跳过的调度次数。",
            Self::GpuUsage => "显卡核心占用率，读取不到时自动隐藏。",
            Self::CpuUsage => "CPU 总占用率，读取不到时自动隐藏。",
            Self::MemoryUsage => "系统内存占用率，读取不到时自动隐藏。",
        }
    }

    pub fn enabled_by_default(self) -> bool {
        matches!(
            self,
            Self::GameFps
                | Self::ProcessingCost
                | Self::PeakProcessingCost
                | Self::CaptureCost
                | Self::SkippedTicks
                | Self::MemoryUsage
        )
    }
}

pub const OVERLAY_METRIC_ITEMS: &[OverlayMetricItem] = &[
    OverlayMetricItem::GameFps,
    OverlayMetricItem::ProcessingCost,
    OverlayMetricItem::PeakProcessingCost,
    OverlayMetricItem::CaptureCost,
    OverlayMetricItem::TriggerCost,
    OverlayMetricItem::SkippedTicks,
    OverlayMetricItem::GpuUsage,
    OverlayMetricItem::CpuUsage,
    OverlayMetricItem::MemoryUsage,
];

pub(super) const DEFAULT_METRICS_LAYOUT: OverlayLayoutRect = OverlayLayoutRect {
    left_ratio: 20.0 / 1920.0,
    top_ratio: 744.0 / 1080.0,
    width_ratio: 477.0 / 1920.0,
    height_ratio: 58.0 / 1080.0,
};

const LEGACY_METRICS_LAYOUTS: &[OverlayLayoutRect] = &[
    OverlayLayoutRect {
        left_ratio: 4.0 / 1920.0,
        top_ratio: 4.0 / 1080.0,
        width_ratio: 720.0 / 1920.0,
        height_ratio: 42.0 / 1080.0,
    },
    OverlayLayoutRect {
        left_ratio: 600.0 / 1920.0,
        top_ratio: 16.0 / 1080.0,
        width_ratio: 720.0 / 1920.0,
        height_ratio: 42.0 / 1080.0,
    },
    OverlayLayoutRect {
        left_ratio: 20.0 / 1920.0,
        top_ratio: 724.0 / 1080.0,
        width_ratio: 760.0 / 1920.0,
        height_ratio: 58.0 / 1080.0,
    },
    OverlayLayoutRect {
        left_ratio: 20.0 / 1920.0,
        top_ratio: 724.0 / 1080.0,
        width_ratio: 760.0 / 1920.0,
        height_ratio: 42.0 / 1080.0,
    },
    OverlayLayoutRect {
        left_ratio: 20.0 / 1920.0,
        top_ratio: 760.0 / 1080.0,
        width_ratio: 477.0 / 1920.0,
        height_ratio: 42.0 / 1080.0,
    },
    OverlayLayoutRect {
        left_ratio: 20.0 / 1920.0,
        top_ratio: 760.0 / 1080.0,
        width_ratio: 477.0 / 1920.0,
        height_ratio: 58.0 / 1080.0,
    },
];

impl MaskWindowConfig {
    pub fn ensure_overlay_metric_items(&mut self) {
        let legacy_trigger_interval = "TriggerInterval";
        if let Some(enabled) = self
            .overlay_metric_items
            .get(legacy_trigger_interval)
            .copied()
        {
            self.overlay_metric_items
                .entry(OverlayMetricItem::PeakProcessingCost.key().to_string())
                .or_insert(enabled);
        }

        for item in OVERLAY_METRIC_ITEMS {
            self.overlay_metric_items
                .entry(item.key().to_string())
                .or_insert_with(|| item.enabled_by_default());
        }

        self.overlay_metric_items
            .retain(|key, _| overlay_metric_item_from_key(key).is_some());
    }

    pub fn migrate_legacy_overlay_metrics_layout(&mut self) -> bool {
        let current = self.metrics_layout();
        if !LEGACY_METRICS_LAYOUTS
            .iter()
            .any(|layout| same_overlay_layout(&current, layout))
        {
            return false;
        }

        self.metrics_left_ratio = DEFAULT_METRICS_LAYOUT.left_ratio;
        self.metrics_top_ratio = DEFAULT_METRICS_LAYOUT.top_ratio;
        self.metrics_width_ratio = DEFAULT_METRICS_LAYOUT.width_ratio;
        self.metrics_height_ratio = DEFAULT_METRICS_LAYOUT.height_ratio;
        true
    }

    pub fn overlay_state(&self) -> MaskWindowState {
        let mut config = self.clone();
        config.ensure_overlay_metric_items();
        let migrated_legacy_metrics_layout = config.migrate_legacy_overlay_metrics_layout();
        let enabled_metric_keys = OVERLAY_METRIC_ITEMS
            .iter()
            .filter(|item| {
                config
                    .overlay_metric_items
                    .get(item.key())
                    .copied()
                    .unwrap_or_else(|| item.enabled_by_default())
            })
            .map(|item| item.key().to_string())
            .collect::<Vec<_>>();

        MaskWindowState {
            mask_enabled: config.mask_enabled,
            show_log_box: config.show_log_box,
            show_status: config.show_status,
            display_recognition_results_on_mask: config.display_recognition_results_on_mask,
            directions_enabled: config.directions_enabled,
            uid_cover_enabled: config.uid_cover_enabled,
            show_fps: config.show_fps,
            show_overlay_metrics: config.show_overlay_metrics,
            text_opacity: config.text_opacity,
            overlay_layout_edit_enabled: config.overlay_layout_edit_enabled,
            log_box_layout: OverlayLayoutRect {
                left_ratio: config.log_text_box_left_ratio,
                top_ratio: config.log_text_box_top_ratio,
                width_ratio: config.log_text_box_width_ratio,
                height_ratio: config.log_text_box_height_ratio,
            },
            status_layout: OverlayLayoutRect {
                left_ratio: config.status_list_left_ratio,
                top_ratio: config.status_list_top_ratio,
                width_ratio: config.status_list_width_ratio,
                height_ratio: config.status_list_height_ratio,
            },
            metrics_layout: config.metrics_layout(),
            metrics: overlay_metric_descriptors(),
            enabled_metric_keys,
            migrated_legacy_metrics_layout,
        }
    }

    fn metrics_layout(&self) -> OverlayLayoutRect {
        OverlayLayoutRect {
            left_ratio: self.metrics_left_ratio,
            top_ratio: self.metrics_top_ratio,
            width_ratio: self.metrics_width_ratio,
            height_ratio: self.metrics_height_ratio,
        }
    }
}

pub fn overlay_metric_descriptors() -> Vec<OverlayMetricDescriptor> {
    OVERLAY_METRIC_ITEMS
        .iter()
        .map(|item| OverlayMetricDescriptor {
            key: item.key(),
            display_name: item.display_name(),
            tooltip: item.tooltip(),
            enabled_by_default: item.enabled_by_default(),
        })
        .collect()
}

pub fn overlay_metric_item_from_key(key: &str) -> Option<OverlayMetricItem> {
    OVERLAY_METRIC_ITEMS
        .iter()
        .copied()
        .find(|item| item.key() == key)
}

pub(super) fn same_overlay_layout(left: &OverlayLayoutRect, right: &OverlayLayoutRect) -> bool {
    same_ratio(left.left_ratio, right.left_ratio)
        && same_ratio(left.top_ratio, right.top_ratio)
        && same_ratio(left.width_ratio, right.width_ratio)
        && same_ratio(left.height_ratio, right.height_ratio)
}

pub(super) fn same_ratio(left: f64, right: f64) -> bool {
    (left - right).abs() < 0.0000001
}

fn overlay_metric_defaults() -> BTreeMap<String, bool> {
    [
        ("GameFps", true),
        ("ProcessingCost", true),
        ("PeakProcessingCost", true),
        ("CaptureCost", true),
        ("TriggerCost", false),
        ("SkippedTicks", true),
        ("GpuUsage", false),
        ("CpuUsage", false),
        ("MemoryUsage", true),
    ]
    .into_iter()
    .map(|(key, enabled)| (key.to_string(), enabled))
    .collect()
}

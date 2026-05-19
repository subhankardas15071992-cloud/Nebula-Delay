//! Preset management for **Nebula Delay**.
//!
//! Presets stay JSON based so users can move and inspect them easily. The
//! value struct keeps the legacy stereo fields for migration compatibility,
//! but the mono plugin only exposes and processes the left-side values.

use std::fs;
use std::path::{Path, PathBuf};

use nih_plug::params::enums::Enum;
use serde::{Deserialize, Serialize};

use crate::parameters::{
    InputModeParam, NebulaStereoDelayParams, NoteValueParam, OversamplingParam, RoutingModeParam,
};

const PRESET_VERSION: &str = "1.0.0";
const FACTORY_AUTHOR: &str = "Nebula Audio";
const FACTORY_CREATED: &str = "2026-05-19T00:00:00Z";

#[derive(Serialize, Deserialize, Clone)]
pub struct PresetData {
    pub name: String,
    pub author: String,
    pub created: String,
    pub version: String,
    pub values: PresetValues,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PresetValues {
    #[serde(default)]
    pub input_level_db: f32,
    #[serde(default)]
    pub output_level_db: f32,
    #[serde(default = "default_input_l")]
    pub input_mode_l: u8,
    #[serde(default)]
    pub input_mode_r: u8,
    pub delay_time_l: f32,
    #[serde(default = "default_delay_time")]
    pub delay_time_r: f32,
    pub note_l: u8,
    #[serde(default = "default_note")]
    pub note_r: u8,
    #[serde(default)]
    pub deviation_l: f32,
    #[serde(default)]
    pub deviation_r: f32,
    #[serde(default)]
    pub halve_l: bool,
    #[serde(default)]
    pub halve_r: bool,
    #[serde(default)]
    pub double_l: bool,
    #[serde(default)]
    pub double_r: bool,
    pub low_cut_l: f32,
    #[serde(default = "default_low_cut")]
    pub low_cut_r: f32,
    #[serde(default = "default_filter_slope")]
    pub low_cut_slope_l: f32,
    #[serde(default = "default_filter_slope")]
    pub low_cut_slope_r: f32,
    pub high_cut_l: f32,
    #[serde(default = "default_high_cut")]
    pub high_cut_r: f32,
    #[serde(default = "default_filter_slope")]
    pub high_cut_slope_l: f32,
    #[serde(default = "default_filter_slope")]
    pub high_cut_slope_r: f32,
    pub feedback_l: f32,
    #[serde(default = "default_feedback")]
    pub feedback_r: f32,
    #[serde(default)]
    pub feedback_phase_l: bool,
    #[serde(default)]
    pub feedback_phase_r: bool,
    #[serde(default)]
    pub crossfeed_lr: f32,
    #[serde(default)]
    pub crossfeed_rl: f32,
    #[serde(default, alias = "crossfeed_phase")]
    pub crossfeed_phase_lr: bool,
    #[serde(default)]
    pub crossfeed_phase_rl: bool,
    #[serde(default = "default_routing")]
    pub routing: u8,
    #[serde(default)]
    pub oversampling: u8,
    #[serde(default)]
    pub tempo_sync: bool,
    #[serde(default)]
    pub stereo_link: bool,
    pub output_mix_l: f32,
    #[serde(default = "default_mix")]
    pub output_mix_r: f32,
}

impl Default for PresetValues {
    fn default() -> Self {
        Self {
            input_level_db: 0.0,
            output_level_db: 0.0,
            input_mode_l: default_input_l(),
            input_mode_r: 0,
            delay_time_l: default_delay_time(),
            delay_time_r: default_delay_time(),
            note_l: default_note(),
            note_r: default_note(),
            deviation_l: 0.0,
            deviation_r: 0.0,
            halve_l: false,
            halve_r: false,
            double_l: false,
            double_r: false,
            low_cut_l: default_low_cut(),
            low_cut_r: default_low_cut(),
            low_cut_slope_l: default_filter_slope(),
            low_cut_slope_r: default_filter_slope(),
            high_cut_l: default_high_cut(),
            high_cut_r: default_high_cut(),
            high_cut_slope_l: default_filter_slope(),
            high_cut_slope_r: default_filter_slope(),
            feedback_l: default_feedback(),
            feedback_r: 0.0,
            feedback_phase_l: false,
            feedback_phase_r: false,
            crossfeed_lr: 0.0,
            crossfeed_rl: 0.0,
            crossfeed_phase_lr: false,
            crossfeed_phase_rl: false,
            routing: default_routing(),
            oversampling: 0,
            tempo_sync: false,
            stereo_link: false,
            output_mix_l: default_mix(),
            output_mix_r: default_mix(),
        }
    }
}

fn default_input_l() -> u8 {
    1
}
fn default_delay_time() -> f32 {
    0.5
}
fn default_note() -> u8 {
    3
}
fn default_low_cut() -> f32 {
    20.0
}
fn default_high_cut() -> f32 {
    20_000.0
}
fn default_filter_slope() -> f32 {
    12.0
}
fn default_feedback() -> f32 {
    0.4
}
fn default_mix() -> f32 {
    1.0
}
fn default_routing() -> u8 {
    1
}

pub struct PresetManager {
    factory_presets: Vec<PresetData>,
    user_preset_dir: PathBuf,
}

impl PresetManager {
    pub fn new() -> Self {
        Self {
            factory_presets: build_factory_presets(),
            user_preset_dir: resolve_user_preset_dir(),
        }
    }

    pub fn factory_presets(&self) -> &[PresetData] {
        &self.factory_presets
    }

    pub fn user_presets(&self) -> Result<Vec<PresetData>, String> {
        let mut presets = Vec::new();
        if !self.user_preset_dir.exists() {
            return Ok(presets);
        }

        let entries = fs::read_dir(&self.user_preset_dir)
            .map_err(|e| format!("Failed to read user preset directory: {e}"))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            if let Ok(preset) = Self::load_preset_file(&path) {
                presets.push(preset);
            }
        }
        presets.sort_by_key(|preset| preset.name.to_lowercase());
        Ok(presets)
    }

    pub fn save_user_preset(
        &self,
        name: &str,
        author: &str,
        values: &PresetValues,
    ) -> Result<(), String> {
        fs::create_dir_all(&self.user_preset_dir)
            .map_err(|e| format!("Failed to create user preset directory: {e}"))?;

        let preset = PresetData {
            name: name.to_string(),
            author: author.to_string(),
            created: now_iso8601(),
            version: PRESET_VERSION.to_string(),
            values: values.clone(),
        };
        let path = self.user_preset_dir.join(preset_filename(name));
        let json = serde_json::to_string_pretty(&preset)
            .map_err(|e| format!("Failed to serialise preset: {e}"))?;
        fs::write(path, json).map_err(|e| format!("Failed to write preset file: {e}"))
    }

    pub fn load_preset(
        &self,
        preset: &PresetData,
        params: &NebulaStereoDelayParams,
        setter: &nih_plug::prelude::ParamSetter,
    ) {
        let v = &preset.values;

        setter.set_parameter(&params.input_level, v.input_level_db);
        setter.set_parameter(&params.output_level, v.output_level_db);
        setter.set_parameter(&params.input_mode_l, InputModeParam::Left);
        setter.set_parameter(&params.input_mode_r, InputModeParam::Off);
        setter.set_parameter(&params.delay_time_l, v.delay_time_l);
        setter.set_parameter(&params.delay_time_r, v.delay_time_l);
        setter.set_parameter(
            &params.note_l,
            NoteValueParam::from_index(v.note_l as usize),
        );
        setter.set_parameter(
            &params.note_r,
            NoteValueParam::from_index(v.note_l as usize),
        );
        setter.set_parameter(&params.deviation_l, 0.0);
        setter.set_parameter(&params.deviation_r, 0.0);
        setter.set_parameter(&params.halve_l, v.halve_l);
        setter.set_parameter(&params.halve_r, false);
        setter.set_parameter(&params.double_l, v.double_l);
        setter.set_parameter(&params.double_r, false);
        setter.set_parameter(&params.low_cut_l, v.low_cut_l);
        setter.set_parameter(&params.low_cut_r, v.low_cut_l);
        setter.set_parameter(&params.low_cut_slope_l, v.low_cut_slope_l);
        setter.set_parameter(&params.low_cut_slope_r, v.low_cut_slope_l);
        setter.set_parameter(&params.high_cut_l, v.high_cut_l);
        setter.set_parameter(&params.high_cut_r, v.high_cut_l);
        setter.set_parameter(&params.high_cut_slope_l, v.high_cut_slope_l);
        setter.set_parameter(&params.high_cut_slope_r, v.high_cut_slope_l);
        setter.set_parameter(&params.feedback_l, v.feedback_l);
        setter.set_parameter(&params.feedback_r, 0.0);
        setter.set_parameter(&params.feedback_phase_l, v.feedback_phase_l);
        setter.set_parameter(&params.feedback_phase_r, false);
        setter.set_parameter(&params.crossfeed_lr, 0.0);
        setter.set_parameter(&params.crossfeed_rl, 0.0);
        setter.set_parameter(&params.crossfeed_phase_lr, false);
        setter.set_parameter(&params.crossfeed_phase_rl, false);
        setter.set_parameter(&params.routing, RoutingModeParam::Straight);
        setter.set_parameter(
            &params.oversampling,
            OversamplingParam::from_index(v.oversampling as usize),
        );
        setter.set_parameter(&params.tempo_sync, v.tempo_sync);
        setter.set_parameter(&params.stereo_link, false);
        setter.set_parameter(&params.output_mix_l, v.output_mix_l);
        setter.set_parameter(&params.output_mix_r, v.output_mix_r);
    }

    pub fn delete_user_preset(&self, name: &str) -> Result<(), String> {
        let path = self.user_preset_dir.join(preset_filename(name));
        if !path.exists() {
            return Err(format!("Preset '{name}' does not exist"));
        }
        fs::remove_file(path).map_err(|e| format!("Failed to delete preset: {e}"))
    }

    pub fn export_preset(&self, preset: &PresetData, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(preset)
            .map_err(|e| format!("Failed to serialise preset: {e}"))?;
        fs::write(path, json).map_err(|e| format!("Failed to export preset: {e}"))
    }

    pub fn import_preset(&self, path: &Path) -> Result<PresetData, String> {
        Self::load_preset_file(path)
    }

    fn load_preset_file(path: &Path) -> Result<PresetData, String> {
        let json = fs::read_to_string(path).map_err(|e| format!("Failed to read preset: {e}"))?;
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse preset: {e}"))
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new()
    }
}

fn preset(name: &str, values: PresetValues) -> PresetData {
    PresetData {
        name: name.to_string(),
        author: FACTORY_AUTHOR.to_string(),
        created: FACTORY_CREATED.to_string(),
        version: PRESET_VERSION.to_string(),
        values,
    }
}

struct MonoPresetSpec {
    delay_time: f32,
    note: u8,
    feedback: f32,
    low_cut: f32,
    high_cut: f32,
    mix: f32,
    tempo_sync: bool,
    phase_invert: bool,
}

fn values(spec: MonoPresetSpec) -> PresetValues {
    PresetValues {
        delay_time_l: spec.delay_time,
        delay_time_r: spec.delay_time,
        note_l: spec.note,
        note_r: spec.note,
        feedback_l: spec.feedback,
        low_cut_l: spec.low_cut,
        low_cut_r: spec.low_cut,
        high_cut_l: spec.high_cut,
        high_cut_r: spec.high_cut,
        output_mix_l: 1.0,
        output_mix_r: spec.mix,
        tempo_sync: spec.tempo_sync,
        feedback_phase_l: spec.phase_invert,
        ..PresetValues::default()
    }
}

fn build_factory_presets() -> Vec<PresetData> {
    vec![
        preset("Init", PresetValues::default()),
        preset(
            "Simple Slap",
            values(MonoPresetSpec {
                delay_time: 0.075,
                note: 5,
                feedback: 0.05,
                low_cut: 80.0,
                high_cut: 12_000.0,
                mix: 0.45,
                tempo_sync: false,
                phase_invert: false,
            }),
        ),
        preset(
            "Ambient Wash",
            values(MonoPresetSpec {
                delay_time: 1.2,
                note: 1,
                feedback: 0.72,
                low_cut: 220.0,
                high_cut: 6_000.0,
                mix: 0.78,
                tempo_sync: true,
                phase_invert: false,
            }),
        ),
        preset(
            "Dub Echo",
            values(MonoPresetSpec {
                delay_time: 0.38,
                note: 5,
                feedback: 0.68,
                low_cut: 120.0,
                high_cut: 4_800.0,
                mix: 0.72,
                tempo_sync: true,
                phase_invert: false,
            }),
        ),
        preset(
            "Tape Throw",
            values(MonoPresetSpec {
                delay_time: 0.32,
                note: 5,
                feedback: 0.48,
                low_cut: 160.0,
                high_cut: 4_200.0,
                mix: 0.64,
                tempo_sync: false,
                phase_invert: false,
            }),
        ),
        preset(
            "Tight Doubler",
            values(MonoPresetSpec {
                delay_time: 0.018,
                note: 11,
                feedback: 0.0,
                low_cut: 90.0,
                high_cut: 12_000.0,
                mix: 0.42,
                tempo_sync: false,
                phase_invert: false,
            }),
        ),
        preset(
            "Space Echo",
            values(MonoPresetSpec {
                delay_time: 0.42,
                note: 5,
                feedback: 0.56,
                low_cut: 120.0,
                high_cut: 4_500.0,
                mix: 0.74,
                tempo_sync: true,
                phase_invert: false,
            }),
        ),
        preset(
            "Phase Drift",
            values(MonoPresetSpec {
                delay_time: 0.024,
                note: 11,
                feedback: 0.18,
                low_cut: 140.0,
                high_cut: 9_000.0,
                mix: 0.55,
                tempo_sync: false,
                phase_invert: true,
            }),
        ),
        preset(
            "Rhythmic Delay",
            values(MonoPresetSpec {
                delay_time: 0.5,
                note: 6,
                feedback: 0.52,
                low_cut: 150.0,
                high_cut: 7_000.0,
                mix: 0.68,
                tempo_sync: true,
                phase_invert: false,
            }),
        ),
        preset(
            "Vintage Tape",
            values(MonoPresetSpec {
                delay_time: 0.28,
                note: 5,
                feedback: 0.6,
                low_cut: 200.0,
                high_cut: 3_000.0,
                mix: 0.72,
                tempo_sync: true,
                phase_invert: false,
            }),
        ),
    ]
}

fn resolve_user_preset_dir() -> PathBuf {
    let base = if cfg!(target_os = "macos") {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library").join("Application Support"))
            .unwrap_or_else(|| PathBuf::from("."))
    } else if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("LOCALAPPDATA"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .map(|home| home.join(".local").join("share"))
            })
            .unwrap_or_else(|| PathBuf::from("."))
    };

    base.join("NebulaAudio").join("NebulaDelay").join("Presets")
}

fn preset_filename(name: &str) -> String {
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{}.json", safe.trim().replace(' ', "_"))
}

fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("{seconds}")
}

//! Main plugin entry point for **Nebula Delay** by Nebula Audio.
//!
//! This is the monophonic sibling of Nebula Stereo Delay. It keeps the same
//! 64-bit delay core, MIDI learn, A/B, undo/redo, presets, hard bypass,
//! oversampling, and native editors, but exposes a true 1-in/1-out mono
//! delay to the host.

pub mod dsp;

#[cfg(all(feature = "plugin", feature = "gui", not(target_os = "windows")))]
pub mod gui;
#[cfg(feature = "plugin")]
pub mod midi;
#[cfg(feature = "plugin")]
pub mod parameters;
#[cfg(feature = "plugin")]
pub mod preset;
#[cfg(feature = "plugin")]
pub mod state;
#[cfg(all(feature = "plugin", feature = "gui", target_os = "windows"))]
mod windows_editor;

#[cfg(feature = "plugin")]
use std::sync::atomic::Ordering;
#[cfg(feature = "plugin")]
use std::sync::Arc;

#[cfg(feature = "plugin")]
use nih_plug::prelude::*;

#[cfg(feature = "plugin")]
use crate::dsp::{DelayEngine, InputMode, RoutingMode};
#[cfg(feature = "plugin")]
use crate::midi::{sync_runtime_from_learn_state, MidiRuntime, MidiTarget};
#[cfg(feature = "plugin")]
use crate::parameters::NebulaDelayParams;
#[cfg(feature = "plugin")]
use crate::preset::PresetManager;
#[cfg(feature = "plugin")]
use crate::state::{MeterValues, StateManager};

#[cfg(feature = "plugin")]
const DENORMAL_THRESHOLD_F64: f64 = 1e-30;
#[cfg(feature = "plugin")]
const DENORMAL_THRESHOLD_F32: f32 = 1e-30;
#[cfg(feature = "plugin")]
const DEFAULT_TEMPO_BPM: f64 = 120.0;
#[cfg(feature = "plugin")]
const MAX_OVERSAMPLING_FACTOR: usize = 8;

#[cfg(feature = "plugin")]
#[inline(always)]
fn flush_denormal_f64(x: f64) -> f64 {
    if x.abs() < DENORMAL_THRESHOLD_F64 {
        0.0
    } else {
        x
    }
}

#[cfg(feature = "plugin")]
#[inline(always)]
fn flush_denormal_f32(x: f32) -> f32 {
    if x.abs() < DENORMAL_THRESHOLD_F32 {
        0.0
    } else {
        x
    }
}

#[cfg(feature = "plugin")]
pub struct NebulaDelay {
    params: Arc<NebulaDelayParams>,
    engine: DelayEngine,
    state_manager: StateManager,
    meters: Arc<MeterValues>,
    midi_runtime: Arc<MidiRuntime>,
    _preset_manager: PresetManager,
    sample_rate: f64,
    oversampling_factor: usize,
    prev_input: f64,
}

#[cfg(feature = "plugin")]
impl Default for NebulaDelay {
    fn default() -> Self {
        let params = Arc::new(NebulaDelayParams::default());
        let meters = Arc::new(MeterValues::new());
        let sample_rate = 44_100.0;
        let mut engine = DelayEngine::new(sample_rate * MAX_OVERSAMPLING_FACTOR as f64);
        engine.set_sample_rate(sample_rate);

        Self {
            state_manager: StateManager::with_meters(params.clone(), meters.clone()),
            engine,
            params,
            meters,
            midi_runtime: Arc::new(MidiRuntime::new()),
            _preset_manager: PresetManager::new(),
            sample_rate,
            oversampling_factor: 1,
            prev_input: 0.0,
        }
    }
}

#[cfg(feature = "plugin")]
impl Plugin for NebulaDelay {
    const NAME: &'static str = "Nebula Delay";
    const VENDOR: &'static str = "Nebula Audio";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = "1.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: Some(new_nonzero_u32(1)),
        main_output_channels: Some(new_nonzero_u32(1)),
        ..AudioIOLayout::const_default()
    }];
    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        #[cfg(all(feature = "gui", not(target_os = "windows")))]
        {
            gui::create_egui_editor(
                self.params.clone(),
                self.midi_runtime.clone(),
                self.meters.clone(),
            )
        }

        #[cfg(all(feature = "gui", target_os = "windows"))]
        {
            windows_editor::create_editor(
                self.params.clone(),
                self.midi_runtime.clone(),
                self.meters.clone(),
            )
        }

        #[cfg(not(feature = "gui"))]
        {
            None
        }
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate as f64;
        self.oversampling_factor = self.params.oversampling.value().factor();
        self.engine = DelayEngine::new(self.sample_rate * MAX_OVERSAMPLING_FACTOR as f64);
        self.engine
            .set_sample_rate(self.sample_rate * self.oversampling_factor as f64);
        self.engine.reset();
        self.prev_input = 0.0;
        self.state_manager = StateManager::with_meters(self.params.clone(), self.meters.clone());
        if let Ok(learn) = self.params.midi_learn.read() {
            sync_runtime_from_learn_state(&self.midi_runtime, &learn);
        }
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();
        let tempo_bpm = transport.tempo.unwrap_or(DEFAULT_TEMPO_BPM);

        while let Some(event) = context.next_event() {
            if let NoteEvent::MidiCC {
                channel, cc, value, ..
            } = event
            {
                self.midi_runtime.process_cc(channel, cc, value);
            }
        }

        let midi_bypass = self
            .midi_runtime
            .target_value(MidiTarget::Bypass)
            .map(|v| v >= 0.5);
        let hard_bypass = midi_bypass.unwrap_or_else(|| self.params.bypass.load(Ordering::Relaxed));
        self.engine.set_bypass(hard_bypass);

        let phase_invert = self
            .midi_runtime
            .target_value(MidiTarget::FeedbackPhaseL)
            .map(|v| self.params.feedback_phase_l.preview_plain(v))
            .unwrap_or_else(|| self.params.feedback_phase_l.value());
        let tempo_sync = self
            .midi_runtime
            .target_value(MidiTarget::TempoSync)
            .map(|v| self.params.tempo_sync.preview_plain(v))
            .unwrap_or_else(|| self.params.tempo_sync.value());
        let note = self
            .midi_runtime
            .target_value(MidiTarget::NoteL)
            .map(|v| self.params.note_l.preview_plain(v).into())
            .unwrap_or_else(|| self.params.note_l.value().into());
        let halve = self
            .midi_runtime
            .target_value(MidiTarget::HalveL)
            .map(|v| self.params.halve_l.preview_plain(v))
            .unwrap_or_else(|| self.params.halve_l.value());
        let double = self
            .midi_runtime
            .target_value(MidiTarget::DoubleL)
            .map(|v| self.params.double_l.preview_plain(v))
            .unwrap_or_else(|| self.params.double_l.value());
        let oversampling_factor = self
            .midi_runtime
            .target_value(MidiTarget::Oversampling)
            .map(|v| self.params.oversampling.preview_plain(v).factor())
            .unwrap_or_else(|| self.params.oversampling.value().factor());
        if oversampling_factor != self.oversampling_factor {
            self.oversampling_factor = oversampling_factor;
            self.engine
                .set_sample_rate(self.sample_rate * self.oversampling_factor as f64);
            self.engine.reset();
            self.prev_input = 0.0;
        }

        for mut sample in buffer.iter_samples() {
            let Some(mono) = sample.iter_mut().next() else {
                continue;
            };

            macro_rules! midi_float {
                ($target:expr, $param:ident) => {
                    self.midi_runtime
                        .target_value($target)
                        .map(|v| self.params.$param.preview_plain(v) as f64)
                        .unwrap_or_else(|| self.params.$param.smoothed.next() as f64)
                };
            }

            let delay_params = dsp::DelayParams {
                input_level_db: midi_float!(MidiTarget::InputLevel, input_level),
                output_level_db: midi_float!(MidiTarget::OutputLevel, output_level),
                input_mode_l: InputMode::Left,
                input_mode_r: InputMode::Off,
                delay_time_l: midi_float!(MidiTarget::DelayTimeL, delay_time_l),
                delay_time_r: 0.5,
                low_cut_l: midi_float!(MidiTarget::LowCutL, low_cut_l),
                low_cut_r: 20.0,
                low_cut_slope_l: midi_float!(MidiTarget::LowCutSlopeL, low_cut_slope_l),
                low_cut_slope_r: 12.0,
                high_cut_l: midi_float!(MidiTarget::HighCutL, high_cut_l),
                high_cut_r: 20_000.0,
                high_cut_slope_l: midi_float!(MidiTarget::HighCutSlopeL, high_cut_slope_l),
                high_cut_slope_r: 12.0,
                feedback_l: midi_float!(MidiTarget::FeedbackL, feedback_l),
                feedback_r: 0.0,
                feedback_phase_l: false,
                feedback_phase_r: false,
                crossfeed_lr: 0.0,
                crossfeed_rl: 0.0,
                crossfeed_phase_lr: false,
                crossfeed_phase_rl: false,
                routing: RoutingMode::Straight,
                tempo_sync,
                tempo_bpm,
                note_l: note,
                note_r: note,
                deviation_l: 0.0,
                deviation_r: 0.0,
                halve_l: halve,
                halve_r: false,
                double_l: double,
                double_r: false,
                output_mix_l: midi_float!(MidiTarget::OutputMixL, output_mix_l),
                output_mix_r: midi_float!(MidiTarget::OutputMixR, output_mix_r),
                wet_phase_l: phase_invert,
                wet_phase_r: false,
                bypass: hard_bypass,
                stereo_link: false,
            };

            let input = *mono as f64;
            let mut input_meter = 0.0f32;
            let mut output_meter = 0.0f32;

            let out = if self.oversampling_factor == 1 {
                let frame = self.engine.process_frame(input, 0.0, &delay_params);
                input_meter = input_meter.max(frame.input_meter_l.abs() as f32);
                output_meter = output_meter.max(frame.output_meter_l.abs() as f32);
                frame.output_l
            } else {
                let factor = self.oversampling_factor;
                let factor_f = factor as f64;
                let mut acc = 0.0;
                for sub in 0..factor {
                    let t = (sub as f64 + 1.0) / factor_f;
                    let os_input = self.prev_input + (input - self.prev_input) * t;
                    let frame = self.engine.process_frame(os_input, 0.0, &delay_params);
                    input_meter = input_meter.max(frame.input_meter_l.abs() as f32);
                    output_meter = output_meter.max(frame.output_meter_l.abs() as f32);
                    acc += frame.output_l;
                }
                acc / factor_f
            };
            self.prev_input = input;

            let out = flush_denormal_f64(out);
            let out_f32 = flush_denormal_f32(out as f32);

            *mono = out_f32;

            self.state_manager.update_meters(
                input_meter,
                0.0,
                output_meter.max(out_f32.abs()),
                0.0,
                0.0,
                0.0,
            );
        }

        ProcessStatus::Normal
    }
}

#[cfg(all(feature = "plugin", not(target_os = "windows")))]
impl ClapPlugin for NebulaDelay {
    const CLAP_ID: &'static str = "audio.nebula.delay";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Professional mono delay audio effect");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Delay,
        ClapFeature::Mono,
    ];
}

#[cfg(feature = "plugin")]
impl Vst3Plugin for NebulaDelay {
    const VST3_CLASS_ID: [u8; 16] = *b"NebulaDelayVST3!";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Delay];
}

#[cfg(all(feature = "plugin", not(target_os = "windows")))]
nih_plug::nih_export_clap!(NebulaDelay);

#[cfg(feature = "plugin")]
nih_plug::nih_export_vst3!(NebulaDelay);

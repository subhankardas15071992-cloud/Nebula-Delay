use nebula_delay::dsp::{DelayEngine, DelayParams, InputMode, RoutingMode};

#[test]
fn mono_defaults_pass_audio_without_non_finite_output() {
    let mut engine = DelayEngine::new(44_100.0);
    let params = DelayParams {
        input_mode_l: InputMode::Left,
        input_mode_r: InputMode::Off,
        routing: RoutingMode::Straight,
        crossfeed_lr: 0.0,
        crossfeed_rl: 0.0,
        output_mix_l: 1.0,
        output_mix_r: 1.0,
        ..DelayParams::default()
    };

    for _ in 0..512 {
        let frame = engine.process_frame(0.25, 0.0, &params);
        assert!(frame.output_l.is_finite());
        assert_eq!(frame.output_r, 0.0);
    }
}

#[test]
fn hard_bypass_is_transparent_for_mono() {
    let mut engine = DelayEngine::new(44_100.0);
    let params = DelayParams {
        bypass: true,
        input_mode_l: InputMode::Left,
        input_mode_r: InputMode::Off,
        ..DelayParams::default()
    };

    let frame = engine.process_frame(-0.375, 0.0, &params);
    assert_eq!(frame.output_l, -0.375);
    assert_eq!(frame.output_r, 0.0);
}

#[test]
fn dry_level_controls_only_the_direct_signal() {
    let mut engine = DelayEngine::new(44_100.0);
    let dry_only = DelayParams {
        input_mode_l: InputMode::Left,
        input_mode_r: InputMode::Off,
        routing: RoutingMode::Straight,
        delay_time_l: 0.005,
        feedback_l: 0.0,
        output_mix_l: 1.0,
        output_mix_r: 0.0,
        ..DelayParams::default()
    };
    for _ in 0..128 {
        engine.process_frame(0.0, 0.0, &dry_only);
    }

    let frame = engine.process_frame(0.75, 0.0, &dry_only);
    assert!(
        frame.output_l > 0.70,
        "dry-only path should pass the direct signal, got {}",
        frame.output_l
    );

    let muted_dry = DelayParams {
        output_mix_l: 0.0,
        ..dry_only
    };
    for _ in 0..128 {
        engine.process_frame(0.0, 0.0, &muted_dry);
    }
    let frame = engine.process_frame(0.75, 0.0, &muted_dry);
    assert!(
        frame.output_l.abs() < 1.0e-6,
        "dry level at 0% should remove the direct signal, got {}",
        frame.output_l
    );
}

#[test]
fn wet_level_controls_only_the_delayed_signal() {
    let mut engine = DelayEngine::new(44_100.0);
    let wet_only = DelayParams {
        input_mode_l: InputMode::Left,
        input_mode_r: InputMode::Off,
        routing: RoutingMode::Straight,
        delay_time_l: 0.005,
        feedback_l: 0.0,
        output_mix_l: 0.0,
        output_mix_r: 1.0,
        ..DelayParams::default()
    };
    for _ in 0..160 {
        engine.process_frame(0.0, 0.0, &wet_only);
    }

    let immediate = engine.process_frame(1.0, 0.0, &wet_only);
    assert!(
        immediate.output_l.abs() < 1.0e-6,
        "wet-only path should not leak the direct signal, got {}",
        immediate.output_l
    );

    let mut delayed_peak = 0.0_f64;
    for _ in 0..260 {
        let frame = engine.process_frame(0.0, 0.0, &wet_only);
        delayed_peak = delayed_peak.max(frame.output_l.abs());
    }
    assert!(
        delayed_peak > 0.45,
        "wet level at 100% should return the delayed signal, peak was {delayed_peak}"
    );

    let muted_wet = DelayParams {
        output_mix_r: 0.0,
        ..wet_only
    };
    let mut engine = DelayEngine::new(44_100.0);
    for _ in 0..160 {
        engine.process_frame(0.0, 0.0, &muted_wet);
    }
    engine.process_frame(1.0, 0.0, &muted_wet);
    let mut muted_peak = 0.0_f64;
    for _ in 0..260 {
        let frame = engine.process_frame(0.0, 0.0, &muted_wet);
        muted_peak = muted_peak.max(frame.output_l.abs());
    }
    assert!(
        muted_peak < 1.0e-6,
        "wet level at 0% should mute delayed repeats, peak was {muted_peak}"
    );
}

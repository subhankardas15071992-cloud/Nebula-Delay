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

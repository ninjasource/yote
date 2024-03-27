use cs47l63::registers::{
    audio_serial_port::asp_ctrl,
    clocking_sample_rates::{clock, fll, sample_rate},
    digital_core::{compression, equalizer, mixers},
    gpio::gpio_ctrl,
    input_signal_path::{
        input_signal_path_config, input_signal_path_control, input_signal_path_enable,
    },
    output_signal_path::volume_ctrl,
    reset,
    voltage_regulators::voltage,
};

// This module serves as a set of known working configuration for the CS47L63 DSP
// Each const array can be sent directly to the DSP to control it in some way.

pub const SOFT_RESET: [[u32; 2]; 1] = [reset::SoftReset::new().serialize()];

pub const CLOCK_CONFIGURATION: [[u32; 2]; 9] = [
    sample_rate::SampleRate {
        num: sample_rate::Num::_3,
        select: sample_rate::Select::_16kHz,
    }
    .serialize(),
    sample_rate::SampleRate {
        num: sample_rate::Num::_2,
        select: sample_rate::Select::_24kHz,
    }
    .serialize(),
    sample_rate::SampleRate {
        num: sample_rate::Num::_1,
        select: sample_rate::Select::_48kHz,
    }
    .serialize(),
    clock::SystemClock1 {
        frac: clock::Fraction::MultipleOf6M144Hz,
        freq: clock::Freqency::_49p152MHz,
        enabled: true,
        src: clock::Source::FLL1_45to50MHz,
    }
    .serialize(),
    clock::AsyncClock1 {
        freq: clock::Freqency::_49p152MHz,
        enabled: true,
        src: clock::Source::FLL1_45to50MHz,
    }
    .serialize(),
    fll::Fll1Control2 {
        lock_detect_threshold: 8,
        lock_detect: true,
        phase_detect: false,
        ref_detect: true,
        divider: fll::ReferenceClockDivider::_1,
        source: fll::ReferenceClockSource::MCLK1,
        multiplier: 8,
    }
    .serialize(),
    fll::Fll1Control3 {
        lambda: 1,
        theta: 0,
    }
    .serialize(),
    fll::FllGpioClock {
        num: fll::Num::_1,
        source: fll::GpioClockSource::Fll,
        divider: 2,
        enabled: true,
    }
    .serialize(),
    fll::Fll1Control1 {
        control_update: false,
        hold: false,
        enabled: true,
    }
    .serialize(),
];

pub const GPIO_CONFIGURATION: [[u32; 2]; 4] = [
    gpio_ctrl1(
        gpio_ctrl::Num::_6(gpio_ctrl::ExtendedPinFunction::ButtonDetectInputOrLogicLevelOutput),
        gpio_ctrl::Direction::Output,
    ),
    gpio_ctrl1(
        gpio_ctrl::Num::_7(gpio_ctrl::ExtendedPinFunction::ButtonDetectInputOrLogicLevelOutput),
        gpio_ctrl::Direction::Output,
    ),
    gpio_ctrl1(
        gpio_ctrl::Num::_8(gpio_ctrl::ExtendedPinFunction::ButtonDetectInputOrLogicLevelOutput),
        gpio_ctrl::Direction::Output,
    ),
    gpio_ctrl1(
        gpio_ctrl::Num::_10(gpio_ctrl::ExtendedPinFunction::ButtonDetectInputOrLogicLevelOutput),
        gpio_ctrl::Direction::Output,
    ),
];

// audio serial port setup
pub const ASP1_ENABLE: [[u32; 2]; 14] = [
    // enable ASP1 GPIOs
    gpio_ctrl1(
        gpio_ctrl::Num::_1(gpio_ctrl::PinFunction::AlternateFunction),
        gpio_ctrl::Direction::Output,
    ),
    gpio_ctrl1(
        gpio_ctrl::Num::_2(gpio_ctrl::PinFunction::AlternateFunction),
        gpio_ctrl::Direction::Input,
    ),
    gpio_ctrl1(
        gpio_ctrl::Num::_3(gpio_ctrl::PinFunction::AlternateFunction),
        gpio_ctrl::Direction::Input,
    ),
    gpio_ctrl1(
        gpio_ctrl::Num::_4(gpio_ctrl::PinFunction::AlternateFunction),
        gpio_ctrl::Direction::Input,
    ),
    gpio_ctrl1(
        gpio_ctrl::Num::_5(gpio_ctrl::ExtendedPinFunction::ButtonDetectInputOrLogicLevelOutput),
        gpio_ctrl::Direction::Output,
    ),
    sample_rate::SampleRate {
        num: sample_rate::Num::_1,
        select: sample_rate::Select::_48kHz,
    }
    .serialize(),
    // disable unused sample rates
    sample_rate::SampleRate {
        num: sample_rate::Num::_2,
        select: sample_rate::Select::None,
    }
    .serialize(),
    sample_rate::SampleRate {
        num: sample_rate::Num::_3,
        select: sample_rate::Select::None,
    }
    .serialize(),
    sample_rate::SampleRate {
        num: sample_rate::Num::_4,
        select: sample_rate::Select::None,
    }
    .serialize(),
    // set ASP1 in slave mode and 16 bit per channel
    asp_ctrl::AspControl2 {
        num: asp_ctrl::Num::_1,
        rx_width: 0x10,
        tx_width: 0x10,
        format: asp_ctrl::AspFormat::I2sMode,
        bclk_invert: false,
        bclk_frc: asp_ctrl::AspBclkOutputControl::Normal,
        bclk_mstr: asp_ctrl::AspBclkMasterSelect::SlaveMode,
        fsync_invert: false,
        fsync_frc: asp_ctrl::AspBclkOutputControl::Normal,
        fsync_mstr: asp_ctrl::AspBclkMasterSelect::SlaveMode,
    }
    .serialize(),
    asp_ctrl::AspControl3 {
        num: asp_ctrl::Num::_1,
        dout_hiz_ctrl: asp_ctrl::AspDoutTristateControl::Mode00,
    }
    .serialize(),
    asp_ctrl::AspDataControl1 {
        num: asp_ctrl::Num::_1,
        tx_data_width_bits: 32,
    }
    .serialize(),
    asp_ctrl::AspDataControl5 {
        num: asp_ctrl::Num::_1,
        rx_data_width_bits: 32,
    }
    .serialize(),
    asp_ctrl::Asp1Enables1 {
        rx8_enabled: false,
        rx7_enabled: false,
        rx6_enabled: false,
        rx5_enabled: false,
        rx4_enabled: false,
        rx3_enabled: false,
        rx2_enabled: true,
        rx1_enabled: true,
        tx8_enabled: false,
        tx7_enabled: false,
        tx6_enabled: false,
        tx5_enabled: false,
        tx4_enabled: false,
        tx3_enabled: false,
        tx2_enabled: true,
        tx1_enabled: true,
    }
    .serialize(),
];

const fn gpio_ctrl1(num: gpio_ctrl::Num, direction: gpio_ctrl::Direction) -> [u32; 2] {
    gpio_ctrl::GpioCtrl1 {
        num,
        direction,
        pull_up_en: true,
        pull_down_en: true,
        drive_strength: gpio_ctrl::DriveStrength::_8mA,
        debounce_time: gpio_ctrl::DebounceTime::_100us,
        output_level: false,
        output_config: gpio_ctrl::OutputConfig::Cmos,
        debounce_en: false,
        output_polarity: gpio_ctrl::OutputPolarity::NoninvertedActiveHigh,
    }
    .serialize()
}

// feed the mic into the equalizer
// Asp1Rx1 -> Eq1 -> Out1L
pub const OUTPUT_ENABLE_EQ: [[u32; 2]; 4] = [
    volume_ctrl::OutputEnable1 { enabled: true }.serialize(),
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Eq1Input1,
        input_volume: 64,
        status_enabled: false,
        source_select: mixers::InputSourceSelect::Asp1Rx1,
    }
    .serialize(),
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Out1LInput1,
        input_volume: 64,
        status_enabled: false,
        source_select: mixers::InputSourceSelect::Eq1,
    }
    .serialize(),
    volume_ctrl::Out1LVolume1 {
        mute: false,
        volume: 0x62,
        update: true,
    }
    .serialize(),
];

// pass from mic to out1l
pub const OUTPUT_ENABLE_PASSTHOUGH: [[u32; 2]; 2] = [
    volume_ctrl::OutputEnable1 { enabled: true }.serialize(),
    volume_ctrl::Out1LVolume1 {
        mute: false,
        volume: 0x90,
        update: true,
    }
    .serialize(),
];

// feed the mic into the equalizer
// Asp1Rx1 and Asp1Rx2 -> Out1L
pub const OUTPUT_ENABLE_BASIC: [[u32; 2]; 4] = [
    volume_ctrl::OutputEnable1 { enabled: true }.serialize(),
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Out1LInput1,
        input_volume: 64,
        status_enabled: false,
        source_select: mixers::InputSourceSelect::Asp1Rx1,
    }
    .serialize(),
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Out1LInput2,
        input_volume: 64,
        status_enabled: false,
        source_select: mixers::InputSourceSelect::Asp1Rx2,
    }
    .serialize(),
    volume_ctrl::Out1LVolume1 {
        mute: false,
        volume: 0x62,
        update: true,
    }
    .serialize(),
];

// feed the mic into the equalizer
// Asp1Rx1 -> Drc1Left -> Out1L
pub const OUTPUT_ENABLE_COMPRESSION: [[u32; 2]; 4] = [
    volume_ctrl::OutputEnable1 { enabled: true }.serialize(),
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Drc1LInput1,
        input_volume: 64,
        status_enabled: false,
        source_select: mixers::InputSourceSelect::Asp1Rx1,
    }
    .serialize(),
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Out1LInput1,
        input_volume: 64,
        status_enabled: false,
        source_select: mixers::InputSourceSelect::Drc1Left,
    }
    .serialize(),
    volume_ctrl::Out1LVolume1 {
        mute: false,
        volume: 0x62,
        update: true,
    }
    .serialize(),
];

pub const FLL_DISABLE: [[u32; 2]; 1] = [fll::Fll1Control1 {
    control_update: false,
    hold: false,
    enabled: false,
}
.serialize()];

pub const FLL_ENABLE: [[u32; 2]; 1] = [fll::Fll1Control1 {
    control_update: false,
    hold: false,
    enabled: true,
}
.serialize()];

pub const PDM_MIC_ENABLE_CONFIGURE: [[u32; 2]; 10] = [
    // set MICBIASes
    voltage::Ldo2Crtl1 {
        output_voltage_select: voltage::Ldo2OutputVoltageSelect::_2p4V,
        discharge: true,
        enabled: true,
    }
    .serialize(),
    voltage::MicBiasCtrl1 {
        has_external_capacitor: false,
        level: voltage::MicBias1VoltageLevel::_2p2V,
        fast_rate: true,
        discharge: true,
        bypass_mode: false,
        enabled: false,
    }
    .serialize(),
    voltage::MicBiasCtrl5 {
        mic_bias_1c_source: voltage::MicBias1Source::MicBiasRegulator,
        mic_bias_1c_discharge: true,
        mic_bias_1c_enabled: false,
        mic_bias_1b_source: voltage::MicBias1Source::VddA,
        mic_bias_1b_discharge: true,
        mic_bias_1b_enabled: true,
        mic_bias_1a_source: voltage::MicBias1Source::MicBiasRegulator,
        mic_bias_1a_discharge: true,
        mic_bias_1a_enabled: false,
    }
    .serialize(),
    // enable IN1L
    input_signal_path_enable::InputControl {
        in2_left_enable: true,
        in2_right_enable: true,
        in1_left_enable: true,
        in1_right_enable: true,
    }
    .serialize(),
    // enable PDM mic as digital input
    input_signal_path_config::Input1Control1 {
        oversample_rate_control:
            input_signal_path_config::OversampleRateControl::Digital3p072MHzOrAnalogHighPerformance,
        mode: input_signal_path_config::InputPath1Mode::DigitalMode,
    }
    .serialize(),
    // un-mute and set gain to 0dB
    input_signal_path_control::InControl2 {
        reg: input_signal_path_control::Reg::In1Left,
        mute: false,
        digital_volume: 128,
        analog_volume: 128,
    }
    .serialize(),
    input_signal_path_control::InControl2 {
        reg: input_signal_path_control::Reg::In1Right,
        mute: false,
        digital_volume: 128,
        analog_volume: 128,
    }
    .serialize(),
    // volume update
    input_signal_path_control::InputControl3 {
        volume_update: true,
    }
    .serialize(),
    // send PDM MIC to I2S Tx
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Asp1Tx1Input1,
        input_volume: 64,
        status_enabled: false,
        source_select: mixers::InputSourceSelect::In1LSignalPath,
    }
    .serialize(),
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Asp1Tx2Input1,
        input_volume: 64,
        status_enabled: false,
        source_select: mixers::InputSourceSelect::In1RSignalPath,
    }
    .serialize(),
];

pub const PDM_MIC_ENABLE_CONFIGURE_PASSTHOUGH: [[u32; 2]; 12] = [
    // set MICBIASes
    voltage::Ldo2Crtl1 {
        output_voltage_select: voltage::Ldo2OutputVoltageSelect::_2p4V,
        discharge: true,
        enabled: true,
    }
    .serialize(),
    voltage::MicBiasCtrl1 {
        has_external_capacitor: false,
        level: voltage::MicBias1VoltageLevel::_2p2V,
        fast_rate: true,
        discharge: true,
        bypass_mode: false,
        enabled: false,
    }
    .serialize(),
    voltage::MicBiasCtrl5 {
        mic_bias_1c_source: voltage::MicBias1Source::MicBiasRegulator,
        mic_bias_1c_discharge: true,
        mic_bias_1c_enabled: false,
        mic_bias_1b_source: voltage::MicBias1Source::VddA,
        mic_bias_1b_discharge: true,
        mic_bias_1b_enabled: true,
        mic_bias_1a_source: voltage::MicBias1Source::MicBiasRegulator,
        mic_bias_1a_discharge: true,
        mic_bias_1a_enabled: false,
    }
    .serialize(),
    // enable IN1L and IN1R
    input_signal_path_enable::InputControl {
        in2_left_enable: false,
        in2_right_enable: false,
        in1_left_enable: true,
        in1_right_enable: true,
    }
    .serialize(),
    // enable PDM mic as digital input
    input_signal_path_config::Input1Control1 {
        oversample_rate_control:
            input_signal_path_config::OversampleRateControl::Digital3p072MHzOrAnalogHighPerformance,
        mode: input_signal_path_config::InputPath1Mode::DigitalMode,
    }
    .serialize(),
    // un-mute and set gain to 0dB
    input_signal_path_control::InControl2 {
        reg: input_signal_path_control::Reg::In1Left,
        mute: false,
        digital_volume: 0x80, // 0db
        analog_volume: 0x80,  // 0db
    }
    .serialize(),
    input_signal_path_control::InControl2 {
        reg: input_signal_path_control::Reg::In1Right,
        mute: false,
        digital_volume: 0x80, // 0db
        analog_volume: 0x80,  // 0db
    }
    .serialize(),
    // volume update
    input_signal_path_control::InputControl3 {
        volume_update: true,
    }
    .serialize(),
    // send MicL + MicR -> Drc1 -> Eq1 -> Out1L
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Drc1LInput1,
        input_volume: 0x40, // 0db
        status_enabled: false,
        source_select: mixers::InputSourceSelect::In1LSignalPath,
    }
    .serialize(),
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Drc1LInput2,
        input_volume: 0x40, // 0db
        status_enabled: false,
        source_select: mixers::InputSourceSelect::In1RSignalPath,
    }
    .serialize(),
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Eq1Input1,
        input_volume: 0x40, // 0db
        status_enabled: false,
        source_select: mixers::InputSourceSelect::Drc1Left,
    }
    .serialize(),
    mixers::InputSource {
        reg: mixers::ImportSourceReg::Out1LInput1,
        //input_volume: 0x30, // -16db
        //input_volume: 0x38, // -8db
        input_volume: 0x40, // 0db
        status_enabled: false,
        source_select: mixers::InputSourceSelect::Eq1,
    }
    .serialize(),
];

// use default frequency bands and increase the volume
// in the last two bands (higher frequency)
pub const EQUALIZER_ENABLE_CONFIGURE: [[u32; 2]; 3] = [
    equalizer::EqControl1 {
        eq1_enabled: true,
        eq2_enabled: false,
        eq3_enabled: false,
        eq4_enabled: false,
    }
    .serialize(),
    equalizer::EqGain1 {
        reg: equalizer::EqGain1Reg::Eq1Gain1,
        band1_gain: equalizer::EqBandGain::ZeroDb,
        band2_gain: equalizer::EqBandGain::ZeroDb,
        band3_gain: equalizer::EqBandGain::ZeroDb,
        band4_gain: equalizer::EqBandGain::Plus06db,
    }
    .serialize(),
    equalizer::EqGain2 {
        reg: equalizer::EqGain2Reg::Eq1Gain2,
        band5_gain: equalizer::EqBandGain::Plus03db,
    }
    .serialize(),
];

pub const COMPRESSION_ENABLE_CONFIGURE: [[u32; 2]; 4] = [
    compression::DrcControl1 {
        reg: compression::DrcControl1Reg::Drc1Control1,
        left_enabled: true,
        right_enabled: false,
    }
    .serialize(),
    compression::DrcControl2 {
        reg: compression::DrcControl2Reg::Drc1Control2,
        anticlip_enabled: true, // should be false when quick_release_enabled is true
        quick_release_enabled: false, // should be false when anticlip_enabled is true
        knee2_output_enabled: false,
        signal_detect_enabled: false,
        signal_detect_mode: compression::SignalDetectMode::PeakThreshold, // ignored
        knee2_input_enabled: false,                                       // enable noise gate
        signal_detect_peak_threshold: compression::SignalDetectPeakThreshold::_12db, // ignored
        signal_detect_rms_threshold: 0,                                   // ignored
        max_gain: compression::MaxGain::_12db,                            // was 24
        min_gain: compression::MinGain::Minus24db,
        gain_decay_rate: compression::GainDecayRate::_23_25ms,
        gain_attack_rate: compression::GainAttackRate::_726us,
    }
    .serialize(),
    compression::DrcControl3 {
        reg: compression::DrcControl3Reg::Drc1Control3,
        compressor_slope_lower: compression::CompressorSlopeLowerRegion::_1div2,
        compressor_slope_upper: compression::CompressorSlopeUpperRegion::_1div8,
        quick_release_decay_rate: compression::QuickReleaseDecayRate::_1_45ms,
        quick_release_threshold: compression::QuickReleaseThreshold::_18db,
        noise_gate_slope: compression::NoiseGateSlope::_4,
        noise_gate_min_gain: compression::NoiseGateMinGain::_12db,
    }
    .serialize(),
    compression::DrcControl4 {
        reg: compression::DrcControl4Reg::Drc1Control4,
        knee1_output_level: 0x0B, // -16.5db
        knee1_input_level: 0x0E,  // -21db
        knee2_output_level: 0x12, // -57db
        knee2_input_level: 0x00,  // -36db
    }
    .serialize(),
];

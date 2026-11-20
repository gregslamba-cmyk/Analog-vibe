use num_traits::Float;

pub use core_dsp::core_dsp_modules::realtime_controls::EnvelopeFollower;
pub use core_dsp::core_dsp_modules::realtime_controls::{BypassCrossfade, LatencyDelay};
pub use core_dsp::core_dsp_modules::resampler_bitdepth::{Oversampler, OversamplingFactor};

pub fn soft_saturate<T: Float>(sample: T, drive: T) -> T {
    core_dsp::core_dsp_modules::nonlinear_stages::soft_clip(sample, drive)
}

pub fn gain_reduction_db<T: Float>(level_db: T, threshold_db: T, ratio: T, knee_db: T) -> T {
    if !level_db.is_finite()
        || !threshold_db.is_finite()
        || !ratio.is_finite()
        || !knee_db.is_finite()
    {
        return T::zero();
    }

    let ratio = ratio.max(T::one());
    let knee_db = knee_db.max(T::zero());
    let two = T::one() + T::one();
    let half_knee = knee_db / two;
    let level_delta = level_db - threshold_db;
    let ratio_slope = T::one() / ratio - T::one();

    if knee_db > T::zero() && level_delta > -half_knee && level_delta < half_knee {
        let knee_position = level_delta + half_knee;
        ratio_slope * knee_position * knee_position / (two * knee_db)
    } else if level_delta >= half_knee {
        ratio_slope * level_delta
    } else {
        T::zero()
    }
}

pub const FET_ATTACK_MIN_MS: f32 = 0.02;
pub const FET_ATTACK_MAX_MS: f32 = 0.8;
pub const FET_RELEASE_MIN_MS: f32 = 50.0;
pub const FET_RELEASE_MAX_MS: f32 = 1100.0;
pub const FET_RATIO_4: f32 = 4.0;
pub const FET_RATIO_8: f32 = 8.0;
pub const FET_RATIO_12: f32 = 12.0;
pub const FET_RATIO_20: f32 = 20.0;

#[cfg(test)]
mod tests {
    use super::{
        EnvelopeFollower, FET_ATTACK_MAX_MS, FET_ATTACK_MIN_MS, FET_RATIO_4, FET_RATIO_8,
        FET_RATIO_12, FET_RATIO_20, FET_RELEASE_MAX_MS, FET_RELEASE_MIN_MS, gain_reduction_db,
        soft_saturate,
    };

    #[test]
    fn historical_1176ln_ranges_are_represented() {
        assert_eq!(FET_ATTACK_MIN_MS, 0.02);
        assert_eq!(FET_ATTACK_MAX_MS, 0.8);
        assert_eq!(FET_RELEASE_MIN_MS, 50.0);
        assert_eq!(FET_RELEASE_MAX_MS, 1100.0);
        assert_eq!(
            [FET_RATIO_4, FET_RATIO_8, FET_RATIO_12, FET_RATIO_20],
            [4.0, 8.0, 12.0, 20.0]
        );
    }

    #[test]
    fn coefficients_follow_the_host_sample_rate() {
        let mut at_44k = EnvelopeFollower::<f32>::default();
        let mut at_96k = EnvelopeFollower::<f32>::default();

        assert!(at_44k.initialize(44_100.0, 0.0002, 0.25));
        assert!(at_96k.initialize(96_000.0, 0.0002, 0.25));
        assert!(at_44k.next(1.0) > at_96k.next(1.0));
    }

    #[test]
    fn envelope_supports_f32_and_f64() {
        let mut single = EnvelopeFollower::<f32>::default();
        let mut double = EnvelopeFollower::<f64>::default();

        assert!(single.initialize(48_000.0, 0.001, 0.2));
        assert!(double.initialize(48_000.0_f64, 0.001, 0.2));
        assert!(single.next(1.0_f32) > 0.0);
        assert!(double.next(1.0_f64) > 0.0);
    }

    #[test]
    fn rejects_invalid_sample_rate() {
        let mut envelope = EnvelopeFollower::<f32>::default();
        assert!(!envelope.initialize(0.0, 0.001, 0.2));
    }

    #[test]
    fn soft_knee_has_no_reduction_below_its_lower_edge() {
        assert_eq!(gain_reduction_db(-22.0_f32, -18.0, 4.0, 6.0), 0.0);
    }

    #[test]
    fn compression_increases_above_the_knee() {
        let at_threshold = gain_reduction_db(-18.0_f32, -18.0, 4.0, 6.0);
        let above_knee = gain_reduction_db(-10.0_f32, -18.0, 4.0, 6.0);

        assert!(at_threshold < 0.0);
        assert!(above_knee < at_threshold);
    }

    #[test]
    fn soft_saturation_is_near_linear_for_small_signals_and_bounded_for_large_ones() {
        let small_signal = soft_saturate(0.01_f32, 0.75);
        let large_signal = soft_saturate(100.0_f32, 0.75);

        assert!((small_signal - 0.01).abs() < 0.0001);
        assert!(large_signal.is_finite() && large_signal < 2.0);
    }

    #[test]
    fn non_finite_values_do_not_escape_dsp_helpers() {
        let mut envelope = EnvelopeFollower::<f32>::default();
        assert!(envelope.initialize(48_000.0, 0.001, 0.2));

        assert_eq!(envelope.next(f32::NAN), 0.0);
        assert_eq!(soft_saturate(f32::INFINITY, 0.75), 0.0);
        assert_eq!(gain_reduction_db(f32::NAN, -18.0, 4.0, 6.0), 0.0);
    }
}

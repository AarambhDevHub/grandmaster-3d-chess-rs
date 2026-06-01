#[derive(Clone, Default)]
pub struct AudioSystem;

impl AudioSystem {
    pub fn speak(&self, text: &str, enabled: bool) {
        if !enabled {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let Some(window) = web_sys::window() else {
                return;
            };
            let Ok(synth) = window.speech_synthesis() else {
                return;
            };
            synth.cancel();
            if let Ok(utterance) = web_sys::SpeechSynthesisUtterance::new_with_text(text) {
                utterance.set_volume(0.8);
                utterance.set_rate(1.0);
                utterance.set_pitch(1.0);
                synth.speak(&utterance);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = text;
        }
    }

    pub fn play_move(&self, enabled: bool) {
        self.play_tone(enabled, 320.0, 120.0, 0.1);
    }

    pub fn play_capture(&self, enabled: bool) {
        self.play_tone(enabled, 130.0, 70.0, 0.18);
    }

    pub fn play_win(&self, enabled: bool) {
        self.play_tone(enabled, 660.0, 440.0, 0.32);
    }

    fn play_tone(&self, enabled: bool, start: f32, end: f32, duration: f64) {
        if !enabled {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let Ok(ctx) = web_sys::AudioContext::new() else {
                return;
            };
            let Ok(osc) = ctx.create_oscillator() else {
                return;
            };
            let Ok(gain) = ctx.create_gain() else {
                return;
            };
            let _ = osc.connect_with_audio_node(&gain);
            let _ = gain.connect_with_audio_node(&ctx.destination());
            let now = ctx.current_time();
            osc.frequency().set_value_at_time(start, now).ok();
            osc.frequency()
                .exponential_ramp_to_value_at_time(end.max(1.0), now + duration)
                .ok();
            gain.gain().set_value_at_time(0.18, now).ok();
            gain.gain()
                .exponential_ramp_to_value_at_time(0.01, now + duration)
                .ok();
            osc.start().ok();
            osc.stop_with_when(now + duration).ok();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (start, end, duration);
        }
    }
}

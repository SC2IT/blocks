use ggez::{audio::SoundData, Context, GameResult};
use std::io::Cursor;

/// sound effect, a replay cuts off the last one
/// ggez Source::play blocks until the audio thread drops the playing sound (~14ms),
/// so each play gets its own player and the last is dropped, which only flags it to stop
pub struct Effect {
    data: SoundData,
    mixer: rodio::mixer::Mixer,
    volume: f32,
    voice: Option<rodio::Player>,
}

impl Effect {
    pub fn new(ctx: &Context, path: &str) -> GameResult<Self> {
        Ok(Effect {
            data: SoundData::new(ctx, path)?,
            mixer: ctx.audio.device().mixer().clone(),
            volume: 1.0,
            voice: None,
        })
    }

    pub fn play(&mut self) {
        match rodio::Decoder::new(Cursor::new(self.data.clone())) {
            Ok(decoder) => {
                let player = rodio::Player::connect_new(&self.mixer);
                player.set_volume(self.volume);
                player.append(decoder);
                self.voice = Some(player);
            }
            Err(err) => log::error!("sfx decode failed: {err}"),
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Some(player) = &self.voice {
            player.set_volume(volume);
        }
    }
}

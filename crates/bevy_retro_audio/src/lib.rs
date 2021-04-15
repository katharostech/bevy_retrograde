//! An audio system for Bevy Retro
//!
//! Currently this is just a thin layer over [Kira] and the design is not well fleshed out, but it
//! will evolve over time as we get a better understanding of our needs and how to integrate audio
//! control through the ECS.
//!
//! [Kira]: https://docs.rs/kira

use bevy::prelude::*;

pub use kira;

mod assets;
pub use assets::*;

mod components;
pub use components::*;

mod systems;
pub(crate) use systems::*;

/// Bevy plugin that adds support for loading LDtk tile maps
#[derive(Default)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // Add audio manager resource
            .insert_non_send_resource(AudioManager::default())
            .add_event::<SoundEvent>();

        // Add asssets and systems
        add_assets(app);
        add_systems(app);
    }
}

pub use events::*;
mod events {
    use super::*;

    #[derive(Debug, Clone)]
    #[allow(clippy::large_enum_variant)]
    pub enum SoundEvent {
        CreateSound(Handle<SoundData>, Sound),
        PlaySound(Sound, PlaySoundSettings),
        PauseSound(Sound, PauseSoundSettings),
        ResumeSound(Sound, ResumeSoundSettings),
        StopSound(Sound, StopSoundSettings),
    }
}

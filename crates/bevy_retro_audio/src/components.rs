use bevy::{ecs::system::SystemParam, prelude::*, reflect::TypeUuid};
use kira::manager::AudioManager as KiraAudioManager;
use uuid::Uuid;

use super::*;

pub use kira::instance::{
    InstanceLoopStart as LoopStart, InstanceSettings as PlaySoundSettings,
    PauseInstanceSettings as PauseSoundSettings, ResumeInstanceSettings as ResumeSoundSettings,
    StopInstanceSettings as StopSoundSettings,
};

#[derive(SystemParam)]
pub struct SoundController<'a> {
    sound_event_writer: EventWriter<'a, SoundEvent>,
}

impl<'a> SoundController<'a> {
    /// Create a new sound that can then be played, paused, resumed, or stopped
    pub fn create_sound(&mut self, sound_data: &Handle<SoundData>) -> Sound {
        // Create a sound handle
        let sound = Sound::new();

        // Send the sound create event
        self.sound_event_writer
            .send(SoundEvent::CreateSound(sound_data.clone(), sound));

        // Return the sound handle
        sound
    }

    pub fn play_sound(&mut self, sound: Sound) {
        self.play_sound_with_settings(sound, Default::default())
    }
    pub fn play_sound_with_settings(&mut self, sound: Sound, settings: PlaySoundSettings) {
        self.sound_event_writer
            .send(SoundEvent::PlaySound(sound, settings));
    }
    pub fn pause_sound(&mut self, sound: Sound) {
        self.pause_sound_with_settings(sound, Default::default())
    }
    pub fn pause_sound_with_settings(&mut self, sound: Sound, settings: PauseSoundSettings) {
        self.sound_event_writer
            .send(SoundEvent::PauseSound(sound, settings));
    }
    pub fn resume_sound(&mut self, sound: Sound) {
        self.resume_sound_with_settings(sound, Default::default())
    }
    pub fn resume_sound_with_settings(&mut self, sound: Sound, settings: ResumeSoundSettings) {
        self.sound_event_writer
            .send(SoundEvent::ResumeSound(sound, settings));
    }
    pub fn stop_sound(&mut self, sound: Sound) {
        self.stop_sound_with_settings(sound, Default::default())
    }
    pub fn stop_sound_with_settings(&mut self, sound: Sound, settings: StopSoundSettings) {
        self.sound_event_writer
            .send(SoundEvent::StopSound(sound, settings));
    }
}

/// A Handle to a sound that can be played, paused, etc.
#[derive(Debug, Clone, TypeUuid, Copy, PartialEq, Eq, Hash)]
#[uuid = "dee749dd-060d-40dd-b2ea-f675018dbfc4"]
pub struct Sound(Uuid);

impl Sound {
    fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// The audio manager
pub(crate) struct AudioManager(pub(crate) KiraAudioManager);

impl Default for AudioManager {
    fn default() -> Self {
        AudioManager(
            KiraAudioManager::new(Default::default()).expect("Could not start audio manager"),
        )
    }
}

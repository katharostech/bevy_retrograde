use bevy::{
    app::{Events, ManualEventReader},
    prelude::*,
    utils::HashMap,
};
use bevy_retro_core::RetroStage;
use kira::sound::handle::SoundHandle as KiraSoundHandle;

use super::*;

/// Add the Ldtk map systems to the app builder
pub(crate) fn add_systems(app: &mut AppBuilder) {
    app.add_system_to_stage(
        RetroStage::Render,
        get_handle_sound_events_system().exclusive_system(),
    );
}

fn get_handle_sound_events_system() -> impl FnMut(&mut World) {
    let mut audio_event_reader = ManualEventReader::<SoundEvent>::default();
    let mut sound_to_handle_map = HashMap::<Sound, KiraSoundHandle>::default();
    let mut pending_events = Vec::<SoundEvent>::new();

    move |world| {
        let world = world.cell();
        let mut audio_manager = world.get_non_send_mut::<AudioManager>().unwrap();
        let audio_events = world.get_resource::<Events<SoundEvent>>().unwrap();
        let mut sound_data_assets = world.get_resource_mut::<Assets<SoundData>>().unwrap();

        let mut handle_event = |event: &SoundEvent| match event {
            SoundEvent::CreateSound(handle, sound) => {
                if let Some(sound_data) = sound_data_assets.remove(handle) {
                    let sound_handle = audio_manager.add_sound(sound_data.0).unwrap();
                    sound_to_handle_map.insert(*sound, sound_handle);

                    true
                } else {
                    false
                }
            }
            SoundEvent::PlaySound(sound, settings) => {
                if let Some(sound_handle) = sound_to_handle_map.get_mut(&sound) {
                    sound_handle.play(*settings).unwrap();
                    true
                } else {
                    false
                }
            }
            SoundEvent::PauseSound(sound, settings) => {
                if let Some(sound_handle) = sound_to_handle_map.get_mut(&sound) {
                    sound_handle.pause(*settings).unwrap();
                    true
                } else {
                    false
                }
            }
            SoundEvent::ResumeSound(sound, settings) => {
                if let Some(sound_handle) = sound_to_handle_map.get_mut(&sound) {
                    sound_handle.resume(*settings).unwrap();
                    true
                } else {
                    false
                }
            }
            SoundEvent::StopSound(sound, settings) => {
                if let Some(sound_handle) = sound_to_handle_map.get_mut(&sound) {
                    sound_handle.stop(*settings).unwrap();
                    true
                } else {
                    false
                }
            }
        };

        let mut new_pending_events = Vec::new();
        for event in pending_events.drain(0..) {
            if !handle_event(&event) {
                new_pending_events.push(event.clone());
            }
        }
        pending_events = new_pending_events;

        for event in audio_event_reader.iter(&audio_events) {
            if !handle_event(event) {
                pending_events.push(event.clone());
            }
        }
    }
}

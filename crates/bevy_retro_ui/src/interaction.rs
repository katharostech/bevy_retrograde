use bevy::{
    app::{Events, ManualEventReader},
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseWheel},
        Input,
    },
    prelude::{KeyCode, World},
    window::{CursorMoved, ReceivedCharacter},
};

use raui::prelude::{
    DefaultInteractionsEngine, DefaultInteractionsEngineResult, InteractionsEngine,
};

pub(crate) struct BevyInteractionsEngine {
    engine: DefaultInteractionsEngine,
    mouse_position: raui::prelude::Vec2,
    keyboard_event_reader: ManualEventReader<KeyboardInput>,
    cursor_moved_event_reader: ManualEventReader<CursorMoved>,
    mouse_button_event_reader: ManualEventReader<MouseButtonInput>,
    mouse_scroll_event_reader: ManualEventReader<MouseWheel>,
    character_input_event_reader: ManualEventReader<ReceivedCharacter>,
}

impl Default for BevyInteractionsEngine {
    fn default() -> Self {
        BevyInteractionsEngine {
            engine: {
                let mut e = DefaultInteractionsEngine::default();
                // Make sure buttons are un-hovered when the mouse moves off of them
                e.deselect_when_no_button_found = true;
                e
            },
            mouse_position: Default::default(),
            keyboard_event_reader: Default::default(),
            cursor_moved_event_reader: Default::default(),
            mouse_button_event_reader: Default::default(),
            mouse_scroll_event_reader: Default::default(),
            character_input_event_reader: Default::default(),
        }
    }
}

impl BevyInteractionsEngine {
    pub fn update(&mut self, world: &mut World, target_size: bevy::math::UVec2) {
        use crate::raui::prelude::*;

        let windows = world.get_resource::<bevy::window::Windows>().unwrap();
        let keyboard_state = world.get_resource::<Input<KeyCode>>().unwrap();

        // Process cursor move events
        let cursor_moved_events = world.get_resource::<Events<CursorMoved>>().unwrap();
        for event in self.cursor_moved_event_reader.iter(&cursor_moved_events) {
            let window = windows.get(event.id).unwrap();
            let coords_mapping = CoordsMapping::new_scaling(
                Rect {
                    left: 0.,
                    right: window.width(),
                    top: 0.,
                    bottom: window.height(),
                },
                CoordsMappingScaling::Fit(Vec2 {
                    x: target_size.x as f32,
                    y: target_size.y as f32,
                }),
            );

            self.mouse_position = coords_mapping.real_to_virtual_vec2(Vec2 {
                x: event.position.x,
                y: window.height() - event.position.y,
            });

            self.engine
                .interact(Interaction::PointerMove(self.mouse_position));
        }

        // Process mouse button events
        let mouse_button_events = world.get_resource::<Events<MouseButtonInput>>().unwrap();
        for event in self.mouse_button_event_reader.iter(&mouse_button_events) {
            let button = match event.button {
                bevy::prelude::MouseButton::Left => raui::prelude::PointerButton::Trigger,
                bevy::prelude::MouseButton::Right => raui::prelude::PointerButton::Context,
                _ => continue,
            };

            self.engine.interact(match event.state {
                bevy::input::ElementState::Pressed => {
                    Interaction::PointerDown(button, self.mouse_position)
                }
                bevy::input::ElementState::Released => {
                    Interaction::PointerUp(button, self.mouse_position)
                }
            });
        }

        // Process mouse scroll events
        let mouse_scroll_events = world.get_resource::<Events<MouseWheel>>().unwrap();
        for event in self.mouse_scroll_event_reader.iter(&mouse_scroll_events) {
            let multiplier = match event.unit {
                bevy::input::mouse::MouseScrollUnit::Line => 10.,
                bevy::input::mouse::MouseScrollUnit::Pixel => 1.,
            };

            let value = Vec2 {
                x: multiplier * event.x,
                y: multiplier * event.y,
            };

            self.engine
                .interact(Interaction::Navigate(NavSignal::Jump(NavJump::Scroll(
                    NavScroll::Units(value, true),
                ))));
        }

        // Process character input events
        let character_input_events = world.get_resource::<Events<ReceivedCharacter>>().unwrap();
        for event in self
            .character_input_event_reader
            .iter(&character_input_events)
        {
            if self.engine.focused_text_input().is_some() {
                self.engine
                    .interact(Interaction::Navigate(NavSignal::TextChange(
                        NavTextChange::InsertCharacter(event.char),
                    )));
            }
        }

        // Process keyboard events
        let keyboard_events = world.get_resource::<Events<KeyboardInput>>().unwrap();
        for event in self.keyboard_event_reader.iter(&keyboard_events) {
            match event.state {
                bevy::input::ElementState::Pressed => {
                    if self.engine.focused_text_input().is_some() {
                        match event.key_code {
                            Some(KeyCode::Left) => {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::TextChange(
                                        NavTextChange::MoveCursorLeft,
                                    )))
                            }
                            Some(KeyCode::Right) => {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::TextChange(
                                        NavTextChange::MoveCursorRight,
                                    )))
                            }
                            Some(KeyCode::Home) => {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::TextChange(
                                        NavTextChange::MoveCursorStart,
                                    )))
                            }
                            Some(KeyCode::End) => {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::TextChange(
                                        NavTextChange::MoveCursorEnd,
                                    )))
                            }
                            Some(KeyCode::Back) => {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::TextChange(
                                        NavTextChange::DeleteLeft,
                                    )))
                            }
                            Some(KeyCode::Delete) => {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::TextChange(
                                        NavTextChange::DeleteRight,
                                    )))
                            }
                            Some(KeyCode::Return) | Some(KeyCode::NumpadEnter) => self
                                .engine
                                .interact(Interaction::Navigate(NavSignal::TextChange(
                                    NavTextChange::NewLine,
                                ))),
                            Some(KeyCode::Escape) => {
                                self.engine.interact(Interaction::Navigate(
                                    NavSignal::FocusTextInput(().into()),
                                ));
                            }
                            _ => {}
                        }
                    } else {
                        let shift_pressed = keyboard_state.pressed(KeyCode::LShift)
                            | keyboard_state.pressed(KeyCode::RShift);
                        match event.key_code {
                            Some(KeyCode::Up) | Some(KeyCode::W) => {
                                self.engine.interact(Interaction::Navigate(NavSignal::Up))
                            }
                            Some(KeyCode::Down) | Some(KeyCode::S) => {
                                self.engine.interact(Interaction::Navigate(NavSignal::Down))
                            }
                            Some(KeyCode::Left) | Some(KeyCode::A) => {
                                if shift_pressed {
                                    self.engine.interact(Interaction::Navigate(NavSignal::Prev));
                                } else {
                                    self.engine.interact(Interaction::Navigate(NavSignal::Left));
                                }
                            }
                            Some(KeyCode::Right) | Some(KeyCode::D) => {
                                if shift_pressed {
                                    self.engine.interact(Interaction::Navigate(NavSignal::Next));
                                } else {
                                    self.engine
                                        .interact(Interaction::Navigate(NavSignal::Right));
                                }
                            }
                            Some(KeyCode::Return)
                            | Some(KeyCode::NumpadEnter)
                            | Some(KeyCode::Space) => {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::Accept(true)));
                            }
                            Some(KeyCode::Escape) => {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::Cancel(true)));
                            }
                            _ => (),
                        }
                    }
                }
                bevy::input::ElementState::Released => {
                    if self.engine.focused_text_input().is_none() {
                        match event.key_code {
                            Some(KeyCode::Return)
                            | Some(KeyCode::NumpadEnter)
                            | Some(KeyCode::Space) => {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::Accept(false)));
                            }
                            Some(KeyCode::Escape) => {
                                self.engine
                                    .interact(Interaction::Navigate(NavSignal::Cancel(false)));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

impl InteractionsEngine<DefaultInteractionsEngineResult, ()> for BevyInteractionsEngine {
    fn perform_interactions(
        &mut self,
        app: &mut raui::prelude::Application,
    ) -> Result<DefaultInteractionsEngineResult, ()> {
        self.engine.perform_interactions(app)
    }
}

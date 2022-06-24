//! 9-Patch style button widget

use bevy::prelude::Handle;

/// Bordered button rendering
///
/// Adapted from <https://docs.rs/egui/0.18.1/src/egui/widgets/button.rs.html>
use bevy_egui::egui::{self, *};

use crate::{BorderedFrame, RetroFont, RetroLabel};

use super::BorderImage;

/// A button rendered with a [`BorderImage`]
pub struct RetroButton<'a> {
    text: &'a str,
    font: &'a Handle<RetroFont>,
    sense: Sense,
    min_size: Vec2,
    default_border: Option<&'a BorderImage>,
    on_hover_border: Option<&'a BorderImage>,
    on_click_border: Option<&'a BorderImage>,
    margin: egui::style::Margin,
    padding: egui::style::Margin,
}

impl<'a> RetroButton<'a> {
    // Create a new button
    #[must_use = "You must call .show() to render the button"]
    pub fn new(text: &'a str, font: &'a Handle<RetroFont>) -> Self {
        Self {
            text,
            font,
            sense: Sense::click(),
            min_size: Vec2::ZERO,
            default_border: None,
            on_hover_border: None,
            on_click_border: None,
            margin: Default::default(),
            padding: Default::default(),
        }
    }

    /// Set the margin. This will be applied on the outside of the border.
    #[must_use = "You must call .show() to render the button"]
    pub fn margin(mut self, margin: bevy::math::Rect<f32>) -> Self {
        self.margin = egui::style::Margin {
            left: margin.left,
            right: margin.right,
            top: margin.top,
            bottom: margin.bottom,
        };

        self
    }

    /// Set the padding. This will be applied on the inside of the border.
    #[must_use = "You must call .show() to render the button"]
    pub fn padding(mut self, margin: bevy::math::Rect<f32>) -> Self {
        self.padding = egui::style::Margin {
            left: margin.left,
            right: margin.right,
            top: margin.top,
            bottom: margin.bottom,
        };

        self
    }

    /// Set the button border image
    #[must_use = "You must call .show() to render the button"]
    pub fn border(mut self, border: &'a BorderImage) -> Self {
        self.default_border = Some(border);
        self
    }

    /// Set a different border to use when hovering over the button
    #[must_use = "You must call .show() to render the button"]
    pub fn on_hover_border(mut self, border: &'a BorderImage) -> Self {
        self.on_hover_border = Some(border);
        self
    }

    /// Set a different border to use when the mouse is clicking on the button
    #[must_use = "You must call .show() to render the button"]
    pub fn on_click_border(mut self, border: &'a BorderImage) -> Self {
        self.on_click_border = Some(border);
        self
    }

    /// By default, buttons senses clicks.
    /// Change this to a drag-button with `Sense::drag()`.
    #[must_use = "You must call .show() to render the button"]
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }

    /// Set the minimum size for the button
    #[must_use = "You must call .show() to render the button"]
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    /// Render the button
    #[must_use = "You must call .show() to render the button"]
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        self.ui(ui)
    }
}

impl<'a> Widget for RetroButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let RetroButton {
            text,
            font,
            sense,
            min_size,
            default_border,
            on_hover_border,
            on_click_border,
            margin,
            padding,
        }: RetroButton = self;

        let total_extra = padding.sum() + margin.sum();

        let wrap_width = ui.available_width() - total_extra.x;
        let label = RetroLabel::new(text, font);
        let label_layout = if let Some(layout) = label.calculate_layout(ui, Some(wrap_width)) {
            layout
        } else {
            return ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
        };

        let mut desired_size = label_layout.size + total_extra;
        desired_size = desired_size.at_least(min_size);

        let (rect, response) = ui.allocate_at_least(desired_size, sense);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, text));

        if ui.is_rect_visible(rect) {
            let mut text_rect = rect;
            text_rect.min += padding.left_top() + margin.left_top();
            text_rect.max -= padding.right_bottom() + margin.right_bottom();
            text_rect.max.x = text_rect.max.x.max(text_rect.min.x);
            text_rect.max.y = text_rect.max.y.max(text_rect.min.y);

            let label_pos = ui
                .layout()
                .align_size_within_rect(label_layout.size, text_rect)
                .min;

            let border = if response.is_pointer_button_down_on() {
                on_click_border.or(default_border)
            } else if response.hovered() {
                on_hover_border.or(default_border)
            } else {
                default_border
            };

            let mut border_rect = rect;
            border_rect.min += margin.left_top();
            border_rect.max -= margin.right_bottom();
            border_rect.max.x = border_rect.max.x.max(border_rect.min.x);
            border_rect.max.y = border_rect.max.y.max(border_rect.min.y);

            if let Some(border) = border {
                ui.painter()
                    .add(BorderedFrame::new(border).paint(border_rect));
            }

            label.paint_at(ui, label_pos, label_layout);
        }

        response
    }
}

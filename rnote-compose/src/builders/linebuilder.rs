use p2d::bounding_volume::{BoundingVolume, AABB};
use piet::RenderContext;

use crate::penhelpers::{PenEvent, PenState};
use crate::penpath::Element;
use crate::shapes::Line;
use crate::style::{drawhelpers, Composer};
use crate::{Shape, Style};

use super::shapebuilderbehaviour::{BuilderProgress, ShapeBuilderCreator};
use super::ShapeBuilderBehaviour;

/// line builder
#[derive(Debug, Clone)]
pub struct LineBuilder {
    /// the start position
    pub start: na::Vector2<f64>,
    /// the current position
    pub current: na::Vector2<f64>,
}

impl ShapeBuilderCreator for LineBuilder {
    fn start(element: Element) -> Self {
        Self {
            start: element.pos,
            current: element.pos,
        }
    }
}

impl ShapeBuilderBehaviour for LineBuilder {
    fn handle_event(&mut self, event: PenEvent) -> BuilderProgress {
        match event {
            PenEvent::Down { element, .. } => {
                self.current = element.pos;
            }
            PenEvent::Up { .. } => {
                return BuilderProgress::Finished(vec![Shape::Line(Line {
                    start: self.start,
                    end: self.current,
                })]);
            }
            PenEvent::Proximity { .. } => {}
            PenEvent::Cancel => {}
        }

        BuilderProgress::InProgress
    }

    fn bounds(&self, style: &Style, zoom: f64) -> AABB {
        self.state_as_line()
            .composed_bounds(style)
            .loosened(drawhelpers::POS_INDICATOR_RADIUS / zoom)
    }

    fn draw_styled(&self, cx: &mut piet_cairo::CairoRenderContext, style: &Style, zoom: f64) {
        cx.save().unwrap();
        let line = self.state_as_line();
        line.draw_composed(cx, style);

        drawhelpers::draw_pos_indicator(cx, PenState::Up, self.start, zoom);
        drawhelpers::draw_pos_indicator(cx, PenState::Down, self.current, zoom);
        cx.restore().unwrap();
    }
}

impl LineBuilder {
    /// The current state as line
    pub fn state_as_line(&self) -> Line {
        Line {
            start: self.start,
            end: self.current,
        }
    }
}

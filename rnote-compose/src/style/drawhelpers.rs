//! Helpers for drawing edit nodes, guides, etc.

use p2d::bounding_volume::{BoundingSphere, BoundingVolume, AABB};
use piet::RenderContext;

use crate::color;
use crate::helpers::{AABBHelpers, Vector2Helpers};
use crate::penhelpers::PenState;

/// ## Pos indicator

/// the radius
pub const POS_INDICATOR_RADIUS: f64 = 5.0;
/// the outline width
pub const POS_INDICATOR_OUTLINE_WIDTH: f64 = 2.0;

/// the pos indicator shape
pub fn pos_indicator_shape(
    _node_state: PenState,
    pos: na::Vector2<f64>,
    zoom: f64,
) -> kurbo::Circle {
    kurbo::Circle::new(
        pos.to_kurbo_point(),
        (POS_INDICATOR_RADIUS - POS_INDICATOR_OUTLINE_WIDTH * 0.5) / zoom,
    )
}

/// Draw a position indicator
pub fn draw_pos_indicator(
    cx: &mut impl RenderContext,
    node_state: PenState,
    pos: na::Vector2<f64>,
    zoom: f64,
) {
    const FILL_COLOR: piet::Color = color::GNOME_REDS[3].with_a8(0xb0);
    const OUTLINE_COLOR: piet::Color = color::GNOME_REDS[4];

    let pos_indicator = pos_indicator_shape(node_state, pos, zoom);

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {}
        PenState::Down => {
            cx.fill(pos_indicator, &FILL_COLOR);
        }
    }
    cx.stroke(
        pos_indicator,
        &OUTLINE_COLOR,
        POS_INDICATOR_OUTLINE_WIDTH / zoom,
    );
}

/// ## Vec indicator

/// the line width
pub const VEC_INDICATOR_LINE_WIDTH: f64 = 1.5;

/// vec indicator shape
pub fn vec_indicator_shape(
    _node_state: PenState,
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    _zoom: f64,
) -> kurbo::Line {
    kurbo::Line::new(start.to_kurbo_point(), end.to_kurbo_point())
}

/// Draw a vec indicator
pub fn draw_vec_indicator(
    cx: &mut impl RenderContext,
    node_state: PenState,
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    zoom: f64,
) {
    let vec_indicator = vec_indicator_shape(node_state, start, end, zoom);

    let line_color = match node_state {
        PenState::Up => color::GNOME_DARKS[0].with_a8(0x80),
        PenState::Proximity => color::GNOME_BRIGHTS[0].with_a8(0xa0),
        PenState::Down => color::GNOME_DARKS[1].with_a8(0xa0),
    };

    cx.stroke(vec_indicator, &line_color, VEC_INDICATOR_LINE_WIDTH / zoom);
}

/// ## Rectangular node

/// the outline width
pub const RECTANGULAR_NODE_OUTLINE_WIDTH: f64 = 1.5;

/// Return the rectangular node shape
pub fn rectangular_node_shape(
    _node_state: PenState,
    bounds: AABB,
    zoom: f64,
) -> kurbo::RoundedRect {
    const CORNER_RADIUS: f64 = 2.0;

    kurbo::RoundedRect::from_rect(
        bounds
            .tightened(RECTANGULAR_NODE_OUTLINE_WIDTH * 0.5 / zoom)
            .to_kurbo_rect(),
        CORNER_RADIUS / zoom,
    )
}

/// Draw a rectangular node
pub fn draw_rectangular_node(
    cx: &mut impl RenderContext,
    node_state: PenState,
    bounds: AABB,
    zoom: f64,
) {
    const OUTLINE_COLOR: piet::Color = color::GNOME_BLUES[4];
    const FILL_COLOR_STATE_DOWN: piet::Color = color::GNOME_BLUES[0].with_a8(0x80);

    let rectangular_node = rectangular_node_shape(node_state, bounds, zoom);

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {}
        PenState::Down => {
            cx.fill(rectangular_node, &FILL_COLOR_STATE_DOWN);
        }
    }

    cx.stroke(
        rectangular_node,
        &OUTLINE_COLOR,
        RECTANGULAR_NODE_OUTLINE_WIDTH / zoom,
    );
}

/// ## Circular Node

/// the outline width
pub const CIRCULAR_NODE_OUTLINE_WIDTH: f64 = 1.5;

/// circular node shape
pub fn circular_node_shape(
    _node_state: PenState,
    mut bounding_sphere: BoundingSphere,
    zoom: f64,
) -> kurbo::Circle {
    bounding_sphere.tighten(CIRCULAR_NODE_OUTLINE_WIDTH * 0.5 / zoom);

    kurbo::Circle::new(
        bounding_sphere.center.coords.to_kurbo_point(),
        bounding_sphere.radius,
    )
}

/// Draw a circular node
pub fn draw_circular_node(
    cx: &mut impl RenderContext,
    node_state: PenState,
    bounding_sphere: BoundingSphere,
    zoom: f64,
) {
    const OUTLINE_COLOR: piet::Color = color::GNOME_BLUES[4];
    const FILL_STATE_DOWN: piet::Color = color::GNOME_BLUES[0].with_a8(0x80);

    let circular_node = circular_node_shape(node_state, bounding_sphere, zoom);

    cx.stroke(
        circular_node,
        &OUTLINE_COLOR,
        CIRCULAR_NODE_OUTLINE_WIDTH / zoom,
    );

    match node_state {
        PenState::Up => {}
        PenState::Proximity => {}
        PenState::Down => {
            cx.fill(circular_node, &FILL_STATE_DOWN);
        }
    }
}

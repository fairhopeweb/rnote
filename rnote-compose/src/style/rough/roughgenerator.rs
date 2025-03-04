use crate::shapes::QuadraticBezier;

use super::roughoptions::RoughOptions;
use rand::Rng;

fn offset<R>(
    min: f64,
    max: f64,
    options: &RoughOptions,
    rng: &mut R,
    roughness_gain: Option<f64>,
) -> f64
where
    R: Rng + ?Sized,
{
    let roughness_gain = roughness_gain.unwrap_or(1.0);
    options.roughness * roughness_gain * (rng.gen_range(0.0..1.0) * (max - min) + min)
}

fn offset_opt<R>(x: f64, options: &RoughOptions, rng: &mut R, roughness_gain: Option<f64>) -> f64
where
    R: Rng + ?Sized,
{
    offset(-x, x, options, rng, roughness_gain)
}

pub(super) fn line<R>(
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    move_to: bool,
    overlay: bool,
    options: &RoughOptions,
    rng: &mut R,
) -> kurbo::BezPath
where
    R: Rng + ?Sized,
{
    let len = (end - start).magnitude();

    let roughness_gain = if len < 200.0 {
        1.0
    } else if len > 500.0 {
        0.4
    } else {
        -0.0016668 * len + 1.233334
    };

    let mut offset = options.max_randomness_offset;
    if offset * offset * 100.0 > (len * len) {
        offset = len / 10.0;
    };
    let half_offset = offset * 0.5;

    let diverge_point = 0.2 + rng.gen_range(0.0..1.0) * 0.2;

    let mid_disp_x = options.bowing * options.max_randomness_offset * (end[1] - start[1]) / 200.0;
    let mid_disp_y = options.bowing * options.max_randomness_offset * (start[0] - end[0]) / 200.0;
    let mid_disp_x = offset_opt(mid_disp_x, options, rng, Some(roughness_gain));
    let mid_disp_y = offset_opt(mid_disp_y, options, rng, Some(roughness_gain));

    let mut bez_path = kurbo::BezPath::new();

    if move_to {
        if overlay {
            let x = start[0]
                + if options.preserve_vertices {
                    0.0
                } else {
                    offset_opt(half_offset, options, rng, Some(roughness_gain))
                };
            let y = start[1]
                + if options.preserve_vertices {
                    0.0
                } else {
                    offset_opt(half_offset, options, rng, Some(roughness_gain))
                };

            bez_path.move_to(kurbo::Point::new(x, y));
        } else {
            let x = start[0]
                + if options.preserve_vertices {
                    0.0
                } else {
                    offset_opt(offset, options, rng, Some(roughness_gain))
                };
            let y = start[1]
                + if options.preserve_vertices {
                    0.0
                } else {
                    offset_opt(offset, options, rng, Some(roughness_gain))
                };

            bez_path.move_to(kurbo::Point::new(x, y));
        }
    }

    if overlay {
        let x2 = end[0]
            + if options.preserve_vertices {
                0.0
            } else {
                offset_opt(half_offset, options, rng, Some(roughness_gain))
            };
        let y2 = end[1]
            + if options.preserve_vertices {
                0.0
            } else {
                offset_opt(half_offset, options, rng, Some(roughness_gain))
            };

        bez_path.curve_to(
            kurbo::Point::new(
                mid_disp_x
                    + start[0]
                    + (end[0] - start[0]) * diverge_point
                    + offset_opt(half_offset, options, rng, Some(roughness_gain)),
                mid_disp_y
                    + start[1]
                    + (end[1] - start[1]) * diverge_point
                    + offset_opt(half_offset, options, rng, Some(roughness_gain)),
            ),
            kurbo::Point::new(
                mid_disp_x
                    + start[0]
                    + 2.0 * (end[0] - start[0]) * diverge_point
                    + offset_opt(half_offset, options, rng, Some(roughness_gain)),
                mid_disp_y
                    + start[1]
                    + 2.0 * (end[1] - start[1]) * diverge_point
                    + offset_opt(half_offset, options, rng, Some(roughness_gain)),
            ),
            kurbo::Point::new(x2, y2),
        );
    } else {
        let x2 = end[0]
            + if options.preserve_vertices {
                0.0
            } else {
                offset_opt(offset, options, rng, Some(roughness_gain))
            };
        let y2 = end[1]
            + if options.preserve_vertices {
                0.0
            } else {
                offset_opt(offset, options, rng, Some(roughness_gain))
            };

        bez_path.curve_to(
            kurbo::Point::new(
                mid_disp_x
                    + start[0]
                    + (end[0] - start[0]) * diverge_point
                    + offset_opt(offset, options, rng, Some(roughness_gain)),
                mid_disp_y
                    + start[1]
                    + (end[1] - start[1]) * diverge_point
                    + offset_opt(offset, options, rng, Some(roughness_gain)),
            ),
            kurbo::Point::new(
                mid_disp_x
                    + start[0]
                    + 2.0 * (end[0] - start[0]) * diverge_point
                    + offset_opt(offset, options, rng, Some(roughness_gain)),
                mid_disp_y
                    + start[1]
                    + 2.0 * (end[1] - start[1]) * diverge_point
                    + offset_opt(offset, options, rng, Some(roughness_gain)),
            ),
            kurbo::Point::new(x2, y2),
        );
    }

    bez_path
}

pub(super) fn doubleline<R>(
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
    options: &RoughOptions,
    rng: &mut R,
) -> kurbo::BezPath
where
    R: Rng + ?Sized,
{
    let mut bez_path = line(start, end, true, false, &options, rng);

    let mut second_options = options.clone();
    second_options.seed = Some(rng.gen::<u64>());

    bez_path.extend(line(start, end, true, true, &second_options, rng).into_iter());

    bez_path
}

pub(super) fn quadratic_bezier<R>(
    start: na::Vector2<f64>,
    cp: na::Vector2<f64>,
    end: na::Vector2<f64>,
    options: &RoughOptions,
    rng: &mut R,
) -> kurbo::BezPath
where
    R: Rng + ?Sized,
{
    // Converting to a cubic, so we can reuse the rough algorithm for it
    let cubbez = QuadraticBezier { start, cp, end }.to_cubic_bezier();

    cubic_bezier(
        cubbez.start,
        cubbez.cp1,
        cubbez.cp2,
        cubbez.end,
        options,
        rng,
    )
}

pub(super) fn cubic_bezier<R>(
    start: na::Vector2<f64>,
    cp1: na::Vector2<f64>,
    cp2: na::Vector2<f64>,
    end: na::Vector2<f64>,
    options: &RoughOptions,
    rng: &mut R,
) -> kurbo::BezPath
where
    R: Rng + ?Sized,
{
    let mut bez_path = kurbo::BezPath::new();

    let ros = [
        options.max_randomness_offset,
        options.max_randomness_offset + 0.3,
    ];

    let iterations = if options.disable_multistroke {
        1_usize
    } else {
        2_usize
    };
    for i in 0..iterations {
        if i == 0 {
            bez_path.move_to(kurbo::Point::new(start[0], start[1]));
        } else {
            let delta = if options.preserve_vertices {
                na::vector![0.0, 0.0]
            } else {
                na::vector![
                    offset_opt(ros[0], options, rng, None),
                    offset_opt(ros[0], options, rng, None)
                ]
            };

            bez_path.move_to(kurbo::Point::new(start[0] + delta[0], start[1] + delta[1]));
        }

        let end_ = if options.preserve_vertices {
            na::vector![end[0], end[1]]
        } else {
            na::vector![
                end[0] + offset_opt(ros[i], options, rng, None),
                end[1] + offset_opt(ros[i], options, rng, None)
            ]
        };

        bez_path.curve_to(
            kurbo::Point::new(
                cp1[0] + offset_opt(ros[i], options, rng, None),
                cp1[1] + offset_opt(ros[i], options, rng, None),
            ),
            kurbo::Point::new(
                cp2[0] + offset_opt(ros[i], options, rng, None),
                cp2[1] + offset_opt(ros[i], options, rng, None),
            ),
            kurbo::Point::new(end_[0], end_[1]),
        );
    }

    bez_path
}

pub(super) fn fill_polygon<R>(
    points: Vec<na::Vector2<f64>>,
    _options: &RoughOptions,
    _rng: &mut R,
) -> kurbo::BezPath
where
    R: Rng + ?Sized,
{
    let mut bez_path = kurbo::BezPath::new();

    for (i, point) in points.iter().enumerate() {
        if i == 0 {
            bez_path.move_to(kurbo::Point::new(point[0], point[1]));
        } else {
            bez_path.line_to(kurbo::Point::new(point[0], point[1]));
        }
    }
    bez_path.close_path();

    bez_path
}

pub(super) fn ellipse<R>(
    center: na::Vector2<f64>,
    mut radius_x: f64,
    mut radius_y: f64,
    options: &RoughOptions,
    rng: &mut R,
) -> EllipseResult
where
    R: Rng + ?Sized,
{
    let mut bez_path = kurbo::BezPath::new();

    // generate ellipse parameters
    let psq =
        (std::f64::consts::PI * 2.0 * (radius_x.powi(2) + radius_y.powi(2)).sqrt() * 0.5).sqrt();
    let stepcount = options
        .curve_stepcount
        .max((options.curve_stepcount / 200.0_f64.sqrt()) * psq)
        .ceil();

    let increment = (std::f64::consts::PI * 2.0) / stepcount;
    let curve_fitrandomness = 1.0 - options.curve_fitting;

    radius_x += offset_opt(radius_x * curve_fitrandomness, options, rng, None);
    radius_y += offset_opt(radius_y * curve_fitrandomness, options, rng, None);

    // creating ellipse
    let overlap_1 = increment
        * self::offset(
            0.1,
            self::offset(0.4, 1.0, options, rng, None),
            options,
            rng,
            None,
        );

    let (all_points_1, core_points_1) = compute_ellipse_points(
        increment, center, radius_x, radius_y, 1.0, overlap_1, options, rng,
    );

    bez_path.extend(curve(all_points_1, None, options, rng).into_iter());

    if !options.disable_multistroke {
        let (all_points_2, _) = compute_ellipse_points(
            increment, center, radius_x, radius_y, 1.5, 0.0, options, rng,
        );

        bez_path.extend(curve(all_points_2, None, options, rng).into_iter());
    }

    EllipseResult {
        estimated_points: core_points_1,
        bez_path,
    }
}

pub(super) fn curve<R>(
    points: Vec<na::Vector2<f64>>,
    close_point: Option<na::Vector2<f64>>,
    options: &RoughOptions,
    rng: &mut R,
) -> kurbo::BezPath
where
    R: Rng + ?Sized,
{
    let mut bez_path = kurbo::BezPath::new();

    let len = points.len();

    if len > 3 {
        let s = 1.0 - options.curve_tightness;

        bez_path.move_to(kurbo::Point::new(points[1][0], points[1][1]));

        let mut i = 1;
        while i + 2 < len {
            let _b0 = points[i];
            let b1 = na::vector![
                points[i][0] + (s + points[i + 1][0] - s * points[i - 1][0]) / 6.0,
                points[i][1] + (s + points[i + 1][1] - s * points[i - 1][1]) / 6.0
            ];
            let b2 = na::vector![
                points[i + 1][0] + (s * points[i][0] - s * points[i + 2][0]) / 6.0,
                points[i + 1][1] + (s * points[i][1] - s * points[i + 2][1]) / 6.0
            ];
            let b3 = points[i + 1];

            bez_path.curve_to(
                kurbo::Point::new(b1[0], b1[1]),
                kurbo::Point::new(b2[0], b2[1]),
                kurbo::Point::new(b3[0], b3[1]),
            );

            i += 1;
        }
        if let Some(close_point) = close_point {
            if close_point.len() == 2 {
                bez_path.line_to(kurbo::Point::new(
                    close_point[0] + offset_opt(options.max_randomness_offset, options, rng, None),
                    close_point[1] + offset_opt(options.max_randomness_offset, options, rng, None),
                ));
            }
        }
    } else if len == 3 {
        bez_path.move_to(kurbo::Point::new(points[1][0], points[1][1]));
        bez_path.curve_to(
            kurbo::Point::new(points[1][0], points[1][1]),
            kurbo::Point::new(points[2][0], points[2][1]),
            kurbo::Point::new(points[2][0], points[2][1]),
        );
    } else if len == 2 {
        bez_path.extend(doubleline(points[0], points[1], options, rng).into_iter());
    }

    bez_path
}

#[derive(Debug, Clone)]
pub struct EllipseResult {
    pub estimated_points: Vec<na::Vector2<f64>>,
    pub bez_path: kurbo::BezPath,
}

// Returns (all_points, core_points)
pub(super) fn compute_ellipse_points<R>(
    increment: f64,
    center: na::Vector2<f64>,
    radius_x: f64,
    radius_y: f64,
    offset: f64,
    overlap: f64,
    options: &RoughOptions,
    rng: &mut R,
) -> (Vec<na::Vector2<f64>>, Vec<na::Vector2<f64>>)
where
    R: Rng + ?Sized,
{
    let mut core_points = Vec::new();
    let mut all_points = Vec::new();

    let rad_offset = offset_opt(0.5, options, rng, None) - std::f64::consts::PI * 0.5;
    all_points.push(na::vector![
        offset_opt(offset, options, rng, None)
            + center[0]
            + 0.9 * radius_x * (rad_offset - increment),
        offset_opt(offset, options, rng, None)
            + center[1]
            + 0.9 * radius_y * (rad_offset - increment)
    ]);

    let mut angle = rad_offset;
    while angle < (std::f64::consts::PI * 2.0 + rad_offset - 0.01) {
        let point = na::vector![
            offset_opt(offset, options, rng, None) + center[0] + radius_x * angle.cos(),
            offset_opt(offset, options, rng, None) + center[1] + radius_y * angle.sin()
        ];

        all_points.push(point);
        core_points.push(point);

        angle += increment;
    }

    all_points.push(na::vector![
        offset_opt(offset, options, rng, None)
            + center[0]
            + radius_x * (rad_offset + std::f64::consts::PI * 2.0 + overlap * 0.5).cos(),
        offset_opt(offset, options, rng, None)
            + center[1]
            + radius_y * (rad_offset + std::f64::consts::PI * 2.0 + overlap * 0.5).sin()
    ]);
    all_points.push(na::vector![
        offset_opt(offset, options, rng, None)
            + center[0]
            + 0.98 * radius_x * (rad_offset + overlap).cos(),
        offset_opt(offset, options, rng, None)
            + center[1]
            + 0.98 * radius_y * (rad_offset + overlap).sin()
    ]);
    all_points.push(na::vector![
        offset_opt(offset, options, rng, None)
            + center[0]
            + 0.9 * radius_x * (rad_offset + overlap * 0.5).cos(),
        offset_opt(offset, options, rng, None)
            + center[1]
            + 0.9 * radius_y * (rad_offset + overlap * 0.5).sin()
    ]);

    (all_points, core_points)
}

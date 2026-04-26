// Copyright 2024 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::util;
use vello_common::kurbo::{Affine, Shape};
use vello_common::peniko::{BlendMode, Fill, Mix};
use vello_hybrid::Scene;

pub(crate) fn render_group<F: FnMut(&mut Scene, &usvg::Node)>(
    scene: &mut Scene,
    group: &usvg::Group,
    transform: Affine,
    error_handler: &mut F,
) {
    for node in group.children() {
        let transform = transform * util::to_affine(&node.abs_transform());
        match node {
            usvg::Node::Group(g) => {
                let alpha = g.opacity().get();
                let blend_mode: BlendMode = match g.blend_mode() {
                    usvg::BlendMode::Normal => Mix::Normal.into(),
                    usvg::BlendMode::Multiply => Mix::Multiply.into(),
                    usvg::BlendMode::Screen => Mix::Screen.into(),
                    usvg::BlendMode::Overlay => Mix::Overlay.into(),
                    usvg::BlendMode::Darken => Mix::Darken.into(),
                    usvg::BlendMode::Lighten => Mix::Lighten.into(),
                    usvg::BlendMode::ColorDodge => Mix::ColorDodge.into(),
                    usvg::BlendMode::ColorBurn => Mix::ColorBurn.into(),
                    usvg::BlendMode::HardLight => Mix::HardLight.into(),
                    usvg::BlendMode::SoftLight => Mix::SoftLight.into(),
                    usvg::BlendMode::Difference => Mix::Difference.into(),
                    usvg::BlendMode::Exclusion => Mix::Exclusion.into(),
                    usvg::BlendMode::Hue => Mix::Hue.into(),
                    usvg::BlendMode::Saturation => Mix::Saturation.into(),
                    usvg::BlendMode::Color => Mix::Color.into(),
                    usvg::BlendMode::Luminosity => Mix::Luminosity.into(),
                };

                let clipped = match g
                    .clip_path()
                    // support clip-path with a single path
                    .and_then(|path| path.root().children().first())
                {
                    Some(usvg::Node::Path(clip_path)) => {
                        let local_path = util::to_bez_path(clip_path);
                        scene.set_transform(transform);
                        scene.set_fill_rule(Fill::NonZero);
                        scene.push_layer(
                            Some(&local_path),
                            Some(blend_mode),
                            Some(alpha),
                            None,
                            None,
                        );
                        true
                    }
                    _ => {
                        // Use bounding box as the clip path.
                        let bounding_box = g.layer_bounding_box();
                        let rect = vello_common::kurbo::Rect::from_origin_size(
                            (bounding_box.x(), bounding_box.y()),
                            (bounding_box.width() as f64, bounding_box.height() as f64),
                        );
                        let clip_bezpath = rect.to_path(0.1);
                        scene.set_transform(transform);
                        scene.set_fill_rule(Fill::NonZero);
                        scene.push_layer(
                            Some(&clip_bezpath),
                            Some(blend_mode),
                            Some(alpha),
                            None,
                            None,
                        );
                        true
                    }
                };

                render_group(scene, g, Affine::IDENTITY, error_handler);

                if clipped {
                    scene.pop_layer();
                }
            }
            usvg::Node::Path(path) => {
                if !path.is_visible() {
                    continue;
                }
                let local_path = util::to_bez_path(path);

                let do_fill = |scene: &mut Scene, error_handler: &mut F| {
                    if let Some(fill) = &path.fill() {
                        if let Some((brush, brush_transform)) =
                            util::to_brush(fill.paint(), fill.opacity())
                        {
                            scene.set_fill_rule(match fill.rule() {
                                usvg::FillRule::NonZero => Fill::NonZero,
                                usvg::FillRule::EvenOdd => Fill::EvenOdd,
                            });
                            scene.set_transform(transform);
                            scene.set_paint(brush);
                            scene.set_paint_transform(brush_transform);
                            scene.fill_path(&local_path);
                        } else {
                            error_handler(scene, node);
                        }
                    }
                };
                let do_stroke = |scene: &mut Scene, error_handler: &mut F| {
                    if let Some(stroke) = &path.stroke() {
                        if let Some((brush, brush_transform)) =
                            util::to_brush(stroke.paint(), stroke.opacity())
                        {
                            let conv_stroke = util::to_stroke(stroke);
                            scene.set_stroke(conv_stroke);
                            scene.set_transform(transform);
                            scene.set_paint(brush);
                            scene.set_paint_transform(brush_transform);
                            scene.stroke_path(&local_path);
                        } else {
                            error_handler(scene, node);
                        }
                    }
                };
                match path.paint_order() {
                    usvg::PaintOrder::FillAndStroke => {
                        do_fill(scene, error_handler);
                        do_stroke(scene, error_handler);
                    }
                    usvg::PaintOrder::StrokeAndFill => {
                        do_stroke(scene, error_handler);
                        do_fill(scene, error_handler);
                    }
                }
            }
            usvg::Node::Image(img) => {
                if !img.is_visible() {
                    continue;
                }
                match img.kind() {
                    usvg::ImageKind::JPEG(_)
                    | usvg::ImageKind::PNG(_)
                    | usvg::ImageKind::GIF(_)
                    | usvg::ImageKind::WEBP(_) => {
                        #[cfg(feature = "image")]
                        {
                            let Ok(decoded_image) = util::decode_raw_raster_image(img.kind())
                            else {
                                error_handler(scene, node);
                                continue;
                            };
                            let width = decoded_image.width();
                            let height = decoded_image.height();
                            let image = util::into_image(decoded_image);
                            let image_ts = util::to_affine(&img.abs_transform());
                            let image_rect = vello_common::kurbo::Rect::new(
                                0.0,
                                0.0,
                                f64::from(width),
                                f64::from(height),
                            );
                            scene.set_transform(image_ts);
                            scene.set_paint(image);
                            scene.fill_rect(&image_rect);
                        }

                        #[cfg(not(feature = "image"))]
                        {
                            error_handler(scene, node);
                            continue;
                        }
                    }
                    usvg::ImageKind::SVG(svg) => {
                        render_group(scene, svg.root(), transform, error_handler);
                    }
                }
            }
            usvg::Node::Text(text) => {
                render_group(scene, text.flattened(), transform, error_handler);
            }
        }
    }
}

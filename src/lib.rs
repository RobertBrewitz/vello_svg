// Copyright 2023 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Render an SVG document to a `vello_hybrid` [`Scene`].
//!
//! This currently lacks support for a [number of important](crate#unsupported-features) SVG features.
//!
//! This is also intended to be the preferred integration between Vello and [usvg], so [consider
//! contributing](https://github.com/linebender/vello_svg) if you need a feature which is missing.
//!
//! This crate also re-exports [`usvg`], [`vello_hybrid`], and [`vello_common`],
//! so you can easily use the specific versions that are compatible with Vello SVG.
//!
//! # Unsupported features
//!
//! Missing features include:
//! - text
//! - group opacity
//! - mix-blend-modes
//! - clipping
//! - masking
//! - filter effects
//! - group background
//! - path shape-rendering
//! - patterns
//! - raster images

// LINEBENDER LINT SET - lib.rs - v1
// See https://linebender.org/wiki/canonical-lints/
// These lints aren't included in Cargo.toml because they
// shouldn't apply to examples and tests
#![warn(unused_crate_dependencies)]
#![warn(clippy::print_stdout, clippy::print_stderr)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// END LINEBENDER LINT SET
// The following lints are part of the Linebender standard set,
// but resolving them has been deferred for now.
// Feel free to send a PR that solves one or more of these.
#![allow(
    missing_docs,
    clippy::shadow_unrelated,
    clippy::missing_errors_doc,
    reason = "Deferred"
)]
#![cfg_attr(test, allow(unused_crate_dependencies, reason = "Deferred"))] // Some dev dependencies are only used in tests

mod render;

mod error;
pub use error::Error;

pub mod util;

/// Re-export `vello_hybrid`.
pub use vello_hybrid;

/// Re-export `vello_common`.
pub use vello_common;

/// Re-export usvg.
pub use usvg;
use vello_common::kurbo::Affine;

use vello_hybrid::Scene;

/// Render a [`Scene`] from an SVG string, with default error handling.
///
/// This will draw a red box over (some) unsupported elements.
#[expect(clippy::cast_possible_truncation, reason = "SVG dimensions are small")]
pub fn render(svg: &str) -> Result<Scene, Error> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opt)?;
    let mut scene = Scene::new(tree.size().width() as u16, tree.size().height() as u16);
    append_tree(&mut scene, &tree);
    Ok(scene)
}

/// Append an SVG to a `vello_hybrid` [`Scene`], with default error handling.
///
/// This will draw a red box over (some) unsupported elements.
pub fn append(scene: &mut Scene, svg: &str) -> Result<(), Error> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opt)?;
    append_tree(scene, &tree);
    Ok(())
}

/// Append an SVG to a `vello_hybrid` [`Scene`], with user-provided error handling logic.
///
/// See the [module level documentation](crate#unsupported-features) for a list of some unsupported svg features
pub fn append_with<F: FnMut(&mut Scene, &usvg::Node)>(
    scene: &mut Scene,
    svg: &str,
    error_handler: &mut F,
) -> Result<(), Error> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opt)?;
    append_tree_with(scene, &tree, error_handler);
    Ok(())
}

/// Render a [`Scene`] from a [`usvg::Tree`], with default error handling.
///
/// This will draw a red box over (some) unsupported elements.
#[expect(clippy::cast_possible_truncation, reason = "SVG dimensions are small")]
pub fn render_tree(svg: &usvg::Tree) -> Scene {
    let mut scene = Scene::new(svg.size().width() as u16, svg.size().height() as u16);
    append_tree(&mut scene, svg);
    scene
}

/// Append an [`usvg::Tree`] to a `vello_hybrid` [`Scene`], with default error handling.
///
/// This will draw a red box over (some) unsupported elements.
pub fn append_tree(scene: &mut Scene, svg: &usvg::Tree) {
    append_tree_with(scene, svg, &mut util::default_error_handler);
}

/// Append an [`usvg::Tree`] to a `vello_hybrid` [`Scene`], with user-provided error handling logic.
///
/// See the [module level documentation](crate#unsupported-features) for a list of some unsupported svg features
pub fn append_tree_with<F: FnMut(&mut Scene, &usvg::Node)>(
    scene: &mut Scene,
    svg: &usvg::Tree,
    error_handler: &mut F,
) {
    render::render_group(scene, svg.root(), Affine::IDENTITY, error_handler);
}

#[cfg(test)]
mod tests {
    // CI will fail unless cargo nextest can execute at least one test per workspace.
    // Delete this dummy test once we have an actual real test.
    #[test]
    fn dummy_test_until_we_have_a_real_test() {}
}

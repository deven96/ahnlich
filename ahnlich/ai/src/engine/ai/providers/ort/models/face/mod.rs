//! The face pipeline: an image in, one embedding per face out.
//!
//! ```text
//! image ─► letterbox 640x640 ─► detect ─► NMS ─► align ─► recognize ─► 512-d
//! ```
//!
//! Detection runs on the letterboxed copy; recognition maps the landmarks back through it
//! and cuts the face from the ORIGINAL, where a face that is small in frame still has its
//! detail. Both halves share one geometry, in [`letterbox`].
//!
//! A change here holds only if the same person still scores > 0.5 cosine and two different
//! people < 0.3: `cargo test -p ai --lib buffalo_l`.

pub(crate) mod align;
pub(crate) mod detect;
pub(crate) mod letterbox;
pub(crate) mod recognize;

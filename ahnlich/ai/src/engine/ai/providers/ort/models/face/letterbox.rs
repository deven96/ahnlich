//! Fitting an image into the detector's square input, and undoing it.
//!
//! One definition, shared by the resize, the bbox metadata and the crop, so they cannot
//! disagree.

/// The image is pasted at a whole-pixel offset, so the truncation is load-bearing: halving
/// in floating point is wrong whenever `target - scaled` is odd.
pub(crate) fn params(src_width: u32, src_height: u32, target: u32) -> Params {
    let scale = (target as f32 / src_width as f32).min(target as f32 / src_height as f32);
    let scaled_width = (src_width as f32 * scale) as u32;
    let scaled_height = (src_height as f32 * scale) as u32;

    Params {
        scale,
        offset_x: ((target - scaled_width) / 2) as f32,
        offset_y: ((target - scaled_height) / 2) as f32,
        scaled_width,
        scaled_height,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Params {
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub scaled_width: u32,
    pub scaled_height: u32,
}

impl Params {
    pub fn to_original(self, [x, y]: [f32; 2]) -> [f32; 2] {
        [
            (x - self.offset_x) / self.scale,
            (y - self.offset_y) / self.scale,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_inverse_round_trips_including_odd_bars() {
        for (w, h) in [
            (4032, 3024),
            (1280, 886),
            (401, 400),
            (999, 1000),
            (640, 640),
        ] {
            let p = params(w, h, 640);

            for point in [
                [0.0, 0.0],
                [w as f32 / 2.0, h as f32 / 3.0],
                [w as f32, h as f32],
            ] {
                let boxed = [
                    point[0] * p.scale + p.offset_x,
                    point[1] * p.scale + p.offset_y,
                ];
                let back = p.to_original(boxed);

                assert!(
                    (back[0] - point[0]).abs() < 1e-3 && (back[1] - point[1]).abs() < 1e-3,
                    "{w}x{h}: {point:?} -> {boxed:?} -> {back:?}"
                );
            }
        }
    }

    #[test]
    fn the_offset_truncates_like_the_paste_does() {
        let p = params(401, 400, 640);
        assert_eq!(p.scaled_height, 638);
        assert_eq!(p.offset_y, 1.0);

        let p = params(999, 1000, 640);
        assert_eq!(p.scaled_width, 639);
        assert_eq!(p.offset_x, 0.0);
    }
}

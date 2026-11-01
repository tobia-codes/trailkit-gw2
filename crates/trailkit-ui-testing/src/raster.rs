//! A small CPU rasterizer for ImGui draw data (memo plan M65).
//!
//! ImGui's output is a list of textured, vertex-colored triangles plus one
//! font atlas texture. This module draws such triangles into an RGBA image:
//! pixel centers inside a triangle (top-left fill rule) get the texture color
//! (nearest sample) multiplied by the interpolated vertex color, blended with
//! straight alpha over what is already there. It is not pixel-identical to
//! GW2's DirectX renderer, but positions, sizes and colors come straight from
//! ImGui, which is what layout tests need. Platform-independent and
//! deterministic.

/// An RGBA image with straight (not premultiplied) alpha, row by row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width × height` pixels, top row first.
    pub pixels: Vec<[u8; 4]>,
}

/// A texture to sample from: RGBA, straight alpha, row by row.
#[derive(Debug, Clone, Copy)]
pub struct Texture<'data> {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width × height × 4` bytes.
    pub rgba: &'data [u8],
}

/// One triangle corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// Screen position in pixels.
    pub position: [f32; 2],
    /// Texture coordinate (0..1).
    pub uv: [f32; 2],
    /// Vertex color, straight RGBA.
    pub color: [u8; 4],
}

impl Image {
    /// An image of `width × height` filled with `background`.
    #[must_use]
    pub fn new(width: u32, height: u32, background: [u8; 4]) -> Self {
        let count = usize::try_from(u64::from(width) * u64::from(height)).unwrap_or(0);
        Self {
            width,
            height,
            pixels: vec![background; count],
        }
    }

    /// The pixel at `(x, y)`, if inside the image.
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.pixels
            .get(usize::try_from(u64::from(y) * u64::from(self.width) + u64::from(x)).ok()?)
            .copied()
    }

    /// Draws the triangle `corners`, clipped to `clip` (`[left, top, right,
    /// bottom]` in pixels), sampling `texture`.
    pub fn draw_triangle(&mut self, corners: [Vertex; 3], clip: [f32; 4], texture: &Texture<'_>) {
        let [a, b, c] = corners;
        let area = edge(a.position, b.position, c.position);
        if area.abs() < f32::EPSILON {
            return;
        }
        // Orient counter-clockwise in screen space so all edge values are ≥ 0 inside.
        let (b, c, area) = if area < 0.0 {
            (c, b, -area)
        } else {
            (b, c, area)
        };
        let left = clip[0]
            .max(a.position[0].min(b.position[0]).min(c.position[0]))
            .max(0.0);
        let top = clip[1]
            .max(a.position[1].min(b.position[1]).min(c.position[1]))
            .max(0.0);
        let right = clip[2]
            .min(a.position[0].max(b.position[0]).max(c.position[0]))
            .min(to_f32(self.width));
        let bottom = clip[3]
            .min(a.position[1].max(b.position[1]).max(c.position[1]))
            .min(to_f32(self.height));
        if left >= right || top >= bottom {
            return;
        }
        let edges = [(b, c), (c, a), (a, b)];
        let top_left = edges.map(|(from, to)| is_top_left(from.position, to.position));
        for y in to_index(top)..to_index(bottom.ceil()) {
            for x in to_index(left)..to_index(right.ceil()) {
                let center = [to_f32(x) + 0.5, to_f32(y) + 0.5];
                if center[0] < clip[0]
                    || center[0] >= clip[2]
                    || center[1] < clip[1]
                    || center[1] >= clip[3]
                {
                    continue;
                }
                let weights = edges.map(|(from, to)| edge(from.position, to.position, center));
                let inside = weights
                    .iter()
                    .zip(top_left)
                    .all(|(weight, is_top_left)| *weight > 0.0 || (*weight == 0.0 && is_top_left));
                if !inside {
                    continue;
                }
                let [wa, wb, wc] = weights.map(|weight| weight / area);
                let uv = [
                    wa * a.uv[0] + wb * b.uv[0] + wc * c.uv[0],
                    wa * a.uv[1] + wb * b.uv[1] + wc * c.uv[1],
                ];
                let color: [f32; 4] = std::array::from_fn(|channel| {
                    wa * f32::from(a.color[channel])
                        + wb * f32::from(b.color[channel])
                        + wc * f32::from(c.color[channel])
                });
                let sample = texture.sample(uv);
                let source: [f32; 4] = std::array::from_fn(|channel| {
                    color[channel] * f32::from(sample[channel]) / 255.0
                });
                self.blend(x, y, source);
            }
        }
    }

    /// Blends `source` (straight RGBA, 0..255 floats) over the pixel.
    fn blend(&mut self, x: u32, y: u32, source: [f32; 4]) {
        let Ok(index) = usize::try_from(u64::from(y) * u64::from(self.width) + u64::from(x)) else {
            return;
        };
        let Some(pixel) = self.pixels.get_mut(index) else {
            return;
        };
        let alpha = source[3] / 255.0;
        let destination_alpha = f32::from(pixel[3]) / 255.0;
        let out_alpha = alpha + destination_alpha * (1.0 - alpha);
        for channel in 0..3 {
            let blended = if out_alpha > 0.0 {
                (source[channel] * alpha
                    + f32::from(pixel[channel]) * destination_alpha * (1.0 - alpha))
                    / out_alpha
            } else {
                0.0
            };
            pixel[channel] = to_byte(blended);
        }
        pixel[3] = to_byte(out_alpha * 255.0);
    }
}

impl Texture<'_> {
    /// The texel nearest to `uv` (clamped to the edge).
    fn sample(&self, uv: [f32; 2]) -> [u8; 4] {
        let x = to_index((uv[0] * to_f32(self.width)).floor()).min(self.width.saturating_sub(1));
        let y = to_index((uv[1] * to_f32(self.height)).floor()).min(self.height.saturating_sub(1));
        let index = usize::try_from((u64::from(y) * u64::from(self.width) + u64::from(x)) * 4)
            .unwrap_or(usize::MAX);
        match self.rgba.get(index..index + 4) {
            Some([red, green, blue, alpha]) => [*red, *green, *blue, *alpha],
            _ => [255; 4],
        }
    }
}

/// Twice the signed area of the triangle `(from, to, point)`; positive when
/// `point` lies left of `from → to` in a y-down coordinate system turned
/// counter-clockwise.
fn edge(from: [f32; 2], to: [f32; 2], point: [f32; 2]) -> f32 {
    (to[0] - from[0]) * (point[1] - from[1]) - (to[1] - from[1]) * (point[0] - from[0])
}

/// Top-left fill rule: pixels exactly on a top or left edge belong to the
/// triangle, so shared edges are drawn once.
fn is_top_left(from: [f32; 2], to: [f32; 2]) -> bool {
    let is_top = (to[1] - from[1]).abs() < f32::EPSILON && to[0] < from[0];
    let is_left = to[1] > from[1];
    is_top || is_left
}

/// A pixel coordinate as `f32` (images are far smaller than 2^24 pixels).
#[expect(clippy::cast_precision_loss, reason = "image sizes are small")]
const fn to_f32(value: u32) -> f32 {
    value as f32
}

/// A non-negative float coordinate as a pixel index (saturating).
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to non-negative image coordinates"
)]
fn to_index(value: f32) -> u32 {
    value.max(0.0) as u32
}

/// A 0..255 float as a byte (rounded, clamped).
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to 0..=255"
)]
fn to_byte(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 1×1 opaque white texture.
    const WHITE: [u8; 4] = [255; 4];

    /// A vertex at `position` with `color` and the white texel.
    fn vertex(position: [f32; 2], color: [u8; 4]) -> Vertex {
        Vertex {
            position,
            uv: [0.0, 0.0],
            color,
        }
    }

    /// Draws an axis-aligned rectangle as two triangles, like ImGui does.
    fn rectangle(image: &mut Image, min: [f32; 2], max: [f32; 2], color: [u8; 4]) {
        let texture = Texture {
            width: 1,
            height: 1,
            rgba: &WHITE,
        };
        let clip = [0.0, 0.0, 1000.0, 1000.0];
        let corners = [
            vertex(min, color),
            vertex([max[0], min[1]], color),
            vertex(max, color),
            vertex([min[0], max[1]], color),
        ];
        image.draw_triangle([corners[0], corners[1], corners[2]], clip, &texture);
        image.draw_triangle([corners[0], corners[2], corners[3]], clip, &texture);
    }

    #[test]
    fn rectangles_cover_exactly_their_pixels_once() {
        let mut image = Image::new(4, 4, [0, 0, 0, 255]);
        rectangle(&mut image, [1.0, 1.0], [3.0, 3.0], [255, 0, 0, 128]);
        assert_eq!(image.pixel(0, 0), Some([0, 0, 0, 255]));
        // Half-transparent red over black, drawn once despite the shared diagonal.
        assert_eq!(image.pixel(1, 1), Some([128, 0, 0, 255]));
        assert_eq!(image.pixel(2, 2), Some([128, 0, 0, 255]));
        assert_eq!(image.pixel(3, 3), Some([0, 0, 0, 255]));
    }

    #[test]
    fn clipping_limits_drawing() {
        let mut image = Image::new(4, 1, [0, 0, 0, 255]);
        let texture = Texture {
            width: 1,
            height: 1,
            rgba: &WHITE,
        };
        let white = [255; 4];
        image.draw_triangle(
            [
                vertex([0.0, -1.0], white),
                vertex([8.0, -1.0], white),
                vertex([0.0, 8.0], white),
            ],
            [0.0, 0.0, 2.0, 1.0],
            &texture,
        );
        assert_eq!(image.pixel(1, 0), Some(white));
        assert_eq!(image.pixel(2, 0), Some([0, 0, 0, 255]));
    }
}

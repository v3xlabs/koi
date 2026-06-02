use image::{DynamicImage, Rgba, RgbaImage};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Hsl {
    hue: f64,
    saturation: f64,
    lightness: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BloImage {
    data: [u8; 32],
    background: Hsl,
    color: Hsl,
    spot: Hsl,
}

pub fn blo_image(address: &str) -> BloImage {
    let mut seed = rand_seed(&address.to_lowercase());
    let color = random_color(&mut seed);
    let background = random_color(&mut seed);
    let spot = random_color(&mut seed);

    let mut data = [0u8; 32];
    for value in &mut data {
        *value = (next_random(&mut seed) * 2.3).floor() as u8;
    }

    BloImage {
        data,
        background,
        color,
        spot,
    }
}

pub(crate) const BLO_GRID: u32 = 8;

pub fn blo_dynamic_image(address: &str) -> DynamicImage {
    let grid = blo_rgb_grid(address);
    let mut image = RgbaImage::new(BLO_GRID, BLO_GRID);

    for y in 0..BLO_GRID {
        for x in 0..BLO_GRID {
            let [r, g, b] = grid[y as usize][x as usize];
            image.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    DynamicImage::ImageRgba8(image)
}

pub(crate) fn blo_rgb_grid(address: &str) -> [[ [u8; 3]; 8]; 8] {
    let image = blo_image(address);
    let background = hsl_to_rgb_bytes(image.background);
    let color = hsl_to_rgb_bytes(image.color);
    let spot = hsl_to_rgb_bytes(image.spot);

    let mut grid = [[background; 8]; 8];
    for (index, value) in image.data.into_iter().enumerate() {
        if value == 0 {
            continue;
        }

        let x = index & 3;
        let y = index >> 2;
        let pixel = match value {
            1 => color,
            2 => spot,
            _ => background,
        };
        grid[y as usize][x as usize] = pixel;
        grid[y as usize][7 - x] = pixel;
    }

    grid
}

fn hsl_to_rgb_bytes(hsl: Hsl) -> [u8; 3] {
    let (red, green, blue) = hsl_to_rgb(hsl.hue, hsl.saturation, hsl.lightness);
    [red, green, blue]
}

pub fn blo_grid(address: &str) -> [[Color; 8]; 8] {
    blo_rgb_grid(address)
        .map(|row| row.map(|[red, green, blue]| Color::Rgb(red, green, blue)))
}

pub fn blo_icon_lines(address: &str) -> Vec<Line<'static>> {
    let grid = blo_grid(address);
    (0..8)
        .step_by(2)
        .map(|row| {
            Line::from(
                (0..8)
                    .map(|col| Span::styled("█", Style::default().fg(grid[row][col])))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

pub fn blo_icon_line(address: &str) -> Line<'static> {
    let grid = blo_grid(address);
    Line::from(
        [0, 2, 4, 6]
            .into_iter()
            .map(|col| {
                let sample = average_color([
                    grid[3][col],
                    grid[3][col + 1],
                    grid[4][col],
                    grid[4][col + 1],
                ]);
                Span::styled("█", Style::default().fg(sample))
            })
            .collect::<Vec<_>>(),
    )
}

fn rand_seed(seed: &str) -> [u32; 4] {
    let mut rseed = [0u32; 4];
    for (index, byte) in seed.bytes().enumerate() {
        let slot = index % 4;
        rseed[slot] = rseed[slot]
            .wrapping_shl(5)
            .wrapping_sub(rseed[slot])
            .wrapping_add(u32::from(byte));
    }
    rseed
}

fn next_random(rseed: &mut [u32; 4]) -> f64 {
    const RANDOM_SCALE: f64 = 1.0 / ((1u64 << 31) as f64);
    let t = (rseed[0] as i32) ^ (rseed[0] as i32).wrapping_shl(11);
    rseed[0] = rseed[1];
    rseed[1] = rseed[2];
    rseed[2] = rseed[3];
    let current = rseed[3] as i32;
    rseed[3] = (current ^ (current >> 19) ^ t ^ (t >> 8)) as u32;
    f64::from(rseed[3]) * RANDOM_SCALE
}

fn random_color(rseed: &mut [u32; 4]) -> Hsl {
    Hsl {
        hue: truncate_u16(next_random(rseed) * 360.0),
        saturation: truncate_u16(40.0 + next_random(rseed) * 60.0),
        lightness: truncate_u16(
            (next_random(rseed)
                + next_random(rseed)
                + next_random(rseed)
                + next_random(rseed))
                * 25.0,
        ),
    }
}

fn truncate_u16(value: f64) -> f64 {
    value.trunc() // Uint16Array assignment truncates toward zero
}

fn hsl_to_color(hsl: Hsl) -> Color {
    let (red, green, blue) = hsl_to_rgb(hsl.hue, hsl.saturation, hsl.lightness);
    Color::Rgb(red, green, blue)
}

fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> (u8, u8, u8) {
    let saturation = (saturation / 100.0).clamp(0.0, 1.0);
    let lightness = (lightness / 100.0).clamp(0.0, 1.0);

    if saturation == 0.0 {
        let value = (lightness * 255.0).round() as u8;
        return (value, value, value);
    }

    let hue = (hue % 360.0) / 360.0;
    let q = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - lightness * saturation
    };
    let p = 2.0 * lightness - q;

    let red = hue_to_rgb(p, q, hue + 1.0 / 3.0);
    let green = hue_to_rgb(p, q, hue);
    let blue = hue_to_rgb(p, q, hue - 1.0 / 3.0);

    (
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8,
    )
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

fn average_color(colors: [Color; 4]) -> Color {
    let mut red = 0u32;
    let mut green = 0u32;
    let mut blue = 0u32;

    for color in colors {
        if let Color::Rgb(r, g, b) = color {
            red += u32::from(r);
            green += u32::from(g);
            blue += u32::from(b);
        }
    }

    Color::Rgb(
        (red / 4) as u8,
        (green / 4) as u8,
        (blue / 4) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::blo_image;

    #[test]
    fn matches_blo_reference_output() {
        let image = blo_image("0xd8dA6BF26964aF9D7eEd9e03E53415C3377Fc63");
        assert_eq!(
            image.data,
            [
                0, 0, 0, 0, 1, 0, 2, 1, 1, 0, 0, 1, 1, 0, 0, 2, 1, 2, 2, 0, 0, 2, 0, 0, 1, 1, 0,
                0, 0, 1, 1, 1
            ]
        );
        assert_eq!(image.background.hue.round() as u16, 17);
        assert_eq!(image.background.saturation.round() as u16, 76);
        assert_eq!(image.background.lightness.round() as u16, 45);
        assert_eq!(image.color.hue.round() as u16, 238);
        assert_eq!(image.color.saturation.round() as u16, 99);
        assert_eq!(image.color.lightness.round() as u16, 54);
        assert_eq!(image.spot.hue.round() as u16, 17);
        assert_eq!(image.spot.saturation.round() as u16, 81);
        assert_eq!(image.spot.lightness.round() as u16, 42);
    }
}

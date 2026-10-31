//! Find where a crop came from: slide the crop over a frame at many scales and
//! report the best match (zero-mean normalised correlation), as fractions of
//! the frame, ready for `apature-cli crop --rect`. Coarse search on a small
//! copy, then a fine search around the best place and scale on a larger one.

use apature_preview::decode::decode_jpeg;

pub struct Gray {
    pub w: usize,
    pub h: usize,
    pub v: Vec<f32>,
}

pub fn load(bytes: &[u8], long_side: u32) -> anyhow::Result<Gray> {
    let img = decode_jpeg(bytes, Some(long_side))?;
    let (w, h) = (img.width as usize, img.height as usize);
    let v = img
        .data
        .chunks_exact(3)
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .collect();
    Ok(Gray { w, h, v })
}

/// Area-average resize.
pub fn resize(g: &Gray, tw: usize, th: usize) -> Gray {
    let (tw, th) = (tw.max(1), th.max(1));
    let mut v = vec![0f32; tw * th];
    for y in 0..th {
        let (y0, y1) = (
            y * g.h / th,
            ((y + 1) * g.h / th).max(y * g.h / th + 1).min(g.h),
        );
        for x in 0..tw {
            let (x0, x1) = (
                x * g.w / tw,
                ((x + 1) * g.w / tw).max(x * g.w / tw + 1).min(g.w),
            );
            let mut s = 0f32;
            for yy in y0..y1 {
                for xx in x0..x1 {
                    s += g.v[yy * g.w + xx];
                }
            }
            v[y * tw + x] = s / ((y1 - y0) * (x1 - x0)) as f32;
        }
    }
    Gray { w: tw, h: th, v }
}

struct Integral {
    w: usize,
    s: Vec<f64>,
    s2: Vec<f64>,
}

impl Integral {
    fn new(g: &Gray) -> Integral {
        let w = g.w + 1;
        let mut s = vec![0f64; w * (g.h + 1)];
        let mut s2 = vec![0f64; w * (g.h + 1)];
        for y in 0..g.h {
            let (mut rs, mut rs2) = (0f64, 0f64);
            for x in 0..g.w {
                let p = g.v[y * g.w + x] as f64;
                rs += p;
                rs2 += p * p;
                s[(y + 1) * w + x + 1] = s[y * w + x + 1] + rs;
                s2[(y + 1) * w + x + 1] = s2[y * w + x + 1] + rs2;
            }
        }
        Integral { w, s, s2 }
    }
    /// (sum, sum of squares) over the window at (x, y) of size tw × th.
    fn win(&self, x: usize, y: usize, tw: usize, th: usize) -> (f64, f64) {
        let f = |a: &Vec<f64>| {
            a[(y + th) * self.w + x + tw] - a[y * self.w + x + tw] - a[(y + th) * self.w + x]
                + a[y * self.w + x]
        };
        (f(&self.s), f(&self.s2))
    }
}

struct Template {
    z: Vec<f32>, // zero-mean
    norm: f64,   // sqrt(sum z^2)
}

fn template(t: &Gray) -> Template {
    let n = t.v.len() as f64;
    let mean = t.v.iter().map(|&p| p as f64).sum::<f64>() / n;
    let z: Vec<f32> = t.v.iter().map(|&p| (p as f64 - mean) as f32).collect();
    let norm = z
        .iter()
        .map(|&p| (p as f64) * (p as f64))
        .sum::<f64>()
        .sqrt();
    Template { z, norm }
}

fn zncc(f: &Gray, ii: &Integral, t: &Template, tw: usize, th: usize, x: usize, y: usize) -> f64 {
    let n = (tw * th) as f64;
    let (s, s2) = ii.win(x, y, tw, th);
    let var = s2 - s * s / n;
    if var <= 1e-6 || t.norm <= 1e-6 {
        return -1.0;
    }
    let mut dot = 0f64;
    for ty in 0..th {
        let row = &f.v[(y + ty) * f.w + x..(y + ty) * f.w + x + tw];
        let zr = &t.z[ty * tw..(ty + 1) * tw];
        let mut d = 0f32;
        for (a, b) in row.iter().zip(zr) {
            d += a * b;
        }
        dot += d as f64;
    }
    dot / (var.sqrt() * t.norm)
}

#[derive(Clone, Copy, Debug)]
pub struct Found {
    pub score: f64,
    /// top, left, bottom, right as fractions of the frame.
    pub rect: (f32, f32, f32, f32),
}

/// Best place and size of `crop` inside `frame` (JPEG bytes of each).
pub fn find(crop: &[u8], frame: &[u8]) -> anyhow::Result<Found> {
    let c = load(crop, 4000)?;
    let big = load(frame, 1280)?;
    let aspect = c.h as f64 / c.w as f64;
    // coarse
    let cw0 = 160usize;
    let f0 = resize(&big, cw0, (big.h * cw0 / big.w).max(1));
    let ii0 = Integral::new(&f0);
    let mut best = (-2.0f64, 0f64, 0usize, 0usize);
    let mut frac = 0.15f64;
    while frac <= 1.0001 {
        let tw = (cw0 as f64 * frac).round() as usize;
        let th = (tw as f64 * aspect).round() as usize;
        if tw >= 8 && th >= 8 && tw <= f0.w && th <= f0.h {
            let t = template(&resize(&c, tw, th));
            for y in 0..=(f0.h - th) {
                for x in 0..=(f0.w - tw) {
                    let s = zncc(&f0, &ii0, &t, tw, th, x, y);
                    if s > best.0 {
                        best = (s, frac, x, y);
                    }
                }
            }
        }
        frac += 0.02;
    }
    // fine
    let fw = 640usize;
    let f1 = resize(&big, fw, (big.h * fw / big.w).max(1));
    let ii1 = Integral::new(&f1);
    let k = fw as f64 / cw0 as f64;
    let (cx, cy) = ((best.2 as f64 * k) as i64, (best.3 as f64 * k) as i64);
    let mut fine = (-2.0f64, 0usize, 0usize, 0usize, 0usize);
    let mut fr = (best.1 - 0.03).max(0.1);
    while fr <= (best.1 + 0.03).min(1.0) {
        let tw = (fw as f64 * fr).round() as usize;
        let th = (tw as f64 * aspect).round() as usize;
        if tw >= 16 && th >= 16 && tw <= f1.w && th <= f1.h {
            let t = template(&resize(&c, tw, th));
            for dy in -8i64..=8 {
                for dx in -8i64..=8 {
                    let (x, y) = (cx + dx, cy + dy);
                    if x < 0 || y < 0 || x as usize + tw > f1.w || y as usize + th > f1.h {
                        continue;
                    }
                    let s = zncc(&f1, &ii1, &t, tw, th, x as usize, y as usize);
                    if s > fine.0 {
                        fine = (s, x as usize, y as usize, tw, th);
                    }
                }
            }
        }
        fr += 0.003;
    }
    let (s, x, y, tw, th) = fine;
    Ok(Found {
        score: s,
        rect: (
            y as f32 / f1.h as f32,
            x as f32 / f1.w as f32,
            (y + th) as f32 / f1.h as f32,
            (x + tw) as f32 / f1.w as f32,
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_crop_is_found_where_it_was_cut() {
        // A synthetic textured frame, cut at known fractions.
        let (w, h) = (320usize, 214usize);
        let v: Vec<f32> = (0..w * h)
            .map(|i| {
                let (x, y) = ((i % w) as f32, (i / w) as f32);
                128.0 + 90.0 * ((x * 0.21).sin() * (y * 0.17).cos()) + 30.0 * ((x + y) * 0.05).sin()
            })
            .collect();
        let g = Gray { w, h, v };
        let ii = Integral::new(&g);
        let (tx, ty, tw, th) = (90usize, 50usize, 120usize, 80usize);
        let sub = Gray {
            w: tw,
            h: th,
            v: (0..th)
                .flat_map(|yy| (0..tw).map(move |xx| (xx, yy)))
                .map(|(xx, yy)| g.v[(ty + yy) * w + tx + xx])
                .collect(),
        };
        let t = template(&sub);
        let s = zncc(&g, &ii, &t, tw, th, tx, ty);
        assert!(s > 0.999, "{s}");
        assert!(zncc(&g, &ii, &t, tw, th, tx + 7, ty + 3) < s);
    }
}

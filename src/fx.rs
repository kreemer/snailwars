//! Short-lived visual effects: laser beams and impact puffs. They have no
//! gameplay effect; damage is applied when they are spawned.

use macroquad::prelude::*;

/// How long a laser beam stays visible, in seconds.
pub const LASER_BEAM_DURATION: f32 = 1.0;
const IMPACT_DURATION: f32 = 0.25;

/// A laser flash from a tower to the point it hit. The laser damages
/// instantly; the beam stays put and fades out.
pub struct LaserBeam {
    pub from: Vec2,
    pub to: Vec2,
    age: f32,
}

impl LaserBeam {
    pub fn new(from: Vec2, to: Vec2) -> Self {
        LaserBeam { from, to, age: 0.0 }
    }

    pub fn update(&mut self, dt: f32) {
        self.age += dt;
    }

    pub fn is_expired(&self) -> bool {
        self.age >= LASER_BEAM_DURATION
    }

    /// Opacity: full for the first fifth, then fading linearly to zero.
    fn alpha(&self) -> f32 {
        let t = (self.age / LASER_BEAM_DURATION).clamp(0.0, 1.0);
        if t < 0.2 { 1.0 } else { 1.0 - (t - 0.2) / 0.8 }
    }

    pub fn draw(&self) {
        let a = self.alpha();
        let (f, t) = (self.from, self.to);
        draw_line(
            f.x,
            f.y,
            t.x,
            t.y,
            10.0,
            Color::new(0.7, 0.2, 1.0, 0.25 * a),
        );
        draw_line(f.x, f.y, t.x, t.y, 5.0, Color::new(0.85, 0.4, 1.0, 0.6 * a));
        draw_line(f.x, f.y, t.x, t.y, 2.0, Color::new(1.0, 0.95, 1.0, a));
        draw_circle(t.x, t.y, 6.0, Color::new(0.9, 0.5, 1.0, 0.7 * a));
        draw_circle(f.x, f.y, 4.0, Color::new(1.0, 0.9, 1.0, 0.8 * a));
    }
}

/// An expanding, fading ring where a projectile landed.
pub struct Impact {
    pub pos: Vec2,
    color: Color,
    radius: f32,
    age: f32,
}

impl Impact {
    pub fn new(pos: Vec2, color: Color, radius: f32) -> Self {
        Impact {
            pos,
            color,
            radius,
            age: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.age += dt;
    }

    pub fn is_expired(&self) -> bool {
        self.age >= IMPACT_DURATION
    }

    pub fn draw(&self) {
        let t = (self.age / IMPACT_DURATION).clamp(0.0, 1.0);
        let r = self.radius * (0.4 + 0.6 * t);
        let mut fill = self.color;
        fill.a *= 0.35 * (1.0 - t);
        let mut ring = self.color;
        ring.a *= 1.0 - t;
        draw_circle(self.pos.x, self.pos.y, r, fill);
        draw_circle_lines(self.pos.x, self.pos.y, r, 2.0, ring);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn laser_beam_lasts_one_second() {
        let mut beam = LaserBeam::new(Vec2::ZERO, Vec2::new(10.0, 0.0));
        beam.update(0.9);
        assert!(!beam.is_expired());
        beam.update(0.1);
        assert!(beam.is_expired());
    }
}

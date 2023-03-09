use ::rand::{self, Rng};
use itertools::Itertools;
use macroquad::prelude::*;
use std::collections::VecDeque;

const VIRTUAL_WIDTH: f32 = 1920.0;
const VIRTUAL_HEIGHT: f32 = 1080.0;

const TRAIL_LENGTH: usize = 15000;

const SCALE_FACTOR: f32 = 10e6;
const G: f32 = 6.674e-11 * SCALE_FACTOR;

#[derive(Debug, Default, Clone)]
struct Planet {
  pos: Vec2,
  mass: f32,
  velocity: Vec2,
  color: Color,

  trail: VecDeque<Vec2>,
}

impl Planet {
  fn render(&self) {
    let scale = screen_width() / VIRTUAL_WIDTH;

    let radius = self.mass.ln() * scale;

    draw_circle(
      pos_x(self.pos.x, scale),
      pos_y(self.pos.y, scale),
      radius,
      self.color,
    );

    let segments = Vec::from_iter(self.trail.iter().tuple_windows());
    let len = segments.len();
    for (i, (a, b)) in segments.iter().enumerate() {
      let mut c = self.color;
      c.a = (len - i) as f32 / len as f32;
      draw_line(
        pos_x(a.x, scale),
        pos_y(a.y, scale),
        pos_x(b.x, scale),
        pos_y(b.y, scale),
        1.0,
        c,
      );
    }
  }

  fn gravitate(&mut self, other: &Planet) {
    let d = self.pos.distance_squared(other.pos);

    // both divided by self.mass
    let f = G * other.mass / d;
    let a = f;

    let dir = (other.pos - self.pos).normalize();

    self.velocity += dir * a;
  }

  fn apply_velocity(&mut self) {
    const SCALE_FACTOR: f32 = 1.0;

    self.trail.push_front(self.pos);
    self.trail.truncate(TRAIL_LENGTH);

    self.pos += self.velocity * SCALE_FACTOR;
  }
}

fn pos_x(x: f32, scale: f32) -> f32 {
  screen_width() / 2.0 + x * scale
}

fn pos_y(y: f32, scale: f32) -> f32 {
  screen_height() / 2.0 + y * scale
}

struct Star {
  pos: Vec2,
  magnitude: f32,
}

impl Star {
  fn new() -> Self {
    let scale = screen_width() / VIRTUAL_WIDTH;
    let mut rng = rand::thread_rng();

    let w = VIRTUAL_WIDTH / 2.0;
    let h = screen_height() / 2.0 / scale;
    Star {
      pos: Vec2 {
        x: rng.gen_range(-w..w),
        y: rng.gen_range(-h..h),
      },
      magnitude: rng.gen_range(0.1..=1.1),
    }
  }

  fn render(&self) {
    let scale = screen_width() / VIRTUAL_WIDTH;
    let mut rng = rand::thread_rng();

    if rng.gen_range(0.0..1.0) >= 0.05 {
      draw_circle(
        pos_x(self.pos.x, scale),
        pos_y(self.pos.y, scale),
        self.magnitude * scale,
        WHITE,
      );
    }
  }
}

fn velocity_for_circular_orbit(sat: &Planet, center: &Planet) -> Vec2 {
  let dist = sat.pos.distance(center.pos);
  let speed = (G * (center.mass + sat.mass) / dist).sqrt();
  let diff = sat.pos - center.pos;
  let tan = Vec2 {
    x: -diff.y,
    y: diff.x,
  }
  .normalize();
  tan * speed + center.velocity
}

#[macroquad::main("Planets")]
async fn main() {
  request_new_screen_size(VIRTUAL_WIDTH, VIRTUAL_HEIGHT);

  let sun = Planet {
    mass: 200000.,
    color: Color::from_rgba(249, 182, 17, 255),
    ..Default::default()
  };

  let earth = Planet {
    pos: Vec2 { x: 500.0, y: 0.0 },
    velocity: Vec2 { x: 0.1, y: 0.3 },
    mass: 999.,
    color: Color::from_rgba(129, 171, 84, 255),
    ..Default::default()
  };

  let mut mun = Planet {
    pos: Vec2 {
      x: 450.0,
      y: -450.0,
    },
    mass: 35.,
    color: Color::from_rgba(75, 109, 119, 255),
    ..Default::default()
  };
  mun.velocity = velocity_for_circular_orbit(&mun, &sun);

  let mut mercury = Planet {
    pos: Vec2 { x: 0.0, y: 300.0 },
    mass: 200.,
    color: Color::from_rgba(201, 55, 55, 255),
    ..Default::default()
  };
  mercury.velocity = velocity_for_circular_orbit(&mercury, &sun);

  let mut objects = vec![sun, earth, mun, mercury];

  let stars = (0..500).map(|_| Star::new()).collect::<Vec<Star>>();

  loop {
    clear_background(BLACK);

    let copy = objects.clone();
    for (i, obj) in objects.iter_mut().enumerate() {
      for (j, obj2) in copy.iter().enumerate() {
        if i != j && !is_key_down(KeyCode::Space) {
          obj.gravitate(obj2);
        }
      }
    }

    if !is_key_down(KeyCode::Space) {
      for obj in objects.iter_mut() {
        obj.apply_velocity();
      }
    }

    for obj in objects.iter() {
      obj.render();
    }

    for s in stars.iter() {
      s.render();
    }

    next_frame().await
  }
}

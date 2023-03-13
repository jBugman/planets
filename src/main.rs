use ::rand::{self, Rng};
use itertools::Itertools;
use macroquad::prelude::*;
use std::collections::VecDeque;

const VIRTUAL_WIDTH: f32 = 1920.;
const VIRTUAL_HEIGHT: f32 = 1080.;

const TRAIL_LENGTH: usize = 1000;

const SCALE_FACTOR: f32 = 10e6;
const G: f32 = 6.674e-11 * SCALE_FACTOR;

const MAX_SPEED: f32 = 2.;

const MAX_ORBIT_RADIUS: f32 = 400.;
const ORBIT_ELLIPTICITY: f32 = 0.8;

const CULL_DISTANCE: f32 = 1500.;

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

    let segments = Vec::from_iter(self.trail.iter().step_by(3).tuple_windows());
    let len = segments.len();
    for (i, (a, b)) in segments.iter().enumerate() {
      let mut c = self.color;
      c.a = (len - i) as f32 / len as f32;
      draw_line(
        pos_x(a.x, scale),
        pos_y(a.y, scale),
        pos_x(b.x, scale),
        pos_y(b.y, scale),
        3.0 * c.a,
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
    let mut rng = rand::thread_rng();

    Star {
      pos: Vec2 {
        x: rng.gen_range(-1.0..1.0),
        y: rng.gen_range(-1.0..1.0),
      },
      magnitude: rng.gen_range(0.1..=1.1),
    }
  }

  fn render(&self) {
    let scale = screen_width() / VIRTUAL_WIDTH;
    let mut rng = rand::thread_rng();

    if rng.gen_range(0.0..1.0) >= 0.05 {
      draw_circle(
        pos_x(self.pos.x * screen_width() / 2., 1.),
        pos_y(self.pos.y * screen_height() / 2., 1.),
        self.magnitude * scale,
        WHITE,
      );
    }
  }
}

fn orbit_velocity(sat: &Planet, center: &Planet) -> Vec2 {
  let dist = sat.pos.distance(center.pos);
  let speed = (G * (center.mass + sat.mass) / dist).sqrt();
  let diff = sat.pos - center.pos;
  let tan = Vec2 {
    x: -diff.y,
    y: diff.x,
  }
  .normalize();
  let speed = speed.min(MAX_SPEED);
  tan * speed + center.velocity
}

#[macroquad::main("Planets")]
async fn main() {
  request_new_screen_size(VIRTUAL_WIDTH, VIRTUAL_HEIGHT);

  let mut objects = random_setup();

  let stars = (0..500).map(|_| Star::new()).collect::<Vec<Star>>();

  loop {
    clear_background(BLACK);

    if is_key_pressed(KeyCode::R) {
      objects = random_setup();
    }

    objects.retain_mut(|p| p.pos.length() <= CULL_DISTANCE);

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

fn random_setup() -> Vec<Planet> {
  let mut rng = rand::thread_rng();
  let amount = rng.gen_range(4..=12);

  let sun = Planet {
    mass: 1500000.,
    color: Color::from_rgba(249, 182, 17, 255),
    ..Default::default()
  };

  let mut planets = Vec::from_iter((0..amount).map(|_| {
    let mut planet = Planet {
      pos: Vec2 {
        x: rng.gen_range(-MAX_ORBIT_RADIUS..MAX_ORBIT_RADIUS),
        y: rng.gen_range(-MAX_ORBIT_RADIUS..MAX_ORBIT_RADIUS),
      },
      mass: rng.gen_range(50.0..=5000.0),
      color: Color::from_rgba(
        rng.gen_range(20..=255),
        rng.gen_range(20..=255),
        rng.gen_range(20..=255),
        255,
      ),
      ..Default::default()
    };
    planet.velocity = orbit_velocity(&planet, &sun);
    planet.velocity.x += rng.gen_range(-ORBIT_ELLIPTICITY..=ORBIT_ELLIPTICITY);
    planet
  }));
  planets.push(sun);
  planets
}

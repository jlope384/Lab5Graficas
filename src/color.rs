use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8,
}

impl Color {
  // Constructor to initialize the color using r, g, b values as u8
  pub fn new(r: u8, g: u8, b: u8) -> Self {
    Color { r, g, b }
  }

  // default color
  pub fn black() -> Self {
    Color { r: 0, g: 0, b: 0 }
  }


  // Function to return the color as a hex value
  pub fn to_hex(&self) -> u32 {
    ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
  }
}

// Implement addition for Color
use std::ops::Add;

impl Add for Color {
  type Output = Color;

  fn add(self, other: Color) -> Color {
    Color {
      r: self.r.saturating_add(other.r),
      g: self.g.saturating_add(other.g),
      b: self.b.saturating_add(other.b),
    }
  }
}

// Implement multiplication by a constant for Color
use std::ops::Mul;

impl Mul<f32> for Color {
  type Output = Color;

  fn mul(self, scalar: f32) -> Color {
    Color {
      r: (self.r as f32 * scalar).clamp(0.0, 255.0) as u8,
      g: (self.g as f32 * scalar).clamp(0.0, 255.0) as u8,
      b: (self.b as f32 * scalar).clamp(0.0, 255.0) as u8,
    }
  }
}

// Implement display formatting for Color
impl fmt::Display for Color {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Color(r: {}, g: {}, b: {})", self.r, self.g, self.b)
  }
}

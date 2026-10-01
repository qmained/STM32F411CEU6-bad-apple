use embedded_graphics::geometry::Point;
use embedded_graphics::{Drawable, Pixel};
use embedded_graphics::pixelcolor::BinaryColor;
use crate::tasks::display_task::SsdDisplay;

pub struct LargePoint {
    pub(crate) x: i32,
    pub(crate) y: i32,
    w: i32,
    h: i32,
}

impl LargePoint {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        LargePoint { x, y, w, h }
    }

    pub fn draw(&mut self, display: &mut SsdDisplay, color: BinaryColor) {
        for i in 0..self.w {
            for j in 0..self.h {
                let curr_point = Point::new(self.x + i, self.y + j);
                Pixel(curr_point, color).draw(display).unwrap();
            }
        }
    }

    pub fn next_point(&mut self) {
        let x_offset = 128 - self.w;
        let y_offset = 64 - self.h;

        match self.y {
            0 => {
                if self.x < x_offset {
                    self.x += 1;
                } else {
                    self.y += 1;
                }
            }
            y if (1..y_offset).contains(&y) => {
                if self.x == 0 {
                    self.y -= 1;
                } else if self.x == x_offset {
                    self.y += 1;
                }
            }
            y if y == y_offset => {
                if self.x > 0 {
                    self.x -= 1;
                } else {
                    self.y -= 1;
                }
            }
            _ => {}
        }
    }
}
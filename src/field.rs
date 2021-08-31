use crate::BefungeCommand;

pub type BefungeCell = i32;

#[derive(Clone, Debug)]
pub struct FungeField {
    width: usize,
    height: usize,
    pub cells: Vec<BefungeCell>,
}

impl FungeField {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![BefungeCommand::NO_OP as i32; width * height],
        }
    }

    fn load_str(&mut self, input: &str) {
        let mut y = 0;
        for line in input.lines() {
            if y >= self.height {
                break;
            }

            let mut x = 0;
            let y_offset = y * self.width;

            for c in line.chars() {
                if x >= self.width || c.len_utf8() > 1 {
                    break;
                }
                self.cells[x + y_offset] = c as i32;
                x += 1;
            }
            y += 1;
        }
    }

    pub fn from_str(input: &str, width: usize, height: usize) -> Self {
        let mut field = Self::new(width, height);
        field.load_str(input);

        field
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Option<BefungeCell> {
        if x < self.width && y < self.height {
            Some(self.cells[x + y * self.width])
        } else {
            None
        }
    }

    pub fn set(&mut self, x: usize, y: usize, value: i32) {
        if x < self.width && y < self.height {
            self.cells[x + y * self.width] = value;
        }
    }
}

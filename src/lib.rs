#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum BefungeCommand {
    NoOp = b' ',
    Negate = b'!',
    ToggleStringMode = b'"',
    Bridge = b'#',
    Discard = b'$',
    Modulo = b'%',
    ReadInt = b'&',
    Multiply = b'*',
    Add = b'+',
    WriteChar = b',',
    Subtract = b'-',
    WriteInt = b'.',
    Divide = b'/',
    Duplicate = b':',
    Left = b'<',
    Right = b'>',
    Random = b'?',
    Stop = b'@',
    Swap = b'\\',
    Up = b'^',
    IfLeftRight = b'_',
    Compare = b'`',
    ReadCell = b'g',
    WriteCell = b'p',
    Down = b'v',
    IfUpDown = b'|',
    ReadChar = b'~',
}

enum Delta {
    Right,
    Left,
    Down,
    Up,
}

pub type BefungeCell = u8;

struct BefungePC {
    x: u8,
    y: u8,
    delta: Delta,
}

impl BefungePC {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            delta: Delta::Right,
        }
    }
}

#[derive(Debug)]
struct BefungeField {
    width: usize,
    height: usize,
    cells: Vec<BefungeCell>,
}

impl BefungeField {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![b' '; width * height],
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
                self.cells[x + y_offset] = c as u8;
                x += 1;
            }
            y += 1;
        }
    }

    pub fn from_str(input: &str, width: usize, height: usize) -> Self {
        let mut field = BefungeField::new(width, height);
        field.load_str(input);

        field
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn value_at(&self, x: usize, y: usize) -> BefungeCell {
        self.cells[x + y * self.width]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_field() {
        let field = BefungeField::new(80, 25);
        assert_eq!(field.value_at(0, 0), b' ');
    }
}

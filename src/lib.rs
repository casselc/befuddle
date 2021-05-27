use std::convert::TryInto;

#[derive(Clone, Copy, Debug)]
struct BefungeCommand;

impl BefungeCommand {
    const NO_OP: u8 = b' ';
    const NEGATE: u8 = b'!';
    const TOGGLE_STRING_MODE: u8 = b'"';
    const BRIDGE: u8 = b'#';
    const DISCARD: u8 = b'$';
    const MODULO: u8 = b'%';
    const READ_INT: u8 = b'&';
    const MULTIPLY: u8 = b'*';
    const ADD: u8 = b'+';
    const WRITE_CHAR: u8 = b',';
    const SUBTRACT: u8 = b'-';
    const WRITE_INT: u8 = b'.';
    const DIVIDE: u8 = b'/';
    const DUPLICATE: u8 = b':';
    const LEFT: u8 = b'<';
    const RIGHT: u8 = b'>';
    const RANDOM: u8 = b'?';
    const STOP: u8 = b'@';
    const SWAP: u8 = b'\\';
    const UP: u8 = b'^';
    const IF_LEFT_RIGHT: u8 = b'_';
    const COMPARE: u8 = b'`';
    const READ_CELL: u8 = b'g';
    const WRITE_CELL: u8 = b'p';
    const DOWN: u8 = b'v';
    const IF_UP_DOWN: u8 = b'|';
    const READ_CHAR: u8 = b'~';
}

type BefungeCell = u8;

#[derive(Clone, Debug)]
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
            cells: vec![BefungeCommand::NO_OP as u8; width * height],
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

    pub fn get(&self, x: usize, y: usize) -> Option<BefungeCell> {
        if x < self.width && y < self.height {
            Some(self.cells[x + y * self.width])
        } else {
            None
        }
    }

    pub fn set(&mut self, x: usize, y: usize, value: u8) {
        if x < self.width && y < self.height {
            self.cells[x + y * self.width] = value;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Delta {
    Right,
    Left,
    Down,
    Up,
}

#[derive(Clone, Debug)]
struct BefungeExecution {
    pc_x: usize,
    pc_y: usize,
    pc_delta: Delta,
    string_mode: bool,
    field: BefungeField,
    stack: Vec<i32>,
}

impl BefungeExecution {
    pub fn new(field: BefungeField) -> Self {
        Self {
            pc_x: 0,
            pc_y: 0,
            pc_delta: Delta::Right,
            string_mode: false,
            field,
            stack: Vec::new(),
        }
    }

    pub fn pc(&self) -> (usize, usize, Delta) {
        (self.pc_x, self.pc_y, self.pc_delta)
    }

    pub fn stack(&self) -> Vec<i32> {
        self.stack.clone()
    }

    pub fn get(&self, x: usize, y: usize) -> Option<BefungeCell> {
        self.field.get(x, y)
    }

    pub fn move_pc(&mut self) {
        match self.pc_delta {
            Delta::Right => {
                self.pc_x = if self.pc_x < self.field.width() - 1 {
                    self.pc_x + 1
                } else {
                    0
                }
            }
            Delta::Left => {
                self.pc_x = if self.pc_x > 0 {
                    self.pc_x - 1
                } else {
                    self.field.width() - 1
                }
            }
            Delta::Down => {
                self.pc_y = if self.pc_y < self.field.height() - 1 {
                    self.pc_y + 1
                } else {
                    0
                }
            }
            Delta::Up => {
                self.pc_y = if self.pc_y > 0 {
                    self.pc_y - 1
                } else {
                    self.field.height() - 1
                }
            }
        }
    }

    pub fn step(&mut self) {
        if let Some(curr) = self.field.get(self.pc_x, self.pc_y) {
            if self.string_mode {
                if curr == BefungeCommand::TOGGLE_STRING_MODE {
                    self.string_mode = false;
                } else {
                    self.stack.push(curr as i32);
                }
            } else {
                match curr {
                    BefungeCommand::NO_OP => {}
                    BefungeCommand::NEGATE => {
                        let top = self.stack.pop().unwrap_or_default();

                        self.stack.push(if top > 0 { 0 } else { 1 })
                    }
                    BefungeCommand::TOGGLE_STRING_MODE => self.string_mode = true,
                    BefungeCommand::BRIDGE => {
                        self.move_pc();
                    }
                    BefungeCommand::DISCARD => {
                        let _top = self.stack.pop();
                    }
                    BefungeCommand::MODULO => {
                        let top = self.stack.pop().unwrap_or_default();
                        let second = self.stack.pop().unwrap_or_default();

                        self.stack.push(top % second);
                    }
                    BefungeCommand::READ_INT => {}
                    BefungeCommand::MULTIPLY => {
                        let top = self.stack.pop().unwrap_or_default();
                        let second = self.stack.pop().unwrap_or_default();

                        self.stack.push(top * second);
                    }
                    BefungeCommand::ADD => {
                        let top = self.stack.pop().unwrap_or_default();
                        let second = self.stack.pop().unwrap_or_default();

                        self.stack.push(top + second);
                    }
                    BefungeCommand::WRITE_CHAR => {}
                    BefungeCommand::SUBTRACT => {
                        let top = self.stack.pop().unwrap_or_default();
                        let second = self.stack.pop().unwrap_or_default();

                        self.stack.push(top - second);
                    }
                    BefungeCommand::WRITE_INT => {}
                    BefungeCommand::DIVIDE => {
                        let top = self.stack.pop().unwrap_or_default();
                        let second = self.stack.pop().unwrap_or_default();

                        self.stack.push(top / second);
                    }
                    BefungeCommand::DUPLICATE => {
                        let top = self.stack.pop().unwrap_or_default();

                        self.stack.push(top);
                        self.stack.push(top);
                    }
                    BefungeCommand::LEFT => {
                        self.pc_delta = Delta::Left;
                    }
                    BefungeCommand::RIGHT => {
                        self.pc_delta = Delta::Right;
                    }
                    BefungeCommand::RANDOM => {}
                    BefungeCommand::STOP => {}
                    BefungeCommand::SWAP => {
                        let top = self.stack.pop().unwrap_or_default();
                        let second = self.stack.pop().unwrap_or_default();

                        self.stack.push(top);
                        self.stack.push(second);
                    }
                    BefungeCommand::UP => {
                        self.pc_delta = Delta::Up;
                    }
                    BefungeCommand::IF_LEFT_RIGHT => {
                        let top = self.stack.pop().unwrap_or_default();

                        self.pc_delta = if top > 0 { Delta::Left } else { Delta::Right };
                    }
                    BefungeCommand::COMPARE => {
                        let top = self.stack.pop().unwrap_or_default();
                        let second = self.stack.pop().unwrap_or_default();

                        self.stack.push(if top > second { 1 } else { 0 })
                    }
                    BefungeCommand::READ_CELL => {
                        let top: usize = self.stack.pop().unwrap_or_default().try_into().unwrap();
                        let second = self.stack.pop().unwrap_or_default().try_into().unwrap();

                        if let Some(val) = self.field.get(second, top) {
                            self.stack.push(val as i32)
                        }
                    }
                    BefungeCommand::WRITE_CELL => {
                        let top = self.stack.pop().unwrap_or_default().try_into().unwrap();
                        let second = self.stack.pop().unwrap_or_default().try_into().unwrap();
                        let value = self.stack.pop().unwrap_or_default().try_into().unwrap();

                        self.field.set(second, top, value);
                    }
                    BefungeCommand::DOWN => {
                        self.pc_delta = Delta::Down;
                    }
                    BefungeCommand::IF_UP_DOWN => {
                        let top = self.stack.pop().unwrap_or_default();

                        self.pc_delta = if top > 0 { Delta::Up } else { Delta::Down };
                    }
                    BefungeCommand::READ_CHAR => {}
                    b'0'..=b'9' => self.stack.push((curr - 48) as i32),
                    _ => self.stack.push(curr as i32),
                }
            }
            self.move_pc();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty_field() {
        let field = BefungeField::new(80, 25);
        assert_eq!(field.get(0, 0), Some(BefungeCommand::NO_OP));
        assert_eq!(field.get(79, 24), Some(BefungeCommand::NO_OP));
        assert_eq!(field.get(80, 0), None);
    }

    #[test]
    fn test_string_field() {
        let field = BefungeField::from_str("0\n1\n", 80, 25);
        assert_eq!(field.get(0, 0), Some(b'0'));
        assert_eq!(field.get(1, 0), Some(b' '));
        assert_eq!(field.get(0, 1), Some(b'1'));
        assert_eq!(field.get(0, 2), Some(b' '));
        assert_eq!(field.get(79, 24), Some(b' '));
    }

    #[test]
    fn test_truncate() {
        let field = BefungeField::from_str("012\n01\n01", 2, 2);
        assert_eq!(field.get(0, 0), Some(b'0'));
        assert_eq!(field.get(1, 0), Some(b'1'));
        assert_eq!(field.get(2, 0), None);
        assert_eq!(field.get(0, 1), Some(b'0'));
        assert_eq!(field.get(1, 1), Some(b'1'));
        assert_eq!(field.get(0, 2), None);
    }

    #[test]
    fn test_horizontal_wrap_right() {
        let mut exec = BefungeExecution::new(BefungeField::new(2, 1));
        exec.step();
        let (x, _y, _delta) = exec.pc();
        assert_eq!(x, 1);
        exec.step();
        let (x, _y, _delta) = exec.pc();
        assert_eq!(x, 0);
        exec.step();
        let (x, _y, _delta) = exec.pc();
        assert_eq!(x, 1);
    }

    #[test]
    fn test_horizontal_wrap_left() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("<", 3, 1));
        exec.step();
        let (x, _y, _delta) = exec.pc();
        assert_eq!(x, 2);
        exec.step();
        let (x, _y, _delta) = exec.pc();
        assert_eq!(x, 1);
        exec.step();
        let (x, _y, _delta) = exec.pc();
        assert_eq!(x, 0);
    }

    #[test]
    fn test_vertical_wrap_down() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("v", 1, 2));
        exec.step();
        let (_x, y, _delta) = exec.pc();
        assert_eq!(y, 1);
        exec.step();
        let (_x, y, _delta) = exec.pc();
        assert_eq!(y, 0);
        exec.step();
        let (_x, y, _delta) = exec.pc();
        assert_eq!(y, 1);
    }

    #[test]
    fn test_vertical_wrap_up() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("^", 1, 2));
        exec.step();
        let (_x, y, _delta) = exec.pc();
        assert_eq!(y, 1);
        exec.step();
        let (_x, y, _delta) = exec.pc();
        assert_eq!(y, 0);
        exec.step();
        let (_x, y, _delta) = exec.pc();
        assert_eq!(y, 1);
    }

    #[test]
    fn test_push_digits() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("0123456789", 10, 1));

        for _i in 0..10 {
            exec.step()
        }

        assert_eq!(exec.stack(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
    }

    #[test]
    fn test_string_mode() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("\"0123456789\"0", 13, 1));

        for _i in 0..13 {
            exec.step()
        }

        assert_eq!(
            exec.stack(),
            vec![48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 0]
        )
    }

    #[test]
    fn test_read_cell() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("g", 1, 1));

        exec.step();

        assert_eq!(exec.stack(), vec![103])
    }

    #[test]
    fn test_write_cell() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("p", 1, 1));

        exec.step();

        assert_eq!(exec.get(0, 0), Some(0));
    }

    #[test]
    fn test_negate() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("!!", 2, 1));

        exec.step();
        assert_eq!(exec.stack(), vec![1]);
        exec.step();
        assert_eq!(exec.stack(), vec![0]);
    }

    #[test]
    fn test_swap() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("01\\", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![1, 0]);
    }

    #[test]
    fn test_add() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("12+", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![3]);
    }

    #[test]
    fn test_subtract() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("12-", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![1]);
    }

    #[test]
    fn test_multiply() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("12*", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![2]);
    }

    #[test]
    fn test_divide() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("12/", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![2]);
    }

    #[test]
    fn test_modulo() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("23%", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![1]);
    }

    #[test]
    fn test_compare() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("12`", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![1]);

        let mut exec = BefungeExecution::new(BefungeField::from_str("21`", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![0]);
    }

    #[test]
    fn test_duplicate() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("1:", 2, 1));

        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![1, 1]);
    }

    #[test]
    fn test_discard() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("1$", 2, 1));

        exec.step();
        assert_eq!(exec.stack(), vec![1]);
        exec.step();
        assert_eq!(exec.stack(), vec![]);
    }

    #[test]
    fn test_if_left_right() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("1_", 2, 1));

        exec.step();
        assert_eq!(exec.stack(), vec![1]);
        exec.step();
        assert_eq!(exec.stack(), vec![]);
        let (x, _y, delta) = exec.pc();
        assert_eq!(x, 0);
        assert_eq!(delta, Delta::Left);

        let mut exec = BefungeExecution::new(BefungeField::from_str("0_", 2, 1));
        exec.step();
        assert_eq!(exec.stack(), vec![0]);
        exec.step();
        assert_eq!(exec.stack(), vec![]);
        let (x, _y, delta) = exec.pc();
        assert_eq!(x, 0);
        assert_eq!(delta, Delta::Right);
    }

    #[test]
    fn test_if_up_down() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("1|", 2, 2));

        exec.step();
        assert_eq!(exec.stack(), vec![1]);
        exec.step();
        assert_eq!(exec.stack(), vec![]);
        let (_x, y, delta) = exec.pc();
        assert_eq!(y, 1);
        assert_eq!(delta, Delta::Up);

        let mut exec = BefungeExecution::new(BefungeField::from_str("0|", 2, 2));
        exec.step();
        assert_eq!(exec.stack(), vec![0]);
        exec.step();
        assert_eq!(exec.stack(), vec![]);
        let (_x, y, delta) = exec.pc();
        assert_eq!(y, 1);
        assert_eq!(delta, Delta::Down);
    }
}

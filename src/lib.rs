use std::collections::{BTreeMap, HashMap};
use std::convert::{From, TryInto};
use std::io;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FungeCell(i32);

impl Default for FungeCell {
    fn default() -> Self {
        Self(b' '.into())
    }
}

impl From<i32> for FungeCell {
    fn from(value: i32) -> Self {
        FungeCell(value)
    }
}

impl std::ops::Add<FungeCell> for FungeCell {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Add<i32> for FungeCell {
    type Output = Self;
    fn add(self, other: i32) -> Self {
        Self(self.0 + other)
    }
}

impl std::ops::AddAssign<FungeCell> for FungeCell {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

impl std::ops::Sub<FungeCell> for FungeCell {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl std::ops::SubAssign<FungeCell> for FungeCell {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl std::ops::Mul<FungeCell> for FungeCell {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}

impl std::ops::MulAssign<FungeCell> for FungeCell {
    fn mul_assign(&mut self, other: Self) {
        self.0 *= other.0
    }
}

impl std::ops::Div<FungeCell> for FungeCell {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Self(self.0 / other.0)
    }
}

impl std::ops::DivAssign<FungeCell> for FungeCell {
    fn div_assign(&mut self, other: Self) {
        self.0 /= other.0
    }
}

impl std::ops::Rem<FungeCell> for FungeCell {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        Self(self.0 % other.0)
    }
}

impl std::ops::RemAssign<FungeCell> for FungeCell {
    fn rem_assign(&mut self, other: Self) {
        self.0 %= other.0
    }
}

#[derive(Debug, Clone)]
struct FungeStack {
    cells: Vec<FungeCell>,
}

impl FungeStack {
    pub fn pop(&mut self) -> FungeCell {
        self.cells.pop().unwrap_or_default()
    }

    pub fn push(&mut self, value: FungeCell) {
        self.cells.push(value)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct FungeCoordinate(i32, i32);

impl FungeCoordinate {
    const ORIGIN: FungeCoordinate = FungeCoordinate(0, 0);
    const TOP_LEFT: FungeCoordinate = FungeCoordinate(FungeSpace::MIN_X, FungeSpace::MIN_Y);
    const TOP_RIGHT: FungeCoordinate = FungeCoordinate(FungeSpace::MAX_X, FungeSpace::MIN_Y);
    const BOTTOM_LEFT: FungeCoordinate = FungeCoordinate(FungeSpace::MIN_X, FungeSpace::MAX_Y);
    const BOTTOM_RIGHT: FungeCoordinate = FungeCoordinate(FungeSpace::MAX_X, FungeSpace::MAX_Y);
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct FungeDelta(i32, i32);

impl FungeDelta {
    const NORTH: FungeDelta = FungeDelta(0, -1);
    const SOUTH: FungeDelta = FungeDelta(0, 1);
    const EAST: FungeDelta = FungeDelta(1, 0);
    const WEST: FungeDelta = FungeDelta(-1, 0);
    const NEGATE: FungeDelta = FungeDelta(-1, -1);
}

impl std::ops::Add<FungeDelta> for FungeCoordinate {
    type Output = Self;
    fn add(self, other: FungeDelta) -> Self {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

impl std::ops::Mul<FungeDelta> for FungeCoordinate {
    type Output = Self;
    fn mul(self, other: FungeDelta) -> Self {
        Self(self.0 * other.0, self.1 * other.1)
    }
}

#[derive(Debug, Clone)]
struct FungeSpace {
    x_max: i32,
    x_min: i32,
    y_max: i32,
    y_min: i32,
    field: BTreeMap<FungeCoordinate, FungeCell>,
}

impl FungeSpace {
    const MAX_X: i32 = i32::MAX;
    const MAX_Y: i32 = i32::MAX;
    const MIN_X: i32 = i32::MIN;
    const MIN_Y: i32 = i32::MIN;

    pub fn new() -> Self {
        Self {
            x_max: 0.into(),
            x_min: 0.into(),
            y_max: 0.into(),
            y_min: 0.into(),
            field: BTreeMap::new(),
        }
    }

    pub fn upper_left(&self) -> FungeCoordinate {
        FungeCoordinate(self.x_min, self.y_min)
    }

    pub fn lower_right(&self) -> FungeCoordinate {
        FungeCoordinate(self.x_max, self.y_max)
    }
}

struct FungePointer {
    position: FungeCoordinate,
    delta: FungeDelta,
}

impl FungePointer {}

#[derive(Debug, StructOpt)]
struct ExecOptions {
    input_file: PathBuf,
    output_file: PathBuf,
    program: PathBuf,
}

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
pub struct BefungeField {
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
        for (y, line) in input.lines().enumerate() {
            if y >= self.height {
                break;
            }

            let y_offset = y * self.width;

            for (x, c) in line.chars().enumerate() {
                if x >= self.width || c.len_utf8() > 1 {
                    break;
                }
                self.cells[x + y_offset] = c as u8;
            }
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
pub struct BefungeExecution {
    pc_x: usize,
    pc_y: usize,
    pc_delta: Delta,
    string_mode: bool,
    field: BefungeField,
    stack: Vec<i32>,
    active: bool,
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
            active: true,
        }
    }

    fn pc(&self) -> (usize, usize, Delta) {
        (self.pc_x, self.pc_y, self.pc_delta)
    }

    pub fn stack(&self) -> Vec<i32> {
        self.stack.clone()
    }

    fn get(&self, x: usize, y: usize) -> Option<BefungeCell> {
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

    pub fn run(&mut self) {
        while self.active {
            self.step();
        }
    }

    pub fn step(&mut self) {
        if self.active {
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
                        BefungeCommand::READ_INT => {
                            let mut input = String::new();
                            io::stdin()
                                .read_line(&mut input)
                                .expect("Error reading integer");

                            let i = input.parse::<i32>().unwrap();
                            self.stack.push(i);
                        }
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
                        BefungeCommand::WRITE_CHAR => {
                            let top = self.stack.pop().unwrap_or_default();
                            let u: u32 = top.try_into().unwrap();
                            let c: char = u.try_into().unwrap();

                            print!("{}", c);
                        }
                        BefungeCommand::SUBTRACT => {
                            let top = self.stack.pop().unwrap_or_default();
                            let second = self.stack.pop().unwrap_or_default();

                            self.stack.push(top - second);
                        }
                        BefungeCommand::WRITE_INT => {
                            let top = self.stack.pop().unwrap_or_default();

                            print!("{}", top);
                        }
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
                        BefungeCommand::STOP => {
                            self.active = false;
                        }
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
                            let top: usize =
                                self.stack.pop().unwrap_or_default().try_into().unwrap();
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
                        BefungeCommand::READ_CHAR => {
                            let mut input = String::new();
                            print!("Enter a character: ");
                            io::stdin()
                                .read_line(&mut input)
                                .expect("Error reading character");

                            let c = input.as_bytes()[0];
                            self.stack.push(c as i32);
                        }
                        b'0'..=b'9' => self.stack.push((curr - 48) as i32),
                        _ => self.stack.push(curr as i32),
                    }
                }
                if self.active {
                    self.move_pc();
                }
            }
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

    #[test]
    fn test_write_int() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("12..", 4, 1));
        exec.step();
        exec.step();
        exec.step();
        exec.step();
    }

    #[test]
    fn test_write_char() {
        let mut exec = BefungeExecution::new(BefungeField::from_str("\"a\",", 4, 1));
        exec.step();
        exec.step();
        exec.step();
        exec.step();
    }
}

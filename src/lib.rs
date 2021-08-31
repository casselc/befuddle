pub mod field;
pub mod ops;
pub mod pointer;
pub mod stack;

use crate::field::{BefungeCell, FungeField};
use crossterm::cursor::*;
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::*;
use std::convert::{From, TryFrom, TryInto};
use std::io::{stdout, Write};
use std::iter::FromIterator;
use std::path::PathBuf;
use std::thread;
use structopt::StructOpt;

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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Delta {
    Right,
    Left,
    Down,
    Up,
}

pub struct BefungeExecution {
    pc_x: usize,
    pc_y: usize,
    pc_delta: Delta,
    string_mode: bool,
    field: FungeField,
    stack: Vec<i32>,
    active: bool,
}

pub trait FungeOutput {
    fn write_character(&mut self, c: i32);
    fn write_number(&mut self, num: i32);
}

pub trait FungeInput {
    fn read_character(&mut self) -> i32;
    fn read_number(&mut self) -> i32;
}

pub trait FungeRenderer: FungeInput + FungeOutput {
    fn render_field(&mut self, cells: &Vec<i32>);

    fn render_stack(&mut self, values: &Vec<i32>);

    fn render_pointer(&mut self, pointer: (usize, usize));
}

pub struct TerminalRenderer {
    field_width: u16,
    field_height: u16,
    term_width: u16,
    term_height: u16,
    prev_width: u16,
    prev_height: u16,
    output_position: (u16, u16),
}

impl TerminalRenderer {
    const BOTTOM_LEFT_CORNER: char = '╚';
    const TOP_LEFT_CORNER: char = '╔';
    const TEE_BOTTOM: char = '╩';
    const TEE_TOP: char = '╦';
    const TEE_LEFT: char = '╠';
    const HORIZONTAL_BORDER: char = '═';

    const TOP_RIGHT_CORNER: char = '╗';
    const BOTTOM_RIGHT_CORNER: char = '╝';
    const VERTICAL_BORDER: char = '║';
    const TEE_RIGHT: char = '╣';

    pub fn new(field_width: u16, field_height: u16) -> Self {
        let (prev_width, prev_height) = size().unwrap_or_default();
        let (term_width, term_height) = (field_width + 13, field_height + 8);

        TerminalRenderer {
            field_width,
            field_height,
            term_width,
            term_height,
            prev_width,
            prev_height,

            output_position: (1, field_height + 2),
        }
    }

    pub fn init(&mut self) -> () {
        queue!(
            stdout(),
            DisableLineWrap,
            Hide,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            Clear(ClearType::All),
            SetTitle("befuddle"),
        )
        .unwrap();

        let mut line = vec![TerminalRenderer::HORIZONTAL_BORDER; self.term_width.into()];
        line[0] = TerminalRenderer::TOP_LEFT_CORNER;
        line[(self.field_width + 1) as usize] = TerminalRenderer::TEE_TOP;
        line[(self.term_width - 1) as usize] = TerminalRenderer::TOP_RIGHT_CORNER;

        let mut line_str = String::from_iter(&line);
        queue!(stdout(), MoveTo(0, 0), Print(line_str)).unwrap();

        for y in 1..=(self.field_height + 1) {
            queue!(
                stdout(),
                MoveTo(0, y),
                Print(TerminalRenderer::VERTICAL_BORDER),
                MoveToColumn(self.field_width + 2),
                Print(TerminalRenderer::VERTICAL_BORDER),
                MoveToColumn(self.term_width),
                Print(TerminalRenderer::VERTICAL_BORDER),
            )
            .unwrap();
        }

        line[0] = TerminalRenderer::TEE_LEFT;
        line[(self.field_width + 1) as usize] = TerminalRenderer::TEE_BOTTOM;
        line[(self.term_width - 1) as usize] = TerminalRenderer::TEE_RIGHT;

        line_str = String::from_iter(&line);
        queue!(stdout(), MoveTo(0, self.field_height + 1), Print(line_str),).unwrap();

        for y in (self.field_height + 2)..self.term_height {
            queue!(
                stdout(),
                MoveTo(0, y),
                Print(TerminalRenderer::VERTICAL_BORDER),
                MoveToColumn(self.term_width),
                Print(TerminalRenderer::VERTICAL_BORDER),
            );
        }

        line[0] = TerminalRenderer::BOTTOM_LEFT_CORNER;
        line[(self.field_width + 1) as usize] = TerminalRenderer::HORIZONTAL_BORDER;
        line[(self.term_width - 1) as usize] = TerminalRenderer::BOTTOM_RIGHT_CORNER;

        line_str = String::from_iter(&line);
        queue!(
            stdout(),
            MoveTo(0, self.term_height),
            Print(line_str),
            MoveTo(self.field_width + 2, 11),
            Print(str::repeat(
                &TerminalRenderer::HORIZONTAL_BORDER.to_string(),
                10
            )),
            MoveTo(1, 1),
            Show
        );

        stdout().flush().unwrap();
    }

    pub fn stop(&mut self) {
        std::io::stdin().read_line(&mut String::new()).unwrap();
        queue!(
            stdout(),
            ResetColor,
            SetSize(self.prev_width, self.prev_height),
            Clear(ClearType::All),
        );

        stdout().flush().unwrap();
    }
}

impl FungeInput for PrintlnRenderer {
    fn read_character(&mut self) -> i32 {
        print!("\nEnter a character, followed by return/enter: ");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Error reading character");

        let c = input.as_bytes()[0];
        c as i32
    }

    fn read_number(&mut self) -> i32 {
        print!("\nEnter a number, followed by return/enter: ");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Error reading integer");

        let i = input.parse::<i32>().unwrap();
        i
    }
}

impl FungeOutput for PrintlnRenderer {
    fn write_character(&mut self, c: i32) {
        if let Ok(b) = u8::try_from(c) {
            println!("Output: {}", unsafe {
                std::char::from_u32_unchecked(b.into())
            })
        } else {
            println!("Output: ");
        }
    }

    fn write_number(&mut self, num: i32) {
        println!("Output: {}", num);
    }
}

impl FungeOutput for TerminalRenderer {
    fn write_character(&mut self, c: i32) {
        let output = &mut stdout();
        let (x, y) = self.output_position;
        if let Ok(b) = u8::try_from(c) {
            queue!(
                output,
                SavePosition,
                Hide,
                MoveTo(x, y),
                Print(unsafe { std::char::from_u32_unchecked(b.into()) }),
                RestorePosition,
                Show
            );
        }

        self.output_position = if c != 13 && x < self.field_width {
            (x + 1, y)
        } else {
            (1, y + 1)
        };

        output.flush().unwrap();
    }

    fn write_number(&mut self, num: i32) {
        let output = &mut stdout();
        let (x, y) = self.output_position;
        let display_num = num.to_string();
        let next_x = x + 1 + display_num.len() as u16;
        let excess_chars: i32 = 0; //(next_x - self.field_width).into();
        queue!(output, SavePosition, Hide, MoveTo(x, y));

        if excess_chars > 0 {
            queue!(
                output,
                Print(&display_num[0..(display_num.len() - excess_chars as usize)]),
                MoveTo(1, y + 1),
                Print(&display_num[(display_num.len() - excess_chars as usize)..display_num.len()])
            );
            self.output_position = (excess_chars as u16 + 2, y + 1);
        } else {
            queue!(output, Print(&display_num));
            self.output_position = (x + display_num.len() as u16, y);
        }
        queue!(output, RestorePosition, Show);
        output.flush().unwrap();
    }
}

impl FungeInput for TerminalRenderer {
    fn read_character(&mut self) -> i32 {
        queue!(
            stdout(),
            SavePosition,
            MoveTo(1, self.output_position.1 + 1),
            Print("Type a character and press Enter: ")
        )
        .unwrap();
        stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Error reading character");

        let c = input.as_bytes()[0];
        queue!(
            stdout(),
            Hide,
            MoveTo(0, self.output_position.1 + 1),
            Clear(ClearType::CurrentLine),
            Print(TerminalRenderer::VERTICAL_BORDER),
            MoveTo(self.term_width - 1, self.output_position.1 + 1),
            Print(TerminalRenderer::VERTICAL_BORDER),
            RestorePosition
        );
        stdout().flush().unwrap();
        c as i32
    }

    fn read_number(&mut self) -> i32 {
        queue!(
            stdout(),
            SavePosition,
            MoveTo(1, self.output_position.1 + 1),
            Print("Type a number and press Enter: ")
        )
        .unwrap();
        stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Error reading integer");
        println!("{:#?}", input);
        let i = input.trim_end().parse::<i32>().unwrap();
        queue!(
            stdout(),
            Hide,
            MoveTo(0, self.output_position.1 + 1),
            Clear(ClearType::CurrentLine),
            Print(TerminalRenderer::VERTICAL_BORDER),
            MoveTo(self.term_width, self.output_position.1 + 1),
            Print(TerminalRenderer::VERTICAL_BORDER),
            RestorePosition
        );
        stdout().flush().unwrap();
        i
    }
}

impl FungeRenderer for TerminalRenderer {
    fn render_field(&mut self, cells: &Vec<i32>) {
        queue!(
            stdout(),
            SavePosition,
            Hide,
            SetForegroundColor(Color::DarkGrey),
            MoveTo(1, 1)
        );
        for line in cells.chunks(80) {
            let bytes = line.iter().map(|c| *c as u8).collect::<Vec<u8>>();
            let to_print = std::str::from_utf8(&bytes).unwrap();
            queue!(
                stdout(),
                MoveToColumn(2),
                Print(to_print),
                MoveToNextLine(1)
            );
        }
        queue!(
            stdout(),
            RestorePosition,
            SetForegroundColor(Color::White),
            Show
        );

        stdout().flush().unwrap();
    }

    fn render_pointer(&mut self, pointer: (usize, usize)) {
        queue!(
            stdout(),
            Hide,
            MoveTo(5, self.field_height + 1),
            Print(format!(" [ {:2}, {:2} ] ", pointer.0, pointer.1)),
            MoveTo(pointer.0 as u16 + 1, pointer.1 as u16 + 1),
            Show
        );

        stdout().flush().unwrap();
    }
    fn render_stack(&mut self, values: &Vec<i32>) {
        queue!(stdout(), SavePosition, Hide);

        let val_count = values.len().min(10);

        for (i, v) in values.iter().take(val_count).enumerate() {
            queue!(
                stdout(),
                MoveTo(self.field_width + 2, (10 - i) as u16),
                Print(format!("{:10}", v))
            );
        }
        for i in 0..(10 - val_count) {
            queue!(
                stdout(),
                MoveTo(self.field_width + 2, 1 + i as u16),
                Print("          ")
            );
        }
        queue!(stdout(), RestorePosition, Show);
        stdout().flush().unwrap();
    }
}
pub struct PrintlnRenderer {}

impl PrintlnRenderer {}

impl FungeRenderer for PrintlnRenderer {
    fn render_field(&mut self, cells: &Vec<i32>) {
        for line in cells.chunks(80) {
            let bytes = line.iter().map(|c| *c as u8).collect::<Vec<u8>>();
            let to_print = unsafe { std::str::from_utf8_unchecked(&bytes) };
            println!("{}", to_print);
        }

        //println!("Field: {:#?}", cells)
    }

    fn render_stack(&mut self, values: &Vec<i32>) {
        println!("Stack: {:#?}", values)
    }

    fn render_pointer(&mut self, pointer: (usize, usize)) {
        println!("Pointer: {:#?}", pointer)
    }
}

impl BefungeExecution {
    pub fn new(field: FungeField) -> Self {
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

    pub fn run_with_renderer(&mut self, renderer: &mut dyn FungeRenderer) {
        renderer.render_field(&self.field.cells);
        while self.active {
            renderer.render_stack(&self.stack);
            renderer.render_pointer((self.pc_x, self.pc_y));
            self.step();
        }
    }

    pub fn run_with_terminal(&mut self) {
        let mut term = TerminalRenderer::new(80, 25);

        term.init();
        term.render_field(&self.field.cells);
        term.render_pointer((self.pc_x, self.pc_y));

        while self.active {
            // term.render_stack(&self.stack);
            // term.render_pointer((self.pc_x, self.pc_y));
            self.step_and_render(&mut term);
            thread::sleep_ms(250);
        }

        term.stop();
    }

    pub fn step(&mut self) {
        self.step_and_render(&mut PrintlnRenderer {});
    }

    pub fn step_and_render(&mut self, renderer: &mut dyn FungeRenderer) {
        if self.active {
            renderer.render_pointer((self.pc_x, self.pc_y));
            renderer.render_stack(&self.stack);
            if let Some(curr) = self.field.get(self.pc_x, self.pc_y) {
                if self.string_mode {
                    if curr == BefungeCommand::TOGGLE_STRING_MODE.into() {
                        self.string_mode = false;
                    } else {
                        self.stack.push(curr as i32);
                    }
                } else {
                    match curr as u8 {
                        BefungeCommand::NO_OP => {}
                        BefungeCommand::NEGATE => {
                            let top = self.stack.pop().unwrap_or_default();

                            self.stack.push(if top > 0 { 0 } else { 1 });
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
                            let i = renderer.read_number();
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
                            renderer.write_character(top);
                        }
                        BefungeCommand::SUBTRACT => {
                            let top = self.stack.pop().unwrap_or_default();
                            let second = self.stack.pop().unwrap_or_default();

                            self.stack.push(top - second);
                        }
                        BefungeCommand::WRITE_INT => {
                            let top = self.stack.pop().unwrap_or_default();
                            renderer.write_number(top);
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

                            self.stack.push(if top > second { 1 } else { 0 });
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
                            renderer.render_field(&self.field.cells);
                        }
                        BefungeCommand::DOWN => {
                            self.pc_delta = Delta::Down;
                        }
                        BefungeCommand::IF_UP_DOWN => {
                            let top = self.stack.pop().unwrap_or_default();

                            self.pc_delta = if top > 0 { Delta::Up } else { Delta::Down };
                        }
                        BefungeCommand::READ_CHAR => {
                            let c = renderer.read_character();
                            self.stack.push(c);
                        }
                        b'0'..=b'9' => {
                            self.stack.push((curr - 48) as i32);
                        }
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
        let field = FungeField::new(80, 25);
        assert_eq!(field.get(0, 0), Some(BefungeCommand::NO_OP));
        assert_eq!(field.get(79, 24), Some(BefungeCommand::NO_OP));
        assert_eq!(field.get(80, 0), None);
    }

    #[test]
    fn test_string_field() {
        let field = FungeField::from_str("0\n1\n", 80, 25);
        assert_eq!(field.get(0, 0), Some(b'0'));
        assert_eq!(field.get(1, 0), Some(b' '));
        assert_eq!(field.get(0, 1), Some(b'1'));
        assert_eq!(field.get(0, 2), Some(b' '));
        assert_eq!(field.get(79, 24), Some(b' '));
    }

    #[test]
    fn test_truncate() {
        let field = FungeField::from_str("012\n01\n01", 2, 2);
        assert_eq!(field.get(0, 0), Some(b'0'));
        assert_eq!(field.get(1, 0), Some(b'1'));
        assert_eq!(field.get(2, 0), None);
        assert_eq!(field.get(0, 1), Some(b'0'));
        assert_eq!(field.get(1, 1), Some(b'1'));
        assert_eq!(field.get(0, 2), None);
    }

    #[test]
    fn test_horizontal_wrap_right() {
        let mut exec = BefungeExecution::new(FungeField::new(2, 1));
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
        let mut exec = BefungeExecution::new(FungeField::from_str("<", 3, 1));
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
        let mut exec = BefungeExecution::new(FungeField::from_str("v", 1, 2));
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
        let mut exec = BefungeExecution::new(FungeField::from_str("^", 1, 2));
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
        let mut exec = BefungeExecution::new(FungeField::from_str("0123456789", 10, 1));

        for _i in 0..10 {
            exec.step()
        }

        assert_eq!(exec.stack(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
    }

    #[test]
    fn test_string_mode() {
        let mut exec = BefungeExecution::new(FungeField::from_str("\"0123456789\"0", 13, 1));

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
        let mut exec = BefungeExecution::new(FungeField::from_str("g", 1, 1));

        exec.step();

        assert_eq!(exec.stack(), vec![103])
    }

    #[test]
    fn test_write_cell() {
        let mut exec = BefungeExecution::new(FungeField::from_str("p", 1, 1));

        exec.step();

        assert_eq!(exec.get(0, 0), Some(0));
    }

    #[test]
    fn test_negate() {
        let mut exec = BefungeExecution::new(FungeField::from_str("!!", 2, 1));

        exec.step();
        assert_eq!(exec.stack(), vec![1]);
        exec.step();
        assert_eq!(exec.stack(), vec![0]);
    }

    #[test]
    fn test_swap() {
        let mut exec = BefungeExecution::new(FungeField::from_str("01\\", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![1, 0]);
    }

    #[test]
    fn test_add() {
        let mut exec = BefungeExecution::new(FungeField::from_str("12+", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![3]);
    }

    #[test]
    fn test_subtract() {
        let mut exec = BefungeExecution::new(FungeField::from_str("12-", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![1]);
    }

    #[test]
    fn test_multiply() {
        let mut exec = BefungeExecution::new(FungeField::from_str("12*", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![2]);
    }

    #[test]
    fn test_divide() {
        let mut exec = BefungeExecution::new(FungeField::from_str("12/", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![2]);
    }

    #[test]
    fn test_modulo() {
        let mut exec = BefungeExecution::new(FungeField::from_str("23%", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![1]);
    }

    #[test]
    fn test_compare() {
        let mut exec = BefungeExecution::new(FungeField::from_str("12`", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![1]);

        let mut exec = BefungeExecution::new(FungeField::from_str("21`", 3, 1));

        exec.step();
        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![0]);
    }

    #[test]
    fn test_duplicate() {
        let mut exec = BefungeExecution::new(FungeField::from_str("1:", 2, 1));

        exec.step();
        exec.step();
        assert_eq!(exec.stack(), vec![1, 1]);
    }

    #[test]
    fn test_discard() {
        let mut exec = BefungeExecution::new(FungeField::from_str("1$", 2, 1));

        exec.step();
        assert_eq!(exec.stack(), vec![1]);
        exec.step();
        assert_eq!(exec.stack(), vec![]);
    }

    #[test]
    fn test_if_left_right() {
        let mut exec = BefungeExecution::new(FungeField::from_str("1_", 2, 1));

        exec.step();
        assert_eq!(exec.stack(), vec![1]);
        exec.step();
        assert_eq!(exec.stack(), vec![]);
        let (x, _y, delta) = exec.pc();
        assert_eq!(x, 0);
        assert_eq!(delta, Delta::Left);

        let mut exec = BefungeExecution::new(FungeField::from_str("0_", 2, 1));
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
        let mut exec = BefungeExecution::new(FungeField::from_str("1|", 2, 2));

        exec.step();
        assert_eq!(exec.stack(), vec![1]);
        exec.step();
        assert_eq!(exec.stack(), vec![]);
        let (_x, y, delta) = exec.pc();
        assert_eq!(y, 1);
        assert_eq!(delta, Delta::Up);

        let mut exec = BefungeExecution::new(FungeField::from_str("0|", 2, 2));
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
        let mut exec = BefungeExecution::new(FungeField::from_str("12..", 4, 1));
        exec.step();
        exec.step();
        exec.step();
        exec.step();
    }

    #[test]
    fn test_write_char() {
        let mut exec = BefungeExecution::new(FungeField::from_str("\"a\",", 4, 1));
        exec.step();
        exec.step();
        exec.step();
        exec.step();
    }
}

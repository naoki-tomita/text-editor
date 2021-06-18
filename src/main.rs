extern crate libc;

use std::fs::File;
use std::io::{Read, StdoutLock, Write};
use std::time::Duration;
use std::{env, io, thread, usize};

// https://abrakatabura.hatenablog.com/entry/2017/09/20/065024
fn init_terminal() -> libc::termios {
    let mut saved_termattr = libc::termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_cc: [0u8; 20],
        c_ispeed: 0,
        c_ospeed: 0,
    };
    unsafe {
        let ptr = &mut saved_termattr;
        libc::tcgetattr(0, ptr);
    }
    let mut termattr = saved_termattr;
    termattr.c_lflag = termattr.c_lflag & !(libc::ICANON | libc::ECHO);
    termattr.c_cc[libc::VMIN] = 1;
    termattr.c_cc[libc::VTIME] = 0;
    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, &termattr);
    }
    unsafe {
        libc::fcntl(0, libc::F_SETFL, libc::O_NONBLOCK);
    }

    saved_termattr
}

fn restore_terminal(terminal_attribute: libc::termios) {
    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, &terminal_attribute);
    }
}

fn read_key(ptr: &mut [u8; 1]) -> isize {
    unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 1) }
}

struct Console<'a> {
    pub lock: StdoutLock<'a>,
}

impl<'a> Console<'a> {
    fn new(lock: StdoutLock<'a>) -> Self {
        Console { lock }
    }

    fn clear(&mut self) {
        let _ = self
            .lock
            .write(format!("{esc}[2J{esc}[1;1H", esc = 27 as char).as_bytes());
    }

    fn write(&mut self, str: String) {
        let _ = self.lock.write(str.as_bytes());
        let _ = self.lock.flush();
    }
}

struct KeyboardReader {}
impl KeyboardReader {}

fn main() -> io::Result<()> {
    let _ = init_terminal();

    let stdout = io::stdout();
    let lock = stdout.lock();
    let mut console = Console::new(lock);

    let args = env::args().collect::<Vec<String>>();
    let mut file = File::open(args.get(1).unwrap())?;
    let mut text_buf = String::new();
    file.read_to_string(&mut text_buf)?;

    let mut text_editor = TextEditor::new(text_buf);
    let mut buf: [u8; 1] = [0; 1];
    loop {
        buf[0] = 0;
        read_key(&mut buf);
        match buf[0] {
            27 => {
                read_key(&mut buf);
                read_key(&mut buf);
                match buf[0] {
                    65 => text_editor.move_up(),
                    66 => text_editor.move_down(),
                    67 => text_editor.move_right(),
                    68 => text_editor.move_left(),
                    _ => (),
                };
            }
            127 => text_editor.delete(),
            0 => (),
            key => text_editor.insert(key as char),
        };
        let text = text_editor.get_text();
        console.clear();
        println!("x: {:?}, y: {:?}", text_editor.get_current_x(), text_editor.get_current_y());
        console.write(text);
        thread::sleep(Duration::from_millis(1000 / 30));
    }
    // restore_terminal(saved_termattr);
}

struct TextEditor {
    pub text: String,
    pub cursor: usize,
}

impl TextEditor {
    fn new(text: String) -> Self {
        Self {
            text,
            cursor: 0,
        }
    }

    fn get_line_number(&self) -> usize {
        let left_side = self.text.split_at(self.cursor).0;
        left_side.split("\n").count()
    }

    fn get_text(&self) -> String {
        let mut text = self.text.clone();
        text.insert(self.cursor, '|');
        text
    }

    fn insert(&mut self, char: char) {
        self.text.insert(self.cursor, char);
        self.move_right();
    }

    // fn jump_cursor(&mut self, cursor: Position) {
    //     self.cursor = cursor;
    // }

    fn current_line_len(&self) -> usize {
        self.get_lines()[self.cursor].len()
    }

    fn get_lines(&self) -> Vec<&str> {
        self.text.split("\n").collect::<Vec<&str>>()
    }

    fn get_current_x(&self) -> usize {
        self.text.split_at(self.cursor).0.split("\n").last().unwrap().len()
    }

    fn get_line_len(&self, at: usize) -> usize {
        self.get_lines().get(at).unwrap().len()
    }

    fn get_current_y(&self) -> usize {
        self.text.split_at(self.cursor).0.split("\n").count() - 1
    }

    fn line_count(&self) -> usize {
        self.text.split("\n").count()
    }

    fn move_right(&mut self) {
        if self.cursor == self.text.len() {
            return;
        }
        self.cursor = self.cursor + 1;
    }

    fn move_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor = self.cursor - 1;
    }

    fn move_up(&mut self) {
        if self.get_current_y() == 0 {
            return;
        }
        self.cursor = self.cursor - self.get_line_len(self.get_current_y() - 1) - 1 //
    }

    fn move_down(&mut self) {
        if self.line_count() - 1 < self.get_current_y() {
            return;
        }
        self.cursor = self.cursor + self.get_line_len(self.get_current_y()) + 1;
    }

    fn delete(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.text.remove(self.cursor - 1);
        self.move_left();
    }
}

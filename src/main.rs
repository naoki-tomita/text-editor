extern crate libc;

use std::io::{StdoutLock, Write};
use std::{io, thread, usize};
use std::time::{Duration};


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
    pub lock: StdoutLock<'a>
}

impl <'a> Console<'a> {
    fn new(lock: StdoutLock<'a>) -> Self {
        Console { lock }
    }

    fn clear(&mut self) {
        let _ = self.lock.write(format!("{esc}[2J{esc}[1;1H", esc = 27 as char).as_bytes());
    }

    fn write(&mut self, str: String) {
        let _ = self.lock.write(str.as_bytes());
        let _ = self.lock.flush();
    }
}

fn main() {
    println!("Hello, world!");
    let _ = init_terminal();
    let stdout = io::stdout();
    let lock = stdout.lock();
    let mut console = Console::new(lock);
    let mut buf: [u8; 1] = [0; 1];
    let mut text_editor = TextEditor::new("foo bar".to_string());


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
                    _ => ()
                };
            }
            127 => text_editor.delete(),
            0 => (),
            key => text_editor.insert(key as char),
        };
        let text = text_editor.get_text();
        console.clear();
        console.write(text);
        thread::sleep(Duration::from_millis(1000 / 30));
    }
    // restore_terminal(saved_termattr);
}

struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn move_right(&mut self) {
        self.x = self.x + 1;
    }

    fn move_left(&mut self) {
        self.x = self.x - 1;
    }

    fn move_up(&mut self) {
        self.y = self.y - 1;
    }

    fn move_down(&mut self) {
        self.y = self.y - 1;
    }
}

struct TextEditor {
    pub text: String,
    pub cursor: Position,
}

impl TextEditor {
    fn new(text: String) -> Self {
        Self {
            text,
            cursor: Position { x: 0, y: 0 },
        }
    }

    fn get_idx(&self) -> usize {
        let x = self.cursor.x;
        let y = self.cursor.y;
        let splitted = self.text.split("\n");
        splitted
            .map(|v| v)
            .collect::<Vec<&str>>()
            .split_at(y as usize)
            .0
            .iter()
            .fold(0 as usize, |curr, str| curr + str.len()) + x as usize
    }

    fn get_text(&self) -> String {
        let idx = self.get_idx();
        let mut text = self.text.clone();
        text.insert(idx as usize, '|');
        text
    }

    fn insert(&mut self, char: char) {
        self.text.insert(self.get_idx(), char);
        self.move_right();
    }

    // fn jump_cursor(&mut self, cursor: Position) {
    //     self.cursor = cursor;
    // }

    fn move_right(&mut self) {
        self.cursor.move_right();
    }

    fn move_left(&mut self) {
        self.cursor.move_left();
    }

    fn move_up(&mut self) {
        self.cursor.move_up();
    }

    fn move_down(&mut self) {
        self.cursor.move_down();
    }

    fn delete(&mut self) {
        self.text.remove(self.get_idx() - 1);
        self.move_left();
    }
}

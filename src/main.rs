extern crate termion;

mod chip8;

use chip8::C8R;
use std::io::{stdout, Read, Write};
use termion::raw::IntoRawMode;

fn main() {
    // Contains original terminal state. Restored when dropped
    let _old_state = terminal_setup();
    let mut c8r = C8R::new();
    c8r.load_rom(&std::env::args().nth(1).expect("Enter rom path"));
    let clock_rate: u64;
    if let Some(val) = std::env::args().nth(2) {
        clock_rate = val.parse().unwrap();
    } else {
        clock_rate = 500;
    }
    let mut stdin = termion::async_stdin().bytes();
    let mut timer_decrement = std::time::Instant::now();
    let mut time_start;
    let cycle = std::time::Duration::from_millis(1000 / clock_rate);
    loop {
        time_start = std::time::Instant::now();
        c8r.cpu_step();
        let key_res = stdin.next();
        if let Some(Ok(key)) = key_res {
            if key == b'\x1B' {
                print!("{}", termion::cursor::Goto(0, 17),);
                break;
            } else if chip8::KEYS[..].contains(&key) {
                c8r.key = chip8::KEYS[..].iter().position(|val| *val == key).unwrap() as u8;
            } else {
                c8r.key = 0;
            }
        }
        while {
            match stdin.next() {
                Some(_) => true,
                None => false,
            }
        } {}
        let time_now = std::time::Instant::now();
        if time_now - timer_decrement > std::time::Duration::from_millis(16) {
            timer_decrement = time_now;
            if c8r.d_timer > 0 {
                c8r.d_timer -= 1;
            }
            if c8r.s_timer > 0 {
                c8r.s_timer -= 1;
            }
        }
        if time_now - time_start < cycle {
            std::thread::sleep(cycle - (time_now - time_start));
        }
    }
}

fn terminal_setup() -> termion::raw::RawTerminal<std::io::Stdout> {
    write!(
        stdout(),
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    )
    .unwrap();
    stdout().flush().unwrap();
    stdout().into_raw_mode().unwrap()
}

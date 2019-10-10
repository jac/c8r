use bitvec::prelude::*;
use std::fs::File;
use std::io::{BufReader, Read, Write};

const FONT_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

const PIXELS: [&str; 4] = [" ", "▄", "▀", "█"];
pub const KEYS: [u8; 16] = [
    b'x', b'1', b'2', b'3', b'q', b'w', b'e', b'a', b's', b'd', b'z', b'c', b'4', b'r', b'f', b'v',
];

pub struct C8R {
    v_reg: [u8; 16],
    memory: [u8; 0x1000],
    pc: u16,
    i: u16,
    sp: u16,
    pub s_timer: u8,
    pub d_timer: u8,
    pub key: u8,
}

impl C8R {
    pub fn new() -> Self {
        let mut c8r = C8R {
            v_reg: [0; 16],
            memory: [0; 0x1000],
            pc: 0x200,
            i: 0,
            sp: 0x150,
            s_timer: 0,
            d_timer: 0,
            key: 0,
        };
        // Copy FONT_DATA into memory
        c8r.memory[..80].copy_from_slice(&FONT_DATA);
        c8r
    }

    pub fn load_rom(&mut self, rom_path: &String) {
        let rom_file = File::open(rom_path).expect("Could not open ROM file");
        let reader = BufReader::new(rom_file);
        // Read ROM into memory starting from index 0x200
        let rom_section = &mut self.memory[0x200..];
        for (pos, byte_result) in reader.bytes().enumerate() {
            rom_section[pos] = byte_result.unwrap();
        }
    }

    pub fn cpu_step(&mut self) {
        let opcode = self.fetch();
        let decoded = self.decode(&opcode[..]);
        decoded.0(self, decoded.1);
    }

    fn fetch(&self) -> Vec<u8> {
        let mem = &self.memory[self.pc as usize..(self.pc + 2) as usize];
        let n1 = mem[0] >> 4;
        let n2 = mem[0] & 0xF;
        let n3 = mem[1] >> 4;
        let n4 = mem[1] & 0xF;
        vec![n1, n2, n3, n4]
    }

    fn decode(&self, op: &[u8]) -> (fn(&mut C8R, Vec<u16>), Vec<u16>) {
        match op {
            [0, 0, 0xE, 0] => (C8R::_00e0, vec![]),
            [0, 0, 0xE, 0xE] => (C8R::_00ee, vec![]),
            [1, _, _, _] => (C8R::_1nnn, vec![n_args(&op[1..])]),
            [2, _, _, _] => (C8R::_2nnn, vec![n_args(&op[1..])]),
            [3, x, _, _] => (C8R::_3xnn, vec![*x as u16, n_args(&op[2..])]),
            [4, x, _, _] => (C8R::_4xnn, vec![*x as u16, n_args(&op[2..])]),
            [5, x, y, 0] => (C8R::_5xy0, vec![*x as u16, *y as u16]),
            [6, x, _, _] => (C8R::_6xnn, vec![*x as u16, n_args(&op[2..])]),
            [7, x, _, _] => (C8R::_7xnn, vec![*x as u16, n_args(&op[2..])]),
            [8, x, y, 0] => (C8R::_8xy0, vec![*x as u16, *y as u16]),
            [8, x, y, 1] => (C8R::_8xy1, vec![*x as u16, *y as u16]),
            [8, x, y, 2] => (C8R::_8xy2, vec![*x as u16, *y as u16]),
            [8, x, y, 3] => (C8R::_8xy3, vec![*x as u16, *y as u16]),
            [8, x, y, 4] => (C8R::_8xy4, vec![*x as u16, *y as u16]),
            [8, x, y, 5] => (C8R::_8xy5, vec![*x as u16, *y as u16]),
            [8, x, y, 6] => (C8R::_8xy6, vec![*x as u16, *y as u16]),
            [8, x, y, 7] => (C8R::_8xy7, vec![*x as u16, *y as u16]),
            [8, x, y, 0xE] => (C8R::_8xye, vec![*x as u16, *y as u16]),
            [9, x, y, 0] => (C8R::_9xy0, vec![*x as u16, *y as u16]),
            [0xA, _, _, _] => (C8R::_annn, vec![n_args(&op[1..])]),
            [0xB, _, _, _] => (C8R::_bnnn, vec![n_args(&op[1..])]),
            [0xC, x, _, _] => (C8R::_cxnn, vec![*x as u16, n_args(&op[2..])]),
            [0xD, x, y, _] => (C8R::_dxyn, vec![*x as u16, *y as u16, n_args(&op[3..])]),
            [0xE, x, 9, 0xE] => (C8R::_ex9e, vec![*x as u16]),
            [0xE, x, 0xA, 1] => (C8R::_exa1, vec![*x as u16]),
            [0xF, x, 0, 7] => (C8R::_fx07, vec![*x as u16]),
            [0xF, x, 0, 0xA] => (C8R::_fx0a, vec![*x as u16]),
            [0xF, x, 1, 5] => (C8R::_fx15, vec![*x as u16]),
            [0xF, x, 1, 8] => (C8R::_fx18, vec![*x as u16]),
            [0xF, x, 1, 0xE] => (C8R::_fx1e, vec![*x as u16]),
            [0xF, x, 2, 9] => (C8R::_fx29, vec![*x as u16]),
            [0xF, x, 3, 3] => (C8R::_fx33, vec![*x as u16]),
            [0xF, x, 5, 5] => (C8R::_fx55, vec![*x as u16]),
            [0xF, x, 6, 5] => (C8R::_fx65, vec![*x as u16]),
            [0, _, _, _] => (C8R::_0nnn, vec![]),
            _ => (C8R::_err, vec![]),
        }
    }

    fn _0nnn(&mut self, _args: Vec<u16>) {
        panic!(
            "{}EOF or Hardware Subroutine @: {:#0x}",
            termion::cursor::Goto(1, 17),
            self.pc - 0x200
        );
    }

    fn _00e0(&mut self, _args: Vec<u16>) {
        self.memory[0x50..0x150].iter_mut().for_each(|x| *x = 0);
        print!("{}", termion::clear::All);
        self.pc += 2;
    }

    fn _00ee(&mut self, _args: Vec<u16>) {
        let upper = (self.memory[self.sp as usize] as u16) << 8;
        let lower = (self.memory[(self.sp + 1) as usize]) as u16;
        self.sp -= 2;
        self.pc = upper | lower;
    }

    fn _1nnn(&mut self, args: Vec<u16>) {
        self.pc = args[0];
    }

    fn _2nnn(&mut self, args: Vec<u16>) {
        let ret = self.pc + 2;
        self.sp += 2;
        self.memory[self.sp as usize] = (ret >> 8) as u8;
        self.memory[(self.sp + 1) as usize] = (ret & 0xFF) as u8;
        self.pc = args[0];
    }

    fn _3xnn(&mut self, args: Vec<u16>) {
        if self.v_reg[args[0] as usize] == args[1] as u8 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn _4xnn(&mut self, args: Vec<u16>) {
        if self.v_reg[args[0] as usize] != args[1] as u8 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn _5xy0(&mut self, args: Vec<u16>) {
        if self.v_reg[args[0] as usize] == self.v_reg[args[1] as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn _6xnn(&mut self, args: Vec<u16>) {
        self.v_reg[args[0] as usize] = args[1] as u8;
        self.pc += 2;
    }

    fn _7xnn(&mut self, args: Vec<u16>) {
        self.v_reg[args[0] as usize] += args[1] as u8;
        self.pc += 2;
    }

    fn _8xy0(&mut self, args: Vec<u16>) {
        self.v_reg[args[0] as usize] = self.v_reg[args[1] as usize];
        self.pc += 2;
    }

    fn _8xy1(&mut self, args: Vec<u16>) {
        self.v_reg[args[0] as usize] |= self.v_reg[args[1] as usize];
        self.pc += 2;
    }

    fn _8xy2(&mut self, args: Vec<u16>) {
        self.v_reg[args[0] as usize] &= self.v_reg[args[1] as usize];
        self.pc += 2;
    }

    fn _8xy3(&mut self, args: Vec<u16>) {
        self.v_reg[args[0] as usize] ^= self.v_reg[args[1] as usize];
        self.pc += 2;
    }

    fn _8xy4(&mut self, args: Vec<u16>) {
        let x = self.v_reg[args[0] as usize];
        let y = self.v_reg[args[1] as usize];
        self.v_reg[0xF] = (0xFF - x < y) as u8;
        self.v_reg[args[0] as usize] = x + y;
        self.pc += 2;
    }

    fn _8xy5(&mut self, args: Vec<u16>) {
        let x = self.v_reg[args[0] as usize];
        let y = self.v_reg[args[1] as usize];
        self.v_reg[0xF] = (x > y) as u8;
        self.v_reg[args[0] as usize] = x - y;
        self.pc += 2;
    }

    fn _8xy6(&mut self, args: Vec<u16>) {
        let temp = self.v_reg[args[0] as usize];
        self.v_reg[0xF] = temp & 1;
        self.v_reg[args[0] as usize] = temp >> 1;
        self.pc += 2;
    }

    fn _8xy7(&mut self, args: Vec<u16>) {
        let x = self.v_reg[args[0] as usize];
        let y = self.v_reg[args[1] as usize];
        self.v_reg[0xF] = (y > x) as u8;
        self.v_reg[args[0] as usize] = y - x;
        self.pc += 2;
    }

    fn _8xye(&mut self, args: Vec<u16>) {
        let temp = (self.v_reg[args[0] as usize] as u8).rotate_left(1);
        self.v_reg[0xF] = temp & 1;
        self.v_reg[args[0] as usize] = temp & 0xFE;
        self.pc += 2;
    }

    fn _9xy0(&mut self, args: Vec<u16>) {
        if self.v_reg[args[0] as usize] != self.v_reg[args[1] as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn _annn(&mut self, args: Vec<u16>) {
        self.i = args[0] & 0xFFF;
        self.pc += 2;
    }

    fn _bnnn(&mut self, args: Vec<u16>) {
        self.pc = self.v_reg[0] as u16 + args[0];
    }

    fn _cxnn(&mut self, args: Vec<u16>) {
        let rand = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            & args[1] as u64;
        self.v_reg[args[0] as usize] = rand as u8;
        self.pc += 2;
    }

    fn _dxyn(&mut self, args: Vec<u16>) {
        self.v_reg[0xF] = 0;
        let x = (self.v_reg[args[0] as usize] % 64) as u16;
        let y = (self.v_reg[args[1] as usize] % 32) as u16;
        let bit_offset = x % 8;
        let base_byte_offset = y * 8 + x / 8 + 0x50;
        for row in 0..args[2].min(32 - y) {
            let byte_offset = (base_byte_offset + row * 8) as usize;
            let secondary_offset = (byte_offset as u16 + 8 - (16 * ((y + row) % 2))) as usize;
            let scnd = BitVec::<BigEndian, u8>::from_slice(
                &self.memory[secondary_offset..secondary_offset + 2],
            );
            let sprite = self.memory[(self.i + row) as usize];

            let mem = BitSlice::<BigEndian, u8>::from_slice_mut(
                &mut self.memory[byte_offset..byte_offset + 2],
            );
            print!("{}", termion::cursor::Goto(x + 1, (y + row) / 2 + 1));
            for bit in 0..8.min(64 - x) {
                let sprite_bit = ((sprite >> (7 - bit)) & 1) == 1;
                let old = mem.get((bit_offset + bit) as usize).unwrap();
                let new = old ^ sprite_bit;
                mem.set((bit_offset + bit) as usize, new);
                self.v_reg[0xF] |= (old && !new) as u8;
                let new = new as u8;
                let scnd_pixel = scnd.get((bit_offset + bit) as usize).unwrap() as u8;
                if (row + y) % 2 == 0 {
                    print!("{}", PIXELS[(new << 1 | scnd_pixel) as usize]);
                } else {
                    print!("{}", PIXELS[(scnd_pixel << 1 | new) as usize]);
                }
            }
        }
        std::io::stdout().flush().unwrap();
        self.pc += 2;
    }

    fn _ex9e(&mut self, args: Vec<u16>) {
        if self.v_reg[args[0] as usize] == self.key {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn _exa1(&mut self, args: Vec<u16>) {
        if self.v_reg[args[0] as usize] != self.key {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn _fx07(&mut self, args: Vec<u16>) {
        self.v_reg[args[0] as usize] = self.d_timer;
        self.pc += 2;
    }

    fn _fx0a(&mut self, args: Vec<u16>) {
        let mut keys = std::io::stdin().bytes();
        let mut key;
        while {
            key = keys.next().unwrap().unwrap();
            if let Some(pos) = KEYS[..].iter().position(|val| *val == key) {
                self.v_reg[args[0] as usize] = pos as u8;
                false
            } else {
                true
            }
        } {}
        self.pc += 2;
    }

    fn _fx15(&mut self, args: Vec<u16>) {
        self.d_timer = self.v_reg[args[0] as usize];
        self.pc += 2;
    }

    fn _fx18(&mut self, args: Vec<u16>) {
        self.s_timer = self.v_reg[args[0] as usize];
        self.pc += 2;
    }

    fn _fx1e(&mut self, args: Vec<u16>) {
        self.i += self.v_reg[args[0] as usize] as u16;
        self.i &= 0xFFF;
        self.pc += 2;
    }

    fn _fx29(&mut self, args: Vec<u16>) {
        self.i = (self.v_reg[args[0] as usize] * 5) as u16;
        self.pc += 2;
    }

    fn _fx33(&mut self, args: Vec<u16>) {
        let val = self.v_reg[args[0] as usize];
        self.memory[self.i as usize] = val / 100;
        self.memory[(self.i + 1) as usize] = (val % 100) / 10;
        self.memory[(self.i + 2) as usize] = val % 10;
        self.pc += 2;
    }

    fn _fx55(&mut self, args: Vec<u16>) {
        self.memory[self.i as usize..=(self.i + args[0]) as usize]
            .copy_from_slice(&self.v_reg[..=args[0] as usize]);
        // self.i += args[0] + 1;
        self.i &= 0xFFF;
        self.pc += 2;
    }

    fn _fx65(&mut self, args: Vec<u16>) {
        self.v_reg[..=args[0] as usize]
            .copy_from_slice(&self.memory[self.i as usize..=(self.i + args[0]) as usize]);
        // self.i += args[0] + 1;
        self.i &= 0xFFF;
        self.pc += 2;
    }

    fn _err(&mut self, _args: Vec<u16>) {
        panic!("Invalid instruction at {:#0x}", self.pc - 0x200);
    }
}

fn n_args(args: &[u8]) -> u16 {
    let mut res = 0;
    for (pos, arg) in args.iter().enumerate() {
        res |= (*arg as u16) << ((args.len() - pos - 1) * 4)
    }
    res
}

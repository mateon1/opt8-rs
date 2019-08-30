use std::io::Read;
use std::time::{Instant, Duration};

use minifb::{Window, WindowOptions, Scale};

use opt8::core;

static SYSTEM_FONT: &'static [u8] = &[
	0xF0, 0x90, 0x90, 0x90, 0xF0, // 0x0
	0x20, 0x60, 0x20, 0x20, 0x70, // 0x1
	0xF0, 0x10, 0xF0, 0x80, 0xF0, // 0x2
	0xF0, 0x10, 0xF0, 0x10, 0xF0, // 0x3
	0x90, 0x90, 0xF0, 0x10, 0x10, // 0x4
	0xF0, 0x80, 0xF0, 0x10, 0xF0, // 0x5
	0xF0, 0x80, 0xF0, 0x90, 0xF0, // 0x6
	0xF0, 0x10, 0x20, 0x40, 0x40, // 0x7
	0xF0, 0x90, 0xF0, 0x90, 0xF0, // 0x8
	0xF0, 0x90, 0xF0, 0x10, 0xF0, // 0x9
	0xF0, 0x90, 0xF0, 0x90, 0x90, // 0xA
	0xE0, 0x90, 0xE0, 0x90, 0xE0, // 0xB
	0xF0, 0x80, 0x80, 0x80, 0xF0, // 0xC
	0xE0, 0x90, 0x90, 0x90, 0xE0, // 0xD
	0xF0, 0x80, 0xF0, 0x80, 0xF0, // 0xE
	0xF0, 0x80, 0xF0, 0x80, 0x80, // 0xF
];

struct ConcreteMachine {
	mem: [u8; 0x1000],
	reg: [u8; 16],
	sound: u8,
	delay: u8,
	i: u16,
	pc: u16,
	stack: Vec<u16>,
	screen_buf: [u32; 64 * 32],
	screen_clean: bool,
	screen_dirty: bool,
	window: Window,
	last_tick: Instant,
}

impl ConcreteMachine {
	fn new(rom: &[u8]) -> ConcreteMachine {
		let mut this = ConcreteMachine {
			mem: [0; 0x1000],
			reg: [0; 16],
			sound: 0,
			delay: 0,
			i: 0,
			pc: 0x200,
			stack: vec![],
			screen_buf: [0xFF000000; 64 * 32],
			screen_clean: true,
			screen_dirty: false,
			window: Window::new("Chip-8 Concrete Interpreter", 64, 32,
				WindowOptions { scale: Scale::X8, ..Default::default() }).unwrap(),
			last_tick: Instant::now(),
		};
		(&mut this.mem[0..SYSTEM_FONT.len()]).copy_from_slice(SYSTEM_FONT);
		(&mut this.mem[0x200..(rom.len() + 0x200)]).copy_from_slice(rom);
		this.window.update_with_buffer(&this.screen_buf).unwrap();
		this.window.update();
		this
	}
	fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		self.last_tick = Instant::now();
		while self.window.is_open() {
			self.window.update();
			for _i in 0..0x1 { // FIXME: hardcoded number of insts...
				core::run_instruction(self);
			}
			// This multiplication now gives the number of ticks, and the remaining time!
			let now = Instant::now();
			let elapsed = now.duration_since(self.last_tick) * 60;
			
			if elapsed.as_secs() > 0 {
				self.sound = (self.sound as u64).saturating_sub(elapsed.as_secs()) as u8;
				self.delay = (self.delay as u64).saturating_sub(elapsed.as_secs()) as u8;
			}
			self.last_tick = now - Duration::from_nanos(elapsed.subsec_nanos() as u64 / 60);
			if self.screen_dirty {
				self.window.update_with_buffer(&self.screen_buf)?;
				self.screen_dirty = false;
			}
			
			std::thread::sleep(Duration::from_millis(30)); // FIXME: hardcoded sleep...
		} // FIXME: This loop just sucks in general
		Ok(())
	}
}

impl core::Chip8State for ConcreteMachine {
	fn read_gp_register(&self, r: u8) -> u8 {
		self.reg[r as usize]
	}
	fn write_gp_register(&mut self, r: u8, v: u8) {
		self.reg[r as usize] = v;
	}
	fn get_pc(&self) -> u16 {
		self.pc
	}
	fn set_pc(&mut self, v: u16) {
		self.pc = v;
	}
	fn get_i(&self) -> u16 {
		self.i
	}
	fn set_i(&mut self, v: u16) {
		self.i = v;
	}
	fn stack_push(&mut self, v: u16) {
		self.stack.push(v);
	}
	fn stack_pop(&mut self) -> u16 {
		self.stack.pop().unwrap()
	}
	fn read_mem(&self, addr: u16) -> u8 {
		self.mem[(addr & 0xFFF) as usize]
	}
	fn write_mem(&mut self, addr: u16, v: u8) {
		self.mem[(addr & 0xFFF) as usize] = v;
	}
	fn clear_screen(&mut self) {
		if self.screen_clean { return; }
		for px in self.screen_buf.iter_mut() {
			*px = 0xFF000000;
		}
		self.screen_clean = true;
		self.screen_dirty = true;
	}
	fn screen_xor_line(&mut self, x: u8, y: u8, bits: u8) -> bool {
		if bits == 0 { return false; }
		self.screen_clean = false;
		self.screen_dirty = true;
		let x = (x & 0x3F) as usize;
		let y = (y & 0x1F) as usize;
		let mut unset = false;
		for xo in 0..8 {
			if x + xo > 0x3F { break; }
			let idx = y << 6 | (x + xo);
			if bits & 1u8 << (7 - xo) != 0 {
				self.screen_buf[idx] ^= 0xFFFFFF;
				unset |= self.screen_buf[idx] != 0xFFFFFFFF;
			}
		}
		unset
	}
	fn get_key_status(&self, _k: u8) -> bool { false }
	fn wait_for_keypress(&mut self) -> u8 {
		loop {}
	}
	fn get_hex_char_addr(&self, ch: u8) -> u16 {
		ch as u16 * 5
	}
}

fn main() {
	let mut args = std::env::args();
	let _ = args.next().unwrap(); // program name
	let path = args.next().expect("Need positional argument");
	let mut data = vec![];
	std::fs::File::open(path).unwrap().read_to_end(&mut data).unwrap();
	for inst in data.chunks_exact_mut(2) {
		// REPLACE SLOPPY SHIFTS
		if inst[0] & 0xF0 == 0x80 && (inst[1] & 0x0F == 0x06 || inst[1] & 0x0F == 0x0E) {
			inst[1] = (inst[0] & 0x0F) << 4 | inst[1] & 0x0F
		}
	}
	assert!(data.len() < 0xDFF);
	ConcreteMachine::new(&data).run().unwrap();
}

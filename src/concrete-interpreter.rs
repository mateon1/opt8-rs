use std::io::Read;
use std::time::{Instant, Duration};

use minifb::{Window, WindowOptions, Scale};

use opt8::core;

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
			screen_buf: [0; 64 * 32],
			screen_clean: true,
			screen_dirty: false,
			window: Window::new("Chip-8 Concrete Interpreter", 64, 32,
				WindowOptions { scale: Scale::X8, ..Default::default() }).unwrap(),
			last_tick: Instant::now(),
		};
		(&mut this.mem[0x200..(rom.len() + 0x200)]).copy_from_slice(rom);
		this.window.update_with_buffer(&this.screen_buf);
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
			let mut elapsed = now.duration_since(self.last_tick) * 60;
			
			if elapsed.as_secs() > 0 {
				self.sound = (self.sound as u64).saturating_sub(elapsed.as_secs()) as u8;
				self.delay = (self.delay as u64).saturating_sub(elapsed.as_secs()) as u8;
			}
			self.last_tick = now - Duration::from_nanos(elapsed.subsec_nanos() as u64 / 60);
			if self.screen_dirty {
				self.window.update_with_buffer(&self.screen_buf);
				self.screen_dirty = false;
			}
			
			std::thread::sleep(Duration::from_millis(10)); // FIXME: hardcoded sleep...
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
			*px = 0;
		}
		self.screen_clean = true;
		self.screen_dirty = true;
	}
	fn screen_xor_line(&mut self, x: u8, y: u8, bits: u8) -> bool {
		if bits == 0 { return false; }
		self.screen_clean = false;
		self.screen_dirty = true;
		let idx = ((y & 0x1f) as usize) << 7 | ((x & 0x3f) as usize);
		let mut smol = self.screen_buf[idx] as u8;
		smol ^= bits;
		let big = smol as u32;
		self.screen_buf[idx] = 0xff000000 | (big << 8) | (big << 4) | big;
		smol & bits != bits
	}
	fn get_key_status(&self, k: u8) -> bool { false }
	fn wait_for_keypress(&mut self) -> u8 {
		loop {}
	}
}

fn main() {
	let mut args = std::env::args();
	let _ = args.next().unwrap(); // program name
	let path = args.next().expect("Need positional argument");
	let mut data = vec![];
	std::fs::File::open(path).unwrap().read_to_end(&mut data).unwrap();
	assert!(data.len() < 0xDFF);
	ConcreteMachine::new(&data).run().unwrap();
}

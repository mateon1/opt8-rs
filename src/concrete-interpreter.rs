use std::io::Read;

use minifb::{Window, WindowOptions, Scale};

use opt8::core::*;

struct ConcreteMachine {
	mem: [u8; 0x1000],
	reg: [u8; 16],
	sound: u8,
	delay: u8,
	i: u16,
	pc: u16,
	stack: Vec<u16>,
	screen_buf: [u32; 64 * 32],
	window: Window,
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
			window: Window::new("Chip-8 Concrete Interpreter", 64, 32,
				WindowOptions { scale: Scale::X8, ..Default::default() }).unwrap()
		};
		(&mut this.mem[0x200..(rom.len() + 0x200)]).copy_from_slice(rom);
		this.window.update_with_buffer(&this.screen_buf);
		this.window.update();
		this
	}
	fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		loop {}
	}
}

impl Chip8State for ConcreteMachine {
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
	fn clear_screen(&mut self) {}
	fn screen_xor_line(&mut self, x: u8, y: u8, bits: u8) -> bool {
		false
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

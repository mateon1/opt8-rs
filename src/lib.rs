pub mod core {
	// Re-implementing this trait can be used for debugging
	pub trait Chip8State {
		fn read_gp_register(&self, r: u8) -> u8;
		fn write_gp_register(&mut self, r: u8, v: u8);
		fn get_pc(&self) -> u16;
		fn set_pc(&mut self, address: u16);
		fn get_i(&self) -> u16;
		fn set_i(&mut self, v: u16);
		fn stack_push(&mut self, v: u16);
		fn stack_pop(&mut self) -> u16;
		fn read_mem(&self, addr: u16) -> u8;
		fn write_mem(&mut self, addr: u16, v: u8);
		fn clear_screen(&mut self);
		fn screen_xor_line(&mut self, x: u16, y: u16, bits: u8);
		fn get_key_status(&self, k: u8) -> bool;
		fn wait_for_keypress(&mut self) -> u8;
	}

	pub enum Register8 {
		Generic(u8), // Must be < 16
		DelayTimer,
		SoundTimer,
	}

	enum IntermediateValue {
		Bool(bool),
		Addr(u16),
		Byte(u8),
	}

	pub enum IntermediateInst {
		Illegal,

		ClearScreen,
		/// Pops x address, y address, and a byte of toggled bits
		/// Returns a boolean collision flag
		WriteScreen,
		WaitForKeypress,
		KeyPressed,
		/// Returns the memory address where hex characters are stored
		/// They must be stored in increasing order, such that each
		/// character can be found at `base_addr + 5 * digit`
		HexCharBase,

		RegRead(Register8),
		RegWrite(Register8),
		MemRead, // pops addr, returns u8
		MemWrite, // pops addr, value
		GetI,
		SetI,
		GetNextPC,
		SetPC,
		OffsetPC, // UNCLEAR
		CondSkipPC,
		StackPush,
		StackPop,

		PushImm(u8),
		PushImm16(u16),
		Swap2,
		PopIgnore,
		Rand,

		// Bool instructions
		True,
		False,
		Equal,
		BoolNot,
		BoolOr,
		/// Equivalent to a ternary operator
		/// v1, v2, true  => v1
		/// v1, v2, false => v2
		Select,

		// 8-bit instructions
		Add,
		Sub,
		BOr,
		BAnd,
		BXor,
		// 8-bit unary
		BShl,
		BShr,

		// 12-bit instructions
		AddOffset,
	}

	pub fn parse_instruction(inst: u16) -> Option<Vec<IntermediateInst>> {
		// Super Chip-48 instructions
		// 00Cn - SCD nibble
		// 00FB - SCR
		// 00FC - SCL
		// 00FD - EXIT
		// 00FE - LOW
		// 00FF - HIGH
		// Dxy0 - DRW Vx, Vy, 0
		// Fx30 - LD HF, Vx
		// Fx75 - LD R, Vx
		// Fx85 - LD Vx, R

		use IntermediateInst::*;
		let n1 = ((inst & 0xF000) >> 12) as u8;
		let n2 = ((inst & 0x0F00) >>  8) as u8;
		let n3 = ((inst & 0x00F0) >>  4) as u8;
		let n4 = ((inst & 0x000F)      ) as u8;
		Some(match n1 {
			0x0 => {
				// SYS
				match inst {
					0x00E0 => vec![ClearScreen],
					0x00EE => vec![StackPop, SetPC],
					_ => return None,
				}
			},
			0x1 => vec![                      PushImm16(inst & 0xFFF), SetPC],
			0x2 => vec![GetNextPC, StackPush, PushImm16(inst & 0xFFF), SetPC],
			0x3 => vec![RegRead(Register8::Generic(n2)), PushImm(inst as u8),             Equal,          CondSkipPC],
			0x4 => vec![RegRead(Register8::Generic(n2)), PushImm(inst as u8),             Equal, BoolNot, CondSkipPC],
			0x5 if n4 == 0
			    => vec![RegRead(Register8::Generic(n2)), RegRead(Register8::Generic(n3)), Equal,          CondSkipPC],
			0x6 => vec![                                 PushImm(inst as u8),      RegWrite(Register8::Generic(n2))],
			0x7 => vec![RegRead(Register8::Generic(n2)), PushImm(inst as u8), Add, RegWrite(Register8::Generic(n2))],
			0x8 => {
				// Ops
				unimplemented!()
/*
			(  8,   x,   y,   0) => Chip8Instruction::Move { dst: x, src: y },
			(  8,   x,   y,   1) => Chip8Instruction::BitOr { dst: x, src: y },
			(  8,   x,   y,   2) => Chip8Instruction::BitAnd { dst: x, src: y },
			(  8,   x,   y,   3) => Chip8Instruction::BitXor { dst: x, src: y },
			(  8,   x,   y,   4) => Chip8Instruction::Add { dst: x, src: y },
			(  8,   x,   y,   5) => Chip8Instruction::Sub { dst: x, src: y },
			(  8,   x,   y,   6) => Chip8Instruction::BitShr { dst: x, src: y },
			(  8,   x,   y,   7) => Chip8Instruction::SubNeg { dst: x, src: y },
			(  8,   x,   y, 0xE) => Chip8Instruction::BitShl { dst: x, src: y },
*/
			},
			0x9 if n4 == 0
			    => vec![RegRead(Register8::Generic(n2)), RegRead(Register8::Generic(n3)), Equal, BoolNot, CondSkipPC],
			0xA => vec![PushImm16(inst & 0xFFF), SetI],
			0xB => vec![PushImm16(inst & 0xFFF), OffsetPC],
			0xC => vec![Rand, PushImm(inst as u8), BAnd, RegWrite(Register8::Generic(n2))],
			0xD => {
				// Draw to screen
				// XXX: What if X or Y are VF?
				// TODO: Compare behavior to other emulators and remove these asserts later
				assert!(n2 != 0xF && n3 != 0xF);
				assert!(n4 != 0);
				// NOTE: THIS IMPLEMENTATION IS INCORRECT
				// The vertical wrapping behavior is wrong!
				let mut v = vec![PushImm(1), PushImm(0), False];
				for i in 0..n4 {
					v.push(RegRead(Register8::Generic(n2)));
					v.push(RegRead(Register8::Generic(n3)));
					if i != 0 {
						v.push(PushImm(i));
						v.push(Add);
					}
					v.push(GetI);
					if i != 0 {
						v.push(PushImm(i));
						v.push(AddOffset);
					}
					v.push(MemRead);
					v.push(WriteScreen);
					v.push(BoolOr);
				}
				v.push(Select);
				v.push(RegWrite(Register8::Generic(0xF)));
				v
			},
			0xE => {
				// Keyboard
				// XXX: Behavior is unclear!
				match (n3, n4) {
					(0x9, 0xE) => vec![RegRead(Register8::Generic(n2)), KeyPressed,          CondSkipPC],
					(0xA, 0x1) => vec![RegRead(Register8::Generic(n2)), KeyPressed, BoolNot, CondSkipPC],
					_ => return None,
				}
			},
			0xF => {
				// Special
				unimplemented!()
/*
			(0xF,   x,   0,   7) => Chip8Instruction::ReadDelayTimer(x),
			(0xF,   x,   0, 0xA) => Chip8Instruction::ReadKey(x),
			(0xF,   x,   1,   5) => Chip8Instruction::SetDelayTimer(x),
			(0xF,   x,   1,   8) => Chip8Instruction::SetSoundTimer(x),
			(0xF,   x,   1, 0xE) => Chip8Instruction::IndexI(x),
			(0xF,   x,   2,   9) => Chip8Instruction::LoadHexCharAddress(x),
			(0xF,   x,   3,   3) => Chip8Instruction::ToDecimal(x),
			(0xF,   x,   5,   5) => Chip8Instruction::BatchWrite(x),
			(0xF,   x,   6,   5) => Chip8Instruction::BatchRead(x),
*/
			},
			_ => return None,
		})
	}

	pub fn run_instruction<S: Chip8State>(s: &mut S) {
		match parse_instruction((s.read_mem(s.get_pc()) as u16) << 8 | s.read_mem(s.get_pc() + 1) as u16) {
			_ => unimplemented!()
		}
	}
}

mod tests {
	// TODO
}

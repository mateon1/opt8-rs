mod core {
	// Re-implementing this trait can be used for debugging
	trait Chip8State {
		fn read_gp_register(r: u8) -> u8;
		fn write_gp_register(r: u8, v: u8);
		fn get_pc() -> u16;
		fn set_pc(address: u16);
		fn get_i() -> u16
		fn set_i(v: u16);
		fn stack_push(v: u16);
		fn stack_pop() -> u16;
		fn read_mem(addr: u16) -> u8;
		fn write_mem(addr: u16, v: u8);
		fn clear_screen();
		fn screen_xor_line(x: u16, y: u16, bits: u8);
		fn get_key_status(k: u8) -> bool;
	}

	#[derive(Debug, PartialEq)]
	enum Chip8Instruction {
		IllegalOpcode(u16),
		Sys(u16)
		ClearScreen,
		ReturnFromSubroutine,
		Jump(u16),
		Call(u16),
		SkipNextEqImm { reg: u8, value: u8 },
		SkipNextNeImm { reg: u8, value: u8 },
		SkipNextEq(u8, u8),
		SkipNextNe(u8, u8),
		Load { reg: u8, value: u8 },
		AddImm { reg: u8, value: u8 },
		Move { dst: u8, src: u8 },
		BitOr { dst: u8, src: u8 },
		BitAnd { dst: u8, src: u8 },
		BitXor { dst: u8, src: u8 },
		Add { dst: u8, src: u8 },
		Sub { dst: u8, src: u8 },
		BitShr { dst: u8, src: u8 }, // Note: Some implementations ignore src register
		SubNeg { dst: u8, src: u8 },
		BitShl { dst: u8, src: u8 }, // Note: Some implementations ignore src register
		SetI(u16),
		JumpRelV0(u16),
		Rand { dst: u8, mask: u8 },
		DrawSprite { x_reg: u8, y_reg: u8, bytes: u8},
		SkipKeyPressed(u8),
		SkipKeyNotPressed(u8),
		ReadDelayTimer(u8),
		ReadKey(u8),
		SetDelayTimer(u8),
		SetSoundTimer(u8),
		IndexI(u8),
		LoadHexCharAddress(u8),
		ToDecimal(u8),
		BatchWrite(u8),
		BatchRead(u8),
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
	}

	fn parse_instruction(inst: u16) -> Chip8Instruction {
		match ((inst >> 12) as u8, (inst >> 8) as u8 & 0xF, (inst >> 4) as u8 & 0xF, inst as u8 & 0xF) {
			(  0,   0, 0xE,   0) => Chip8Instruction::ClearScreen,
			(  0,   0, 0xE, 0xE) => Chip8Instruction::ReturnFromSubroutine,
			(  0,   _,   _,   _) => Chip8Instruction::Sys(inst & 0xFFF),
			(  1,   _,   _,   _) => Chip8Instruction::Jump(inst & 0xFFF),
			(  2,   _,   _,   _) => Chip8Instruction::Call(inst & 0xFFF),
			(  3,   x,   _,   _) => Chip8Instruction::SkipNextEqImm { reg: x, value: inst as u8 },
			(  4,   x,   _,   _) => Chip8Instruction::SkipNextNeImm { reg: x, value: inst as u8 },
			(  5,   x,   y,   0) => Chip8Instruction::SkipNextEq(x, y),
			(  6,   x,   _,   _) => Chip8Instruction::Load { reg: x, value: inst as u8 },
			(  7,   x,   _,   _) => Chip8Instruction::AddImm { reg: x, value: inst as u8 },
			(  8,   x,   y,   0) => Chip8Instruction::Move { dst: x, src: y },
			(  8,   x,   y,   1) => Chip8Instruction::BitOr { dst: x, src: y },
			(  8,   x,   y,   2) => Chip8Instruction::BitAnd { dst: x, src: y },
			(  8,   x,   y,   3) => Chip8Instruction::BitXor { dst: x, src: y },
			(  8,   x,   y,   4) => Chip8Instruction::Add { dst: x, src: y },
			(  8,   x,   y,   5) => Chip8Instruction::Sub { dst: x, src: y },
			(  8,   x,   y,   6) => Chip8Instruction::BitShr { dst: x, src: y },
			(  8,   x,   y,   7) => Chip8Instruction::SubNeg { dst: x, src: y },
			(  8,   x,   y, 0xE) => Chip8Instruction::BitShl { dst: x, src: y },
			(  9,   x,   y,   0) => Chip8Instruction::SkipNextNe(x, y),
			(0xA,   _,   _,   _) => Chip8Instruction::SetI(inst & 0xFFF),
			(0xB,   _,   _,   _) => Chip8Instruction::JumpRelV0(inst & 0xFFF),
			(0xC,   x,   _,   _) => Chip8Instruction::Rand { dst: src, mask: inst as u8 },
			(0xD,   x,   y,   n) => Chip8Instruction::DrawSprite { x_reg: x, y_reg: y, bytes: n },
			(0xE,   x,   9, 0xE) => Chip8Instruction::SkipKeyPressed(x),
			(0xE,   x, 0xA,   1) => Chip8Instruction::SkipKeyNotPressed(x),
			(0xF,   x,   0,   7) => Chip8Instruction::ReadDelayTimer(x),
			(0xF,   x,   0, 0xA) => Chip8Instruction::ReadKey(x),
			(0xF,   x,   1,   5) => Chip8Instruction::SetDelayTimer(x),
			(0xF,   x,   1,   8) => Chip8Instruction::SetSoundTimer(x),
			(0xF,   x,   1, 0xE) => Chip8Instruction::IndexI(x),
			(0xF,   x,   2,   9) => Chip8Instruction::LoadHexCharAddress(x),
			(0xF,   x,   3,   3) => Chip8Instruction::ToDecimal(x),
			(0xF,   x,   5,   5) => Chip8Instruction::BatchWrite(x),
			(0xF,   x,   6,   5) => Chip8Instruction::BatchRead(x),
		}
	}

	fn run_instruction<S: Chip8State>(s: &mut S) {
	}
}

mod tests {
	// TODO
}

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
		fn screen_xor_line(&mut self, x: u8, y: u8, bits: u8) -> bool;
		fn get_key_status(&self, k: u8) -> bool;
		fn wait_for_keypress(&mut self) -> u8;
		fn get_hex_char_addr(&self, ch: u8) -> u16;
	}

	#[derive(Clone, Copy, PartialEq, Debug)]
	pub enum Register8 {
		Generic(u8), // Must be < 16
		DelayTimer,
		SoundTimer,
	}

	#[derive(Clone, Copy, PartialEq, Debug)]
	enum IntermediateValue {
		Bool(bool),
		Addr(u16),
		Byte(u8),
	}

	#[derive(PartialEq, Clone, Copy, Debug)]
	pub enum IntermediateInst {
		Illegal,

		ClearScreen,
		/// Pops x address, y address, and a byte of toggled bits
		/// Returns a boolean collision flag
		WriteScreen,
		WaitForKeypress,
		KeyPressed,
		/// Returns the memory address where the appropriate hex character is stored
		HexCharAddr,

		RegRead(Register8),
		RegWrite(Register8),
		MemRead, // pops addr, returns u8
		MemWrite, // pops addr, value
		GetI,
		SetI,
		SetPC,
		OffsetPC, // UNCLEAR
		CondSkipPC,
		Call,
		Ret,

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
		AddOv,
		BOr,
		BAnd,
		BXor,
		DivMod10,
		// 8-bit unary
		Neg,
		BShlOv,
		BShrOv,

		// 12-bit instructions
		AddOffset,
	}

	impl IntermediateInst {
		fn execute<S: Chip8State>(self, state: &mut S, stack: &mut Vec<IntermediateValue>) -> bool {
			use IntermediateInst::*;
			use IntermediateValue::*;
			match self {
				Illegal => panic!("Executed illegal instruction"),
				ClearScreen => state.clear_screen(),
				WriteScreen => {
					let n = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					let y = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					let x = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					stack.push(Bool(state.screen_xor_line(x & 0x3F, y & 0x1F, n)));
				},
				WaitForKeypress => stack.push(Byte(state.wait_for_keypress())),
				KeyPressed => {
					let k = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					stack.push(Bool(state.get_key_status(k)));
				},
				HexCharAddr => {
					let c = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					assert!(c < 0x10); // XXX: What to do here?
					stack.push(Addr(state.get_hex_char_addr(c)));
				},

				RegRead(Register8::Generic(x)) => stack.push(Byte(state.read_gp_register(x))),
				RegWrite(Register8::Generic(x)) => {
					let v = match stack.pop().unwrap() {
						Byte(x) => x,
						Bool(x) => x as u8,
						_ => unreachable!(),
					};
					state.write_gp_register(x, v);
				},
				MemRead => {
					let a = match stack.pop().unwrap() { Addr(x) => x, _ => unreachable!(), };
					stack.push(Byte(state.read_mem(a & 0xFFF)));
				},
				MemWrite => {
					let v = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					let a = match stack.pop().unwrap() { Addr(x) => x, _ => unreachable!(), };
					state.write_mem(a & 0xFFF, v);
				},
				GetI => {
					stack.push(Addr(state.get_i()));
				},
				SetI => {
					let a = match stack.pop().unwrap() { Addr(x) => x, _ => unreachable!(), };
					state.set_i(a & 0xFFF);
				},
				SetPC => {
					let a = match stack.pop().unwrap() { Addr(x) => x, _ => unreachable!(), };
					state.set_pc(a & 0xFFF);
					return true;
				},
		//OffsetPC, // UNCLEAR
				CondSkipPC => {
					let b = match stack.pop().unwrap() { Bool(x) => x, _ => unreachable!(), };
					if b {
						state.set_pc(state.get_pc() + 4);
					}
					return b;
				},
		//Call,
		//Ret,

				PushImm(v) => stack.push(Byte(v)),
				PushImm16(v) => stack.push(Addr(v)),
				Swap2 => { let l = stack.len(); stack.swap(l - 2, l - 1); },
				PopIgnore => drop(stack.pop().unwrap()),
				Rand => unimplemented!(),

				True => stack.push(Bool(true)),
				False => stack.push(Bool(false)),
				Equal => {
					let a = stack.pop().unwrap();
					let b = stack.pop().unwrap();
					stack.push(Bool(match (a, b) {
						(Byte(a), Byte(b)) => a == b,
						_ => unimplemented!(),
					}));
				},
				BoolNot => {
					let v = match stack.pop().unwrap() { Bool(x) => x, _ => unreachable!(), };
					stack.push(Bool(!v));
				},
				BoolOr => {
					let a = match stack.pop().unwrap() { Bool(x) => x, _ => unreachable!(), };
					let b = match stack.pop().unwrap() { Bool(x) => x, _ => unreachable!(), };
					stack.push(Bool(a | b));
				},
				Select => {
					let s = match stack.pop().unwrap() { Bool(x) => x, _ => unreachable!(), };
					let b = stack.pop().unwrap();
					let a = stack.pop().unwrap();
					match (a, b) {
						(Byte(_), Byte(_)) => {}
						_ => unimplemented!(),
					}
					stack.push(if s { a } else { b });
				}

				Add => {
					let a = stack.pop().unwrap();
					let b = stack.pop().unwrap();
					match (a, b) {
						(Byte(a), Byte(b)) => { stack.push(Byte(a + b)); }
						_ => unimplemented!(),
					}
				}
				AddOv => {
					let a = stack.pop().unwrap();
					let b = stack.pop().unwrap();
					match (a, b) {
						(Byte(a), Byte(b)) => {
							let (sum, ov) = a.overflowing_add(b);
							stack.push(Byte(sum));
							stack.push(Bool(ov));
						}
						_ => unimplemented!(),
					}
				}
				BOr => {
					let a = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					let b = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					stack.push(Byte(a | b));
				},
				BAnd => {
					let a = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					let b = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					stack.push(Byte(a & b));
				},
				BXor => {
					let a = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					let b = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					stack.push(Byte(a ^ b));
				},
				DivMod10 => {
					let v = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					stack.push(Byte(v / 10));
					stack.push(Byte(v % 10));
				},
				Neg => {
					let v = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					stack.push(Byte(v.wrapping_neg()))
				},
				BShlOv => {
					let v = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					let o = v & 0x80 != 0;
					let v = v << 1;
					stack.push(Byte(v));
					stack.push(Bool(o));
				},
				BShrOv => {
					let v = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					let o = v & 1 != 0;
					let v = v >> 1;
					stack.push(Byte(v));
					stack.push(Bool(o));
				},

				// XXX UNCLEAR: Overflow?
				AddOffset => {
					let v = match stack.pop().unwrap() { Byte(x) => x, _ => unreachable!(), };
					let a = match stack.pop().unwrap() { Addr(x) => x, _ => unreachable!(), };
					stack.push(Addr((a + v as u16) & 0xFFF));
				},

				_ => unimplemented!(),
			}
			false
		}
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
					0x00EE => vec![Ret],
					_ => return None,
				}
			},
			0x1 => vec![PushImm16(inst & 0xFFF), SetPC],
			0x2 => vec![PushImm16(inst & 0xFFF), Call],
			0x3 => vec![RegRead(Register8::Generic(n2)), PushImm(inst as u8),             Equal,          CondSkipPC],
			0x4 => vec![RegRead(Register8::Generic(n2)), PushImm(inst as u8),             Equal, BoolNot, CondSkipPC],
			0x5 if n4 == 0
			    => vec![RegRead(Register8::Generic(n2)), RegRead(Register8::Generic(n3)), Equal,          CondSkipPC],
			0x6 => vec![                                 PushImm(inst as u8),      RegWrite(Register8::Generic(n2))],
			// XXX: Does this set VF?
			0x7 => vec![RegRead(Register8::Generic(n2)), PushImm(inst as u8), Add, RegWrite(Register8::Generic(n2))],
			0x8 => {
				// Ops
				let mut v = vec![RegRead(Register8::Generic(n2)), RegRead(Register8::Generic(n3))];
				match n4 {
					0x0 => { v.swap_remove(0); }
					0x1 => v.push(BOr),
					0x2 => v.push(BAnd),
					0x3 => v.push(BXor),
					0x4 => {
						// Add with overflow
						v.push(AddOv); // -> Res, Ov
						v.push(RegWrite(Register8::Generic(0xF)));
						assert!(n2 != 0xF); // XXX
					},
					0x5 => {
						v.push(Neg);
						v.push(AddOv);
						v.push(RegWrite(Register8::Generic(0xF)));
						assert!(n2 != 0xF); // XXX
					}
					0x6 => {
						// XXX: AMBIGUITY: Is VY used?
						v.swap_remove(0);
						v.push(BShrOv);
						v.push(RegWrite(Register8::Generic(0xF)));
						assert!(n2 != 0xF); // XXX
					}
					0x7 => {
						v.swap(0, 1);
						v.push(Neg);
						v.push(AddOv);
						v.push(RegWrite(Register8::Generic(0xF)));
						assert!(n2 != 0xF); // XXX
					}
					0xE => {
						// XXX: AMBIGUITY: Is VY used?
						v.swap_remove(0);
						v.push(BShlOv);
						v.push(RegWrite(Register8::Generic(0xF)));
						assert!(n2 != 0xF); // XXX
					}
					_ => return None,
				}
				v.push(RegWrite(Register8::Generic(n2)));
				v
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
				// NOTE: THIS IMPLEMENTATION IS INCORRECT
				// The vertical wrapping behavior is wrong!
				let mut v = vec![PushImm(1), PushImm(0), False];
				for i in 0..if n4 == 0 { 16 } else { n4 } {
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
				let x = Register8::Generic(n2);
				match inst as u8 {
					0x07 => vec![RegRead(Register8::DelayTimer), RegWrite(x)],
					0x0A => vec![WaitForKeypress, RegWrite(x)],
					0x15 => vec![RegRead(x), RegWrite(Register8::DelayTimer)],
					0x18 => vec![RegRead(x), RegWrite(Register8::SoundTimer)],
					0x1E => vec![GetI, RegRead(x), AddOffset, SetI], // XXX: Unclear - what to do when this overflows?
					0x29 => vec![RegRead(x), HexCharAddr, SetI],
					0x33 => vec![
						RegRead(x),
						DivMod10, GetI, PushImm(2), AddOffset, Swap2, MemWrite,
						DivMod10, GetI, PushImm(1), AddOffset, Swap2, MemWrite,
						GetI, Swap2, MemWrite],
					0x55 => {
						let mut v = Vec::with_capacity(n2 as usize * 7 + 7);
						for i in 0 ..= n2 {
							v.push(GetI);
							v.push(RegRead(Register8::Generic(i)));
							v.push(MemWrite);
							v.push(GetI);
							v.push(PushImm(1));
							v.push(AddOffset);
							v.push(SetI);
						}
						v
					},
					0x65 => {
						let mut v = Vec::with_capacity(n2 as usize * 7);
						for i in 0 ..= n2 {
							v.push(GetI);
							v.push(MemRead);
							v.push(RegWrite(Register8::Generic(i)));
							v.push(GetI);
							v.push(PushImm(1));
							v.push(AddOffset);
							v.push(SetI);
						}
						v
					},
					_ => return None,
				}
			},
			_ => return None,
		})
	}

	pub fn run_instruction<S: Chip8State>(s: &mut S) {
		let mut v = vec![];
		println!("PC = {:03x}", s.get_pc());
		let mut pc_flag = false;
		for il in parse_instruction((s.read_mem(s.get_pc()) as u16) << 8 | s.read_mem(s.get_pc() + 1) as u16).expect("Executed illegal instruction") {
			println!("IL: {:?} | Stack: {:?}", il, v);
			pc_flag |= il.execute(s, &mut v);
		}
		if !pc_flag {
			s.set_pc((s.get_pc() + 2) & 0xFFF); // XXX: Unclear on overflow
		}
		assert!(v.is_empty());
	}
}

mod tests {
	// TODO
}

// 7  bit  0
// ---- ----
// NV1B DIZC
// |||| ||||
// |||| |||+- Carry
// |||| ||+-- Zero
// |||| |+--- Interrupt Disable
// |||| +---- Decimal
// |||+------ (No CPU effect; see: the B flag)
// ||+------- (No CPU effect; always pushed as 1)
// |+-------- Overflow
// +--------- Negative
pub enum Flag {
    Negative,
    Overflow,
    Unused,
    Break,
    DecimalMode,
    Interrupt,
    Zero,
    Carry,
}

pub struct StatusFlags {
    pub n: bool,
    pub v: bool,
    pub u: bool,
    pub b: bool,
    pub d: bool,
    pub i: bool,
    pub z: bool,
    pub c: bool,
}
impl StatusFlags {
    pub fn print_status_flags_readable(&mut self) {
        // println!("Negative Flag: {}", self.n);
        // println!("Overflow Flag: {}", self.v);
        // println!("Unused Flag: {}", self.u);
        // println!("Break Flag: {}", self.b);
        // println!("Decimal Mode: {}", self.d);
        // println!("Interrupt Disable: {}", self.i);
        // println!("Zero Flag: {}", self.z);
        // println!("Carry Flag: {}\n", self.c);
        println!("Flags: NOUBDIZC");
        println!("       {:08b}", self.as_u8());
    }
    pub fn set_flag(&mut self, to_set_flag: Flag, flag_state: bool) {
        match to_set_flag {
            Flag::Negative => self.n = flag_state,
            Flag::Overflow => self.v = flag_state,
            Flag::Unused => self.u = flag_state,
            Flag::Break => self.b = flag_state,
            Flag::DecimalMode => self.d = flag_state,
            Flag::Interrupt => self.i = flag_state,
            Flag::Zero => self.z = flag_state,
            Flag::Carry => self.c = flag_state,
        }
    }

    pub fn as_u8(&mut self) -> u8 {
        let mut result = 0b00000000;

        result |= (self.n as u8) << 7;
        result |= (self.v as u8) << 6;
        result |= (self.u as u8) << 5;
        result |= (self.b as u8) << 4;
        result |= (self.d as u8) << 3;
        result |= (self.i as u8) << 2;
        result |= (self.z as u8) << 1;
        result |= self.c as u8;

        result
    }
}

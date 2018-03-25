use nom::{IResult, ErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    Sys(u16),
    Cls,
    Ret,
    Jp(u16),
    Call(u16),
    SeV(u8, u8), // Skip equal value
    SneV(u8, u8),// Skip non-equal value
    Se(u8, u8),  // Skip equal register
    LdV(u8, u8), // Load value
    AddV(u8, u8),// Add value
    Ld(u8, u8),  // Load register
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    Add(u8, u8), // Add registers
    Sub(u8, u8),
    Shr(u8),
    Subn(u8, u8),
    Shl(u8),
    Sne(u8, u8),
    LdI(u16),    // Load I register
    JpV0(u16),
    Rnd(u8, u8),
    Drw(u8, u8, u8),
    Skp(u8),
    Sknp(u8),
    LdDt(u8),     // Load with delay timer
    LdK(u8),      // Load key press
    LdTd(u8),     // Set delay timer to Vx
    LdSt(u8),     // Load sound timer
    AddI(u8),     // Add Vx to I register
    LdS(u8),      // Set I to sprite location for digit Vx
    LdBCD(u8),    // Store BCD representation of Vx
    LdVM(u8),     // Store reg V0-Vx in [I]
    LdMV(u8),     // Store [I] in V0-Vx
}

fn parse_noarg(inp: (&[u8], usize)) -> IResult<(&[u8], usize), Instruction> {
    let (remaining, constant) = match take_bits!(inp, u16, 16) {
        IResult::Done(remaining, constant) => (remaining, constant),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let ins = match constant {
        0x00E0 => Instruction::Cls,
        0x00EE => Instruction::Ret,
        _      => return IResult::Error(ErrorKind::TagBits),
    };

    IResult::Done(remaining, ins)
}

fn parse_onearg_nnn(inp: (&[u8], usize)) -> IResult<(&[u8], usize), Instruction> {
    let (remaining, group) = match take_bits!(inp, u8, 4) {
        IResult::Done(remaining, group) => (remaining, group),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };
    
    let (remaining, arg) = match take_bits!(remaining, u16, 12) {
        IResult::Done(remaining, arg) => (remaining, arg),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let ins = match group {
        0x0 => Instruction::Sys(arg),
        0x1 => Instruction::Jp(arg),
        0x2 => Instruction::Call(arg),
        0xA => Instruction::LdI(arg),
        0xB => Instruction::JpV0(arg),
        _   => return IResult::Error(ErrorKind::TagBits),
    };

    IResult::Done(remaining, ins)
}

fn parse_onearg_x(inp: (&[u8], usize)) -> IResult<(&[u8], usize), Instruction> {
    let (remaining, group) = match take_bits!(inp, u8, 4) {
        IResult::Done(remaining, group) => (remaining, group),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };
    
    let (remaining, x) = match take_bits!(remaining, u8, 4) {
        IResult::Done(remaining, x) => (remaining, x),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let (remaining, id) = match take_bits!(remaining, u8, 8) {
        IResult::Done(remaining, id) => (remaining, id),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let ins = match (group, id) {
        (0xE, 0x9E) => Instruction::Skp(x),
        (0xE, 0xA1) => Instruction::Sknp(x),
        (0xF, 0x07) => Instruction::LdDt(x),
        (0xF, 0x0A) => Instruction::LdK(x),
        (0xF, 0x15) => Instruction::LdTd(x),
        (0xF, 0x18) => Instruction::LdSt(x),
        (0xF, 0x1E) => Instruction::AddI(x),
        (0xF, 0x29) => Instruction::LdS(x),
        (0xF, 0x33) => Instruction::LdBCD(x),
        (0xF, 0x55) => Instruction::LdVM(x),
        (0xF, 0x65) => Instruction::LdMV(x),
        _           => return IResult::Error(ErrorKind::TagBits),
    };

    IResult::Done(remaining, ins)
}

fn parse_twoarg_xkk(inp: (&[u8], usize)) -> IResult<(&[u8], usize), Instruction> {
    let (remaining, group) = match take_bits!(inp, u8, 4) {
        IResult::Done(remaining, group) => (remaining, group),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let (remaining, x) = match take_bits!(remaining, u8, 4) {
        IResult::Done(remaining, x) => (remaining, x),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let (remaining, kk) = match take_bits!(remaining, u8, 8) {
        IResult::Done(remaining, kk) => (remaining, kk),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let ins = match group {
        0x3 => Instruction::SeV(x, kk),
        0x4 => Instruction::SneV(x, kk),
        0x6 => Instruction::LdV(x, kk),
        0x7 => Instruction::AddV(x, kk),
        0xC => Instruction::Rnd(x, kk),
        _   => return IResult::Error(ErrorKind::TagBits),
    };

    IResult::Done(remaining, ins)
}

fn parse_twoarg_xy(inp: (&[u8], usize)) -> IResult<(&[u8], usize), Instruction> {
    let (remaining, group) = match take_bits!(inp, u8, 4) {
        IResult::Done(remaining, group) => (remaining, group),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };
    
    let (remaining, x) = match take_bits!(remaining, u8, 4) {
        IResult::Done(remaining, x) => (remaining, x),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let (remaining, y) = match take_bits!(remaining, u8, 4) {
        IResult::Done(remaining, y) => (remaining, y),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let (remaining, id) = match take_bits!(remaining, u8, 4) {
        IResult::Done(remaining, id) => (remaining, id),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let ins = match (group, id) {
        (0x5, 0x0) => Instruction::Se(x, y),
        (0x8, 0x0) => Instruction::Ld(x, y),
        (0x8, 0x1) => Instruction::Or(x, y),
        (0x8, 0x2) => Instruction::And(x, y),
        (0x8, 0x3) => Instruction::Xor(x, y),
        (0x8, 0x4) => Instruction::Add(x, y),
        (0x8, 0x5) => Instruction::Sub(x, y),
        (0x8, 0x6) => Instruction::Shr(x),
        (0x8, 0x7) => Instruction::Subn(x, y),
        (0x8, 0xE) => Instruction::Shl(x),
        (0x9, 0x0) => Instruction::Sne(x, y),
        _          => return IResult::Error(ErrorKind::TagBits),
    };

    IResult::Done(remaining, ins)
}

fn parse_threearg(inp: (&[u8], usize)) -> IResult<(&[u8], usize), Instruction> {
    let (remaining, group) = match take_bits!(inp, u8, 4) {
        IResult::Done(remaining, group) => (remaining, group),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };
    
    let (remaining, x) = match take_bits!(remaining, u8, 4) {
        IResult::Done(remaining, x) => (remaining, x),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let (remaining, y) = match take_bits!(remaining, u8, 4) {
        IResult::Done(remaining, y) => (remaining, y),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };

    let (remaining, z) = match take_bits!(remaining, u8, 4) {
        IResult::Done(remaining, z) => (remaining, z),
        IResult::Error(e) => return IResult::Error(e),
        IResult::Incomplete(i) => return IResult::Incomplete(i),
    };
    
    let ins = match group {
        0xD => Instruction::Drw(x, y, z),
        _   => return IResult::Error(ErrorKind::TagBits),
    };

    IResult::Done(remaining, ins)
}

named!(parse_instruction<&[u8], Instruction>, do_parse!(
    result: bits!(alt!(
        parse_noarg
      | parse_onearg_nnn
      | parse_onearg_x
      | parse_twoarg_xkk
      | parse_twoarg_xy
      | parse_threearg
      )) >>
    (result)
));

named!(parse_instructions<&[u8], Vec<Instruction>>, do_parse!(
    result: many0!(parse_instruction) >>
    eof!() >>
    (result)
));

impl Instruction {
    pub fn from_slice_one(s: &[u8]) -> Option<Instruction> {
        let parsed = parse_instruction(s);

        match parsed {
           IResult::Done(_, o) => Some(o),
           IResult::Error(_) => None,
           IResult::Incomplete(_) => None,
        }
    }

    pub fn from_slice(s: &[u8]) -> Vec<Instruction> {
        let parsed = parse_instructions(s);

        match parsed.unwrap() {
            (_, o) => o,
        }
    }
}

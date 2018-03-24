#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Instruction {
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

named!(parse_instruction<&[u8; 2], Instruction>, do_parse!(
    group_type: take_bits!(u8, 4) >>
    (match group_type {
        0 => alt!(
              map!(tag_bits!(u16, 12, 0x0E0), |_| Cls)
            | map!(tag_bits!(u16, 12, 0x0EE), |_| Ret)
            | map!(take_bits!(u16, 12), |addr| Sys(addr))),
        1 => map!(take_bits!(u16, 12), |addr| Jp(addr)),
        2 => map!(take_bits!(u16, 12), |addr| Call(addr)),
        3 => map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 8)), |(Vx, byte)| SeV(Vx, byte)),
        4 => map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 8)), |(Vx, byte)| SneV(Vx, byte)),
        5 => (
            map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy)| Se(Vx, Vy))
            >> tag_bits!(u8, 4, 0x0)),
        6 => map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 8)), |(Vx, byte)| LdV(Vx, byte)),
        7 => map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 8)), |(Vx, byte)| AddV(Vx, byte)),
        8 => alt!(
              (map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy)| Ld(Vx, Vy)) >> tag_bits!(u8, 4, 0x0))
            | (map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy)| Or(Vx, Vy)) >> tag_bits!(u8, 4, 0x1))
            | (map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy)| And(Vx, Vy)) >> tag_bits!(u8, 4, 0x2))
            | (map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy)| Xor(Vx, Vy)) >> tag_bits!(u8, 4, 0x3))
            | (map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy)| Add(Vx, Vy)) >> tag_bits!(u8, 4, 0x4))
            | (map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy)| Sub(Vx, Vy)) >> tag_bits!(u8, 4, 0x5))
            | (map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy)| Shr(Vx, Vy)) >> tag_bits!(u8, 4, 0x6))
            | (map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy)| Subn(Vx, Vy)) >> tag_bits!(u8, 4, 0x7))
            | (map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy)| Shl(Vx, Vy)) >> tag_bits!(u8, 4, 0xe))),
        9 => map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy)| Sne(Vx, Vy)),
        0xa => map!(take_bits!(u16, 12), |addr| LdI(addr)),
        0xb => map!(take_bits!(u16, 12), |addr| JpV0(addr)),
        0xc => map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 8)), |(Vx, byte)| Rnd(Vx, byte)),
        0xd => map!(tuple!(take_bits!(u8, 4), take_bits!(u8, 4), take_bits!(u8, 4)), |(Vx, Vy, nibble)| Drw(Vx, Vy, nibble)),
        0xe => alt!(
                (map!(take_bits!(u8, 4), |Vx| Skp(Vx)) >> tag_bits!(u8, 8, 0x9e))
              | (map!(take_bits!(u8, 4), |Vx| Sknp(Vx)) >> tag_bits!(u8, 8, 0xa1))),
        0xf => alt!(
                (map!(take_bits!(u8, 4), |Vx| LdDt(Vx)) >> tag_bits!(u8, 8, 0x07))
              | (map!(take_bits!(u8, 4), |Vx| LdK(Vx)) >> tag_bits!(u8, 8, 0x0a))
              | (map!(take_bits!(u8, 4), |Vx| LdTd(Vx)) >> tag_bits!(u8, 8, 0x15))
              | (map!(take_bits!(u8, 4), |Vx| LdSt(Vx)) >> tag_bits!(u8, 8, 0x18))
              | (map!(take_bits!(u8, 4), |Vx| AddI(Vx)) >> tag_bits!(u8, 8, 0x1e))
              | (map!(take_bits!(u8, 4), |Vx| LdS(Vx)) >> tag_bits!(u8, 8, 0x29))
              | (map!(take_bits!(u8, 4), |Vx| LdBCD(Vx)) >> tag_bits!(u8, 8, 0x33))
              | (map!(take_bits!(u8, 4), |Vx| LdVM(Vx)) >> tag_bits!(u8, 8, 0x55))
              | (map!(take_bits!(u8, 4), |Vx| LdMV(Vx)) >> tag_bits!(u8, 8, 0x65))),
    }))
);

impl Instruction {
    fn from_slice(s: &[u8; 2]) -> Instruction {
        match parse_instruction(s).unwrap() {
           Done(I, O) => O,
        }
    }
}

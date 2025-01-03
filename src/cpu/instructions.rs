use super::{cpu::{Interrupt, StatusFlag}, Cpu};

#[derive(Clone, Copy, PartialEq)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative
}

type InstructionHandler = fn(&mut Cpu, AddressingMode) -> u8;

#[derive(Clone, Copy)]
pub struct Instruction {
    pub function: InstructionHandler,
    pub mode: AddressingMode,
    pub min_cycles: u8,
}

pub static OPCODE_TABLE: [Instruction; 256] = [
    Instruction { function: brk, mode: AddressingMode::Implied, min_cycles: 7 }, // x00
    Instruction { function: ora, mode: AddressingMode::IndirectX, min_cycles: 6 }, // x01
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // x02
    Instruction { function: slo, mode: AddressingMode::IndirectX, min_cycles: 8 }, // x03
    Instruction { function: nop, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x04
    Instruction { function: ora, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x05
    Instruction { function: asl, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // x06
    Instruction { function: slo, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // x07
    Instruction { function: php, mode: AddressingMode::Implied, min_cycles: 3 }, // x08
    Instruction { function: ora, mode: AddressingMode::Immediate, min_cycles: 2 }, // x09
    Instruction { function: asl, mode: AddressingMode::Accumulator, min_cycles: 2 }, // x0A
    Instruction { function: anc, mode: AddressingMode::Immediate, min_cycles: 4 }, // x0B
    Instruction { function: nop, mode: AddressingMode::Absolute, min_cycles: 4 }, // x0C
    Instruction { function: ora, mode: AddressingMode::Absolute, min_cycles: 4 }, // x0D
    Instruction { function: asl, mode: AddressingMode::Absolute, min_cycles: 6 }, // x0E
    Instruction { function: slo, mode: AddressingMode::Absolute, min_cycles: 6 }, // x0F
    Instruction { function: bpl, mode: AddressingMode::Relative, min_cycles: 2 }, // x10
    Instruction { function: ora, mode: AddressingMode::IndirectY, min_cycles: 5 }, // x11
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // x12
    Instruction { function: slo, mode: AddressingMode::IndirectY, min_cycles: 8 }, // x13
    Instruction { function: nop, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // x14
    Instruction { function: ora, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // x15
    Instruction { function: asl, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // x16
    Instruction { function: slo, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // x17
    Instruction { function: clc, mode: AddressingMode::Implied, min_cycles: 2 }, // x18
    Instruction { function: ora, mode: AddressingMode::AbsoluteY, min_cycles: 4 }, // x19
    Instruction { function: nop, mode: AddressingMode::Implied, min_cycles: 2 }, // x1A
    Instruction { function: slo, mode: AddressingMode::AbsoluteY, min_cycles: 7 }, // x1B
    Instruction { function: nop, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // x1C
    Instruction { function: ora, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // x1D
    Instruction { function: asl, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // x1E
    Instruction { function: slo, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // x1F
    Instruction { function: jsr, mode: AddressingMode::Absolute, min_cycles: 6 }, // x20
    Instruction { function: and, mode: AddressingMode::IndirectX, min_cycles: 6 }, // x21
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // x22
    Instruction { function: rla, mode: AddressingMode::IndirectX, min_cycles: 8 }, // x23
    Instruction { function: bit, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x24
    Instruction { function: and, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x25
    Instruction { function: rol, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // x26
    Instruction { function: rla, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // x27
    Instruction { function: plp, mode: AddressingMode::Implied, min_cycles: 4 }, // x28
    Instruction { function: and, mode: AddressingMode::Immediate, min_cycles: 2 }, // x29
    Instruction { function: rol, mode: AddressingMode::Accumulator, min_cycles: 2 }, // x2A
    Instruction { function: anc, mode: AddressingMode::Immediate, min_cycles: 2 }, // x2B
    Instruction { function: bit, mode: AddressingMode::Absolute, min_cycles: 4 }, // x2C
    Instruction { function: and, mode: AddressingMode::Absolute, min_cycles: 4 }, // x2D
    Instruction { function: rol, mode: AddressingMode::Absolute, min_cycles: 6 }, // x2E
    Instruction { function: rla, mode: AddressingMode::Absolute, min_cycles: 6 }, // x2F
    Instruction { function: bmi, mode: AddressingMode::Relative, min_cycles: 2 }, // x30
    Instruction { function: and, mode: AddressingMode::IndirectY, min_cycles: 5 }, // x31
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // x32
    Instruction { function: rla, mode: AddressingMode::IndirectY, min_cycles: 8 }, // x33
    Instruction { function: nop, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // x34
    Instruction { function: and, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // x35
    Instruction { function: rol, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // x36
    Instruction { function: rla, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // x37
    Instruction { function: sec, mode: AddressingMode::Implied, min_cycles: 2 }, // x38
    Instruction { function: and, mode: AddressingMode::AbsoluteY, min_cycles: 4 }, // x39
    Instruction { function: nop, mode: AddressingMode::Implied, min_cycles: 2 }, // x3A
    Instruction { function: rla, mode: AddressingMode::AbsoluteY, min_cycles: 7 }, // x3B
    Instruction { function: nop, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // x3C
    Instruction { function: and, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // x3D
    Instruction { function: rol, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // x3E
    Instruction { function: rla, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // x3F
    Instruction { function: rti, mode: AddressingMode::Implied, min_cycles: 6 }, // x40
    Instruction { function: eor, mode: AddressingMode::IndirectX, min_cycles: 6 }, // x41
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // x42
    Instruction { function: sre, mode: AddressingMode::IndirectX, min_cycles: 8 }, // x43
    Instruction { function: nop, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x44
    Instruction { function: eor, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x45
    Instruction { function: lsr, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // x46
    Instruction { function: sre, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // x47
    Instruction { function: pha, mode: AddressingMode::Implied, min_cycles: 3 }, // x48
    Instruction { function: eor, mode: AddressingMode::Immediate, min_cycles: 2 }, // x49
    Instruction { function: lsr, mode: AddressingMode::Accumulator, min_cycles: 2 }, // x4A
    Instruction { function: alr, mode: AddressingMode::Immediate, min_cycles: 2 }, // x4B
    Instruction { function: jmp, mode: AddressingMode::Absolute, min_cycles: 3 }, // x4C
    Instruction { function: eor, mode: AddressingMode::Absolute, min_cycles: 4 }, // x4D
    Instruction { function: lsr, mode: AddressingMode::Absolute, min_cycles: 6 }, // x4E
    Instruction { function: sre, mode: AddressingMode::Absolute, min_cycles: 6 }, // x4F
    Instruction { function: bvc, mode: AddressingMode::Relative, min_cycles: 2 }, // x50
    Instruction { function: eor, mode: AddressingMode::IndirectY, min_cycles: 5 }, // x51
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // x52
    Instruction { function: sre, mode: AddressingMode::IndirectY, min_cycles: 8 }, // x53
    Instruction { function: nop, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // x54
    Instruction { function: eor, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // x55
    Instruction { function: lsr, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // x56
    Instruction { function: sre, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // x57
    Instruction { function: cli, mode: AddressingMode::Implied, min_cycles: 2 }, // x58
    Instruction { function: eor, mode: AddressingMode::AbsoluteY, min_cycles: 4 }, // x59
    Instruction { function: nop, mode: AddressingMode::Implied, min_cycles: 2 }, // x5A
    Instruction { function: sre, mode: AddressingMode::AbsoluteY, min_cycles: 7 }, // x5B
    Instruction { function: nop, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // x5C
    Instruction { function: eor, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // x5D
    Instruction { function: lsr, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // x5E
    Instruction { function: sre, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // x5F
    Instruction { function: rts, mode: AddressingMode::Implied, min_cycles: 6 }, // x60
    Instruction { function: adc, mode: AddressingMode::IndirectX, min_cycles: 6 }, // x61
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // x62
    Instruction { function: rra, mode: AddressingMode::IndirectX, min_cycles: 8 }, // x63
    Instruction { function: nop, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x64
    Instruction { function: adc, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x65
    Instruction { function: ror, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // x66
    Instruction { function: rra, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // x67
    Instruction { function: pla, mode: AddressingMode::Implied, min_cycles: 4 }, // x68
    Instruction { function: adc, mode: AddressingMode::Immediate, min_cycles: 2 }, // x69
    Instruction { function: ror, mode: AddressingMode::Accumulator, min_cycles: 2 }, // x6A
    Instruction { function: arr, mode: AddressingMode::Immediate, min_cycles: 2 }, // x6B
    Instruction { function: jmp, mode: AddressingMode::Indirect, min_cycles: 5 }, // x6C
    Instruction { function: adc, mode: AddressingMode::Absolute, min_cycles: 4 }, // x6D
    Instruction { function: ror, mode: AddressingMode::Absolute, min_cycles: 6 }, // x6E
    Instruction { function: rra, mode: AddressingMode::Absolute, min_cycles: 6 }, // x6F
    Instruction { function: bvs, mode: AddressingMode::Relative, min_cycles: 2 }, // x70
    Instruction { function: adc, mode: AddressingMode::IndirectY, min_cycles: 5 }, // x71
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // x72
    Instruction { function: rra, mode: AddressingMode::IndirectY, min_cycles: 8 }, // x73
    Instruction { function: nop, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // x74
    Instruction { function: adc, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // x75
    Instruction { function: ror, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // x76
    Instruction { function: rra, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // x77
    Instruction { function: sei, mode: AddressingMode::Implied, min_cycles: 2 }, // x78
    Instruction { function: adc, mode: AddressingMode::AbsoluteY, min_cycles: 4 }, // x79
    Instruction { function: nop, mode: AddressingMode::Implied, min_cycles: 2 }, // x7A
    Instruction { function: rra, mode: AddressingMode::AbsoluteY, min_cycles: 7 }, // x7B
    Instruction { function: nop, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // x7C
    Instruction { function: adc, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // x7D
    Instruction { function: ror, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // x7E
    Instruction { function: rra, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // x7F
    Instruction { function: nop, mode: AddressingMode::Immediate, min_cycles: 2 }, // x80
    Instruction { function: sta, mode: AddressingMode::IndirectX, min_cycles: 6 }, // x81
    Instruction { function: nop, mode: AddressingMode::Immediate, min_cycles: 2 }, // x82
    Instruction { function: sax, mode: AddressingMode::IndirectX, min_cycles: 6 }, // x83
    Instruction { function: sty, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x84
    Instruction { function: sta, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x85
    Instruction { function: stx, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x86
    Instruction { function: sax, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // x87
    Instruction { function: dey, mode: AddressingMode::Implied, min_cycles: 2 }, // x88
    Instruction { function: nop, mode: AddressingMode::Immediate, min_cycles: 2 }, // x89
    Instruction { function: txa, mode: AddressingMode::Implied, min_cycles: 2 }, // x8A
    Instruction { function: ane, mode: AddressingMode::Immediate, min_cycles: 2 }, // x8B
    Instruction { function: sty, mode: AddressingMode::Absolute, min_cycles: 4 }, // x8C
    Instruction { function: sta, mode: AddressingMode::Absolute, min_cycles: 4 }, // x8D
    Instruction { function: stx, mode: AddressingMode::Absolute, min_cycles: 4 }, // x8E
    Instruction { function: sax, mode: AddressingMode::Absolute, min_cycles: 4 }, // x8F
    Instruction { function: bcc, mode: AddressingMode::Relative, min_cycles: 2 }, // x90
    Instruction { function: sta, mode: AddressingMode::IndirectY, min_cycles: 6 }, // x91
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // x92
    Instruction { function: sha, mode: AddressingMode::IndirectY, min_cycles: 6 }, // x93
    Instruction { function: sty, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // x94
    Instruction { function: sta, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // x95
    Instruction { function: stx, mode: AddressingMode::ZeroPageY, min_cycles: 4 }, // x96
    Instruction { function: sax, mode: AddressingMode::ZeroPageY, min_cycles: 4 }, // x97
    Instruction { function: tya, mode: AddressingMode::Implied, min_cycles: 2 }, // x98
    Instruction { function: sta, mode: AddressingMode::AbsoluteY, min_cycles: 5 }, // x99
    Instruction { function: txs, mode: AddressingMode::Implied, min_cycles: 2 }, // x9A
    Instruction { function: tas, mode: AddressingMode::AbsoluteY, min_cycles: 5 }, // x9B
    Instruction { function: shy, mode: AddressingMode::AbsoluteX, min_cycles: 5 }, // x9C
    Instruction { function: sta, mode: AddressingMode::AbsoluteX, min_cycles: 5 }, // x9D
    Instruction { function: shx, mode: AddressingMode::AbsoluteY, min_cycles: 5 }, // x9E
    Instruction { function: sha, mode: AddressingMode::AbsoluteY, min_cycles: 5 }, // x9F
    Instruction { function: ldy, mode: AddressingMode::Immediate, min_cycles: 2 }, // xA0
    Instruction { function: lda, mode: AddressingMode::IndirectX, min_cycles: 6 }, // xA1
    Instruction { function: ldx, mode: AddressingMode::Immediate, min_cycles: 2 }, // xA2
    Instruction { function: lax, mode: AddressingMode::IndirectX, min_cycles: 6 }, // xA3
    Instruction { function: ldy, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // xA4
    Instruction { function: lda, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // xA5
    Instruction { function: ldx, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // xA6
    Instruction { function: lax, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // xA7
    Instruction { function: tay, mode: AddressingMode::Implied, min_cycles: 2 }, // xA8
    Instruction { function: lda, mode: AddressingMode::Immediate, min_cycles: 2 }, // xA9
    Instruction { function: tax, mode: AddressingMode::Implied, min_cycles: 2 }, // xAA
    Instruction { function: lxa, mode: AddressingMode::Immediate, min_cycles: 2 }, // xAB
    Instruction { function: ldy, mode: AddressingMode::Absolute, min_cycles: 4 }, // xAC
    Instruction { function: lda, mode: AddressingMode::Absolute, min_cycles: 4 }, // xAD
    Instruction { function: ldx, mode: AddressingMode::Absolute, min_cycles: 4 }, // xAE
    Instruction { function: lax, mode: AddressingMode::Absolute, min_cycles: 4 }, // xAF
    Instruction { function: bcs, mode: AddressingMode::Relative, min_cycles: 2 }, // xB0
    Instruction { function: lda, mode: AddressingMode::IndirectY, min_cycles: 5 }, // xB1
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // xB2
    Instruction { function: lax, mode: AddressingMode::IndirectY, min_cycles: 5 }, // xB3
    Instruction { function: ldy, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // xB4
    Instruction { function: lda, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // xB5
    Instruction { function: ldx, mode: AddressingMode::ZeroPageY, min_cycles: 4 }, // xB6
    Instruction { function: lax, mode: AddressingMode::ZeroPageY, min_cycles: 4 }, // xB7
    Instruction { function: clv, mode: AddressingMode::Implied, min_cycles: 2 }, // xB8
    Instruction { function: lda, mode: AddressingMode::AbsoluteY, min_cycles: 4 }, // xB9
    Instruction { function: tsx, mode: AddressingMode::Implied, min_cycles: 2 }, // xBA
    Instruction { function: las, mode: AddressingMode::AbsoluteY, min_cycles: 4 }, // xBB
    Instruction { function: ldy, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // xBC
    Instruction { function: lda, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // xBD
    Instruction { function: ldx, mode: AddressingMode::AbsoluteY, min_cycles: 4 }, // xBE
    Instruction { function: lax, mode: AddressingMode::AbsoluteY, min_cycles: 4 }, // xBF
    Instruction { function: cpy, mode: AddressingMode::Immediate, min_cycles: 2 }, // xC0
    Instruction { function: cmp, mode: AddressingMode::IndirectX, min_cycles: 6 }, // xC1
    Instruction { function: nop, mode: AddressingMode::Immediate, min_cycles: 2 }, // xC2
    Instruction { function: dcp, mode: AddressingMode::IndirectX, min_cycles: 8 }, // xC3
    Instruction { function: cpy, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // xC4
    Instruction { function: cmp, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // xC5
    Instruction { function: dec, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // xC6
    Instruction { function: dcp, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // xC7
    Instruction { function: iny, mode: AddressingMode::Implied, min_cycles: 2 }, // xC8
    Instruction { function: cmp, mode: AddressingMode::Immediate, min_cycles: 2 }, // xC9
    Instruction { function: dex, mode: AddressingMode::Implied, min_cycles: 2 }, // xCA
    Instruction { function: sbx, mode: AddressingMode::Immediate, min_cycles: 2 }, // xCB
    Instruction { function: cpy, mode: AddressingMode::Absolute, min_cycles: 4 }, // xCC
    Instruction { function: cmp, mode: AddressingMode::Absolute, min_cycles: 4 }, // xCD
    Instruction { function: dec, mode: AddressingMode::Absolute, min_cycles: 6 }, // xCE
    Instruction { function: dcp, mode: AddressingMode::Absolute, min_cycles: 6 }, // xCF
    Instruction { function: bne, mode: AddressingMode::Relative, min_cycles: 2 }, // xD0
    Instruction { function: cmp, mode: AddressingMode::IndirectY, min_cycles: 5 }, // xD1
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // xD2
    Instruction { function: dcp, mode: AddressingMode::IndirectY, min_cycles: 8 }, // xD3
    Instruction { function: nop, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // xD4
    Instruction { function: cmp, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // xD5
    Instruction { function: dec, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // xD6
    Instruction { function: dcp, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // xD7
    Instruction { function: cld, mode: AddressingMode::Implied, min_cycles: 2 }, // xD8
    Instruction { function: cmp, mode: AddressingMode::AbsoluteY, min_cycles: 4 }, // xD9
    Instruction { function: nop, mode: AddressingMode::Implied, min_cycles: 2 }, // xDA
    Instruction { function: dcp, mode: AddressingMode::AbsoluteY, min_cycles: 7 }, // xDB
    Instruction { function: nop, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // xDC
    Instruction { function: cmp, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // xDD
    Instruction { function: dec, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // xDE
    Instruction { function: dcp, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // xDF
    Instruction { function: cpx, mode: AddressingMode::Immediate, min_cycles: 2 }, // xE0
    Instruction { function: sbc, mode: AddressingMode::IndirectX, min_cycles: 6 }, // xE1
    Instruction { function: nop, mode: AddressingMode::Immediate, min_cycles: 2 }, // xE2
    Instruction { function: isc, mode: AddressingMode::IndirectX, min_cycles: 8 }, // xE3
    Instruction { function: cpx, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // xE4
    Instruction { function: sbc, mode: AddressingMode::ZeroPage, min_cycles: 3 }, // xE5
    Instruction { function: inc, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // xE6
    Instruction { function: isc, mode: AddressingMode::ZeroPage, min_cycles: 5 }, // xE7
    Instruction { function: inx, mode: AddressingMode::Implied, min_cycles: 2 }, // xE8
    Instruction { function: sbc, mode: AddressingMode::Immediate, min_cycles: 2 }, // xE9
    Instruction { function: nop, mode: AddressingMode::Implied, min_cycles: 2 }, // xEA
    Instruction { function: sbc, mode: AddressingMode::Immediate, min_cycles: 2 }, // xEB
    Instruction { function: cpx, mode: AddressingMode::Absolute, min_cycles: 4 }, // xEC
    Instruction { function: sbc, mode: AddressingMode::Absolute, min_cycles: 4 }, // xED
    Instruction { function: inc, mode: AddressingMode::Absolute, min_cycles: 6 }, // xEE
    Instruction { function: isc, mode: AddressingMode::Absolute, min_cycles: 6 }, // xEF
    Instruction { function: beq, mode: AddressingMode::Relative, min_cycles: 2 }, // xF0
    Instruction { function: sbc, mode: AddressingMode::IndirectY, min_cycles: 5 }, // xF1
    Instruction { function: jam, mode: AddressingMode::Implied, min_cycles: 0 }, // xF2
    Instruction { function: isc, mode: AddressingMode::IndirectY, min_cycles: 8 }, // xF3
    Instruction { function: nop, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // xF4
    Instruction { function: sbc, mode: AddressingMode::ZeroPageX, min_cycles: 4 }, // xF5
    Instruction { function: inc, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // xF6
    Instruction { function: isc, mode: AddressingMode::ZeroPageX, min_cycles: 6 }, // xF7
    Instruction { function: sed, mode: AddressingMode::Implied, min_cycles: 2 }, // xF8
    Instruction { function: sbc, mode: AddressingMode::AbsoluteY, min_cycles: 4 }, // xF9
    Instruction { function: nop, mode: AddressingMode::Implied, min_cycles: 2 }, // xFA
    Instruction { function: isc, mode: AddressingMode::AbsoluteY, min_cycles: 7 }, // xFB
    Instruction { function: nop, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // xFC
    Instruction { function: sbc, mode: AddressingMode::AbsoluteX, min_cycles: 4 }, // xFD
    Instruction { function: inc, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // xFE
    Instruction { function: isc, mode: AddressingMode::AbsoluteX, min_cycles: 7 }, // xFF
];

// // Official Instructions
//Access Instructions
fn lda(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (data, cycles) = if mode == AddressingMode::Immediate {
        (cpu.fetch_operand(), 0)
    }else{
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        let data = cpu.bus.read(addr);
        (data, cycles)
    };

    cpu.a = data;
    cpu.set_zero_negative_flag(cpu.a);
    cycles
}

fn sta(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    cpu.bus.write(addr, cpu.a);
    cycles
}

fn ldx(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (data, cycles) = if mode == AddressingMode::Immediate {
        (cpu.fetch_operand(), 0)
    }else {
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        (cpu.bus.read(addr), cycles)
    };
    cpu.x = data;
    cpu.set_zero_negative_flag(cpu.x);
    cycles
}

fn stx(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    cpu.bus.write(addr, cpu.x);
    cycles
}

fn ldy(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (data, cycles) = if mode == AddressingMode::Immediate {
        (cpu.fetch_operand(), 0)
    }else {
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        (cpu.bus.read(addr), cycles)
    };
    cpu.y = data;
    cpu.set_zero_negative_flag(cpu.y);
    cycles
}

fn sty(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    cpu.bus.write(addr, cpu.y);
    cycles
}

//Transfer Instructions
fn tax(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.x = cpu.a;
    cpu.set_zero_negative_flag(cpu.a);
    0
}

fn txa(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.a = cpu.x;
    cpu.set_zero_negative_flag(cpu.x);
    0
}

fn tay(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.y = cpu.a;
    cpu.set_zero_negative_flag(cpu.a);
    0
}

fn tya(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.a = cpu.y;
    cpu.set_zero_negative_flag(cpu.y);
    0
}

//Arithmetic Instructions
fn adc(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (data, cycles) = if mode == AddressingMode::Immediate {
        (cpu.fetch_operand(), 0)
    }else {
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        (cpu.bus.read(addr), cycles)
    };

    let result = u16::from(cpu.a).wrapping_add(u16::from(data)).wrapping_add(u16::from(cpu.get_carry_bit()));
    let final_result = result as u8;
    cpu.set_flag(StatusFlag::Carry, result > 0xFF);
    cpu.set_flag(StatusFlag::Zero, final_result == 0);
    cpu.set_flag(StatusFlag::Negative, (final_result & 0x80) != 0);
    cpu.set_flag(StatusFlag::Overflow, ((cpu.a ^ final_result) & (data ^ final_result) & 0x80) != 0);
    cpu.a = final_result;
    cycles
}

fn sbc(cpu: &mut Cpu, mode: AddressingMode) -> u8 {
    let mut total_cycles: u8 = 0;
    
    let data = if mode == AddressingMode::Immediate {
        cpu.fetch_operand()
    } else {
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        total_cycles += cycles;
        cpu.bus.read(addr)
    };

    // For subtraction, we use the complement of the carry flag
    // 1 - carry_bit becomes carry_bit - 1 to get the correct borrow
    let borrow = 1 - cpu.get_carry_bit();
    let result = cpu.a.wrapping_sub(data).wrapping_sub(borrow);

    // Carry is set when no borrow is needed, clear when borrow is needed
    // This is the opposite of what you might expect!
    cpu.set_flag(StatusFlag::Carry, (cpu.a as i16 - data as i16 - borrow as i16) >= 0);

    cpu.set_zero_negative_flag(result);

    // Overflow occurs when the sign of the result is different from what it should be
    // We need to consider both the data and the borrow in the calculation
    cpu.set_flag(
        StatusFlag::Overflow,
        ((cpu.a ^ result) & !(data ^ result) & 0x80) != 0
    );

    cpu.a = result;

    total_cycles
}

fn inc(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let data = cpu.bus.read(addr);
    let result = data.wrapping_add(1);
    cpu.set_zero_negative_flag(result);
    cpu.bus.write(addr, result);
    cycles
}

fn dec(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let data = cpu.bus.read(addr);
    let result = data.wrapping_sub(1);
    cpu.set_zero_negative_flag(result);
    cpu.bus.write(addr, result);
    cycles
}

fn inx(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    let result = cpu.x.wrapping_add(1);
    cpu.set_zero_negative_flag(result);
    cpu.x = result;
    0
}

fn dex(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    let result = cpu.x.wrapping_sub(1);
    cpu.set_zero_negative_flag(result);
    cpu.x = result;
    0
}

fn iny(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    let result = cpu.y.wrapping_add(1);
    cpu.set_zero_negative_flag(result);
    cpu.y = result;
    0
}

fn dey(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    let result = cpu.y.wrapping_sub(1);
    cpu.set_zero_negative_flag(result);
    cpu.y = result;
    0
}

//Shift Instructions
fn asl(cpu: &mut Cpu, mode: AddressingMode) -> u8 {
    if mode == AddressingMode::Accumulator {
        cpu.set_flag(StatusFlag::Carry, cpu.a & 0x80 != 0);
        let result = cpu.a << 1;
        cpu.set_zero_negative_flag(result);
        cpu.a = result;
        0
    }else{
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        let data = cpu.bus.read(addr);
        cpu.set_flag(StatusFlag::Carry, data & 0x80 != 0);
        let result = data << 1;
        cpu.set_zero_negative_flag(result);
        cpu.bus.write(addr, result);
        cycles
    }
}

fn lsr(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    if mode == AddressingMode::Accumulator {
        cpu.set_flag(StatusFlag::Carry, cpu.a & 0x1u8 != 0);
        let result = cpu.a >> 1;
        cpu.set_zero_negative_flag(result);
        cpu.a = result;
        0
    }else{
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        let data = cpu.bus.read(addr);
        cpu.set_flag(StatusFlag::Carry, data & 0x1u8 != 0);
        let result = data >> 1;
        cpu.bus.write(addr, result);
        cpu.set_zero_negative_flag(result);
        cycles
    }
}

fn rol(cpu: &mut Cpu, mode: AddressingMode) -> u8 {
    let mut total_cycles: u8 = 0;
    
    if mode == AddressingMode::Accumulator {
        let data = cpu.a;
        // Store old carry flag
        let old_carry = if cpu.p & 0x1u8 != 0 { 1 } else { 0 };
        // Set new carry flag from bit 7
        cpu.set_flag(StatusFlag::Carry, data & 0x80 != 0);
        // Perform logical left shift and add old carry to bit 0
        let result = (data << 1) | old_carry;
        
        cpu.set_flag(StatusFlag::Zero, result == 0);
        cpu.set_flag(StatusFlag::Negative, result & 0x80 != 0);
        cpu.a = result;
    } else {
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        total_cycles += cycles;
        let data = cpu.bus.read(addr);
        
        // Store old carry flag
        let old_carry = if cpu.p & 0x1u8 != 0 { 1 } else { 0 };
        // Set new carry flag from bit 7
        cpu.set_flag(StatusFlag::Carry, data & 0x80 != 0);
        // Perform logical left shift and add old carry to bit 0
        let result = (data << 1) | old_carry;
        
        cpu.set_flag(StatusFlag::Zero, result == 0);
        cpu.set_flag(StatusFlag::Negative, result & 0x80 != 0);
        cpu.bus.write(addr, result);
    }
    
    total_cycles
}

fn ror(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let mut total_cycles: u8 = 0;
    if mode == AddressingMode::Accumulator {
        let data = cpu.a;
        let old_carry: u8 = if cpu.p & 0x1u8 != 0 { 0x80 } else { 0 };
        cpu.set_flag(StatusFlag::Carry, data & 0x1u8 != 0);
        let result = (data >> 1) | old_carry;
        cpu.set_flag(StatusFlag::Zero, result == 0);
        cpu.set_flag(StatusFlag::Negative, result & 0x80 != 0);
        cpu.a = result;
    }else{
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        total_cycles += cycles;
        let data = cpu.bus.read(addr);
        let old_carry: u8 = if cpu.p & 0x1u8 != 0 { 0x80 } else { 0 };
        cpu.set_flag(StatusFlag::Carry, data & 0x1u8 != 0);
        let result = (data >> 1) | old_carry;
        cpu.set_flag(StatusFlag::Zero, result == 0);
        cpu.set_flag(StatusFlag::Negative, result & 0x80 != 0);
        cpu.bus.write(addr, result);
    }
    total_cycles
}

//Bitwise Instructions
fn and(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (data, cycles) = if mode == AddressingMode::Immediate {
        (cpu.fetch_operand(), 0)
    }else {
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        (cpu.bus.read(addr), cycles)
    };
    let result = cpu.a & data;
    cpu.set_zero_negative_flag(result);
    cpu.a = result;
    cycles
}

fn ora(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let mut total_cycles: u8 = 0;
    let data = if mode == AddressingMode::Immediate {
        cpu.fetch_operand()
    }else{
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        total_cycles += cycles;
        cpu.bus.read(addr)
    };

    let result = cpu.a | data;
    cpu.set_flag(StatusFlag::Zero, result == 0);
    cpu.set_flag(StatusFlag::Negative, result & 0x80u8 != 0);
    cpu.a = result;
    total_cycles
}

fn eor(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let mut total_cycles: u8 = 0;
    let data = if mode == AddressingMode::Immediate {
        cpu.fetch_operand()
    }else{
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        total_cycles += cycles;
        cpu.bus.read(addr)
    };
    let result = cpu.a ^ data;
    cpu.set_flag(StatusFlag::Zero, result == 0);
    cpu.set_flag(StatusFlag::Negative, result & 0x80 != 0);
    cpu.a = result;
    total_cycles

}

fn bit(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let data = cpu.bus.read(addr);
    let result = cpu.a & data;
    cpu.set_flag(StatusFlag::Zero, result == 0);
    cpu.set_flag(StatusFlag::Overflow, data & 0x40u8 != 0);
    cpu.set_flag(StatusFlag::Negative, data & 0x80u8 != 0);
    cycles
}

//Compare Instructions
fn cmp(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let mut total_cycles = 0;
    let data = if mode == AddressingMode::Immediate {
        cpu.fetch_operand()
    }else{
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        total_cycles += cycles;
        cpu.bus.read(addr)
    };
    let result = (cpu.a as i16 - data as i16) as u8;
    cpu.set_flag(StatusFlag::Carry, cpu.a >= data);
    cpu.set_flag(StatusFlag::Zero, cpu.a == data);
    cpu.set_flag(StatusFlag::Negative, result & 0x80 != 0);
    total_cycles

}

fn cpx(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let mut total_cycles = 0;
    let data = if mode == AddressingMode::Immediate {
        cpu.fetch_operand()
    }else{
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        total_cycles += cycles;
        cpu.bus.read(addr)
    };
    let result = cpu.x.wrapping_sub(data);
    cpu.set_flag(StatusFlag::Carry, cpu.x >= data);
    cpu.set_flag(StatusFlag::Zero, cpu.x == data);
    cpu.set_flag(StatusFlag::Negative, result & 0x80 != 0);
    total_cycles
}

fn cpy(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let mut total_cycles = 0;
    let data = if mode == AddressingMode::Immediate {
        cpu.fetch_operand()
    }else{
        let (addr, cycles) = cpu.fetch_operand_addr(mode);
        total_cycles += cycles;
        cpu.bus.read(addr)
    };
    let result = cpu.y.wrapping_sub(data);
    cpu.set_flag(StatusFlag::Carry, cpu.y >= data);
    cpu.set_flag(StatusFlag::Zero, cpu.y == data);
    cpu.set_flag(StatusFlag::Negative, result & 0x80 != 0);
    total_cycles
}

//Branch Instructions
pub fn branch(cpu: &mut Cpu, mode: AddressingMode, condition: bool) -> u8 {
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let mut cycles_took = cycles;
    if condition {
        cpu.pc = addr;
        cycles_took += 1;
    }
    cycles_took
}

fn bcc(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let carry = cpu.p & 0x1u8 == 0;
    branch(cpu, mode, carry)
}

fn bcs(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let carry = cpu.p & 0x1u8 != 0;
    branch(cpu, mode, carry)
}

fn beq(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let zero = cpu.p & 0x2u8 != 0;
    branch(cpu, mode, zero)
}

fn bne(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let zero = cpu.p & 0x2u8 == 0;
    branch(cpu, mode, zero)
}

fn bpl(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let negative = cpu.p & 0x80 == 0;
    branch(cpu, mode, negative)
}

fn bmi(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let negative = cpu.p & 0x80 != 0;
    branch(cpu, mode, negative)
}

fn bvc(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let overflow = cpu.p & 0x40 == 0;
    branch(cpu, mode, overflow)
}

fn bvs(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let overflow = cpu.p & 0x40 != 0;
    branch(cpu, mode, overflow)
}

//Jump Instructions
fn jmp(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    cpu.pc = addr;
    cycles
}

fn jsr(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (target_address, cycles) = cpu.fetch_operand_addr(mode);
    
    let return_addr = cpu.pc.wrapping_sub(1);
    cpu.stack_push((return_addr >> 8) as u8);
    cpu.stack_push(return_addr as u8);

    cpu.pc = target_address;
    cycles
}

fn rts(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    let lo = cpu.stack_pop() as u16; 
    let hi = cpu.stack_pop() as u16;

    let return_address = (hi << 8) | lo;
    cpu.pc = return_address.wrapping_add(1);
    0
}

fn brk(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.interrupt(Interrupt::BRK);
    0
}

fn rti(cpu: &mut Cpu, _mode: AddressingMode) -> u8{

    cpu.p = cpu.stack_pop();
    cpu.set_flag(StatusFlag::Break, false);
    let lo = cpu.stack_pop() as u16;
    let hi = cpu.stack_pop() as u16;
    cpu.pc = (hi << 8) | lo;
    0
}

//Stack Instructions
fn pha(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    let addr = 0x0100 + cpu.sp as u16;
    cpu.bus.write(addr, cpu.a);
    cpu.sp = cpu.sp.wrapping_sub(1);
    0
}

fn pla(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.sp = cpu.sp.wrapping_add(1);
    let addr = 0x0100 + cpu.sp as u16;
    let data = cpu.bus.read(addr);
    cpu.set_flag(StatusFlag::Zero, data == 0);
    cpu.set_flag(StatusFlag::Negative, data & 0x80 != 0);
    cpu.a = data;
    0
}

fn php(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    let addr = 0x0100 + cpu.sp as u16;
    let value = cpu.p | 0x30u8;
    cpu.bus.write(addr, value);
    cpu.sp = cpu.sp.wrapping_sub(1);
    0
}

fn plp(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.sp = cpu.sp.wrapping_add(1);
    let addr = 0x0100 + cpu.sp as u16;
    let data = cpu.bus.read(addr);

    cpu.set_flag(StatusFlag::Carry, data & 0x1u8 != 0);
    cpu.set_flag(StatusFlag::Zero, data & 0x2u8 != 0);
    cpu.update_interrupt_disable = (true, data & 0x4u8);
    cpu.set_flag(StatusFlag::Break, false);
    cpu.set_flag(StatusFlag::Decimal, data & 0x8u8 != 0);
    cpu.set_flag(StatusFlag::Overflow, data & 0x40u8 != 0);
    cpu.set_flag(StatusFlag::Negative, data & 0x80u8 != 0);
    0
}

fn txs(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.sp = cpu.x;
    0
}

fn tsx(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.x = cpu.sp;
    cpu.set_flag(StatusFlag::Zero, cpu.sp == 0);
    cpu.set_flag(StatusFlag::Negative, cpu.sp & 0x80u8 != 0);
    0
}

//Flag Instructions
fn clc(cpu: &mut Cpu, _mode: AddressingMode) -> u8{ 
    cpu.set_flag(StatusFlag::Carry, false);
    0
}

fn sec(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.set_flag(StatusFlag::Carry, true);
    0
}

fn cli(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.set_flag(StatusFlag::InterruptDisable, false);
    0
}

fn sei(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.update_interrupt_disable = (true, 1);
    0
}

fn cld(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.set_flag(StatusFlag::Decimal, false);
    0
}

fn sed(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.set_flag(StatusFlag::Decimal, true);
    0
}

fn clv(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    cpu.set_flag(StatusFlag::Overflow, false);
    0
}

// // Unofficial Opcodes
fn nop(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    if mode == AddressingMode::Implied {
        0
    }else if mode == AddressingMode::Immediate {
        cpu.fetch_operand();
        0
    }else{
        let (_addr, cycles) = cpu.fetch_operand_addr(mode);
        return cycles
    }
}

fn jam(_cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    0
    //panic!("CPU halted due to JAM instruction at PC: {:X}", cpu.pc);
}

fn slo(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let mut data = cpu.bus.read(addr);
    cpu.set_flag(StatusFlag::Carry, data & 0x80u8 != 0);
    data <<= 1;
    cpu.bus.write(addr, data);
    cpu.a |= data;
    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, cpu.a & 0x80u8 != 0);
    cycles
}

fn ane(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    //Unstable, recommended to use operand 0.
    cpu.a = 0;
    cpu.set_flag(StatusFlag::Zero, true);
    cpu.set_flag(StatusFlag::Negative, false);
    0
}

fn anc(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    let operand = cpu.fetch_operand();
    cpu.a &= operand;
    cpu.set_flag(StatusFlag::Carry, cpu.a & 0x80u8 != 0);
    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, cpu.a & 0x80 != 0);
    0
}

fn sre(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let mut data = cpu.bus.read(addr);
    cpu.set_flag(StatusFlag::Carry, data & 0x01 != 0);
    data >>= 1;
    cpu.bus.write(addr, data);
    cpu.a ^= data;
    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, cpu.a & 0x80 != 0);
    cycles
}

fn rla(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let mut data = cpu.bus.read(addr);
    let carry_in = cpu.p & 0x1u8;
    cpu.set_flag(StatusFlag::Carry, data & 0x80 != 0);
    data = (data << 1) | carry_in;

    cpu.bus.write(addr, data);
    cpu.a &= data;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, cpu.a & 0x80 != 0);
    cycles
}

fn sax(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    cpu.bus.write(addr, cpu.a & cpu.x);
    cycles
}

fn rra(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, extra_cycles) = cpu.fetch_operand_addr(mode);
    
    // First do ROR
    let mut data = cpu.bus.read(addr);
    let old_carry = cpu.get_carry_bit();
    
    // Set new carry from bit 0
    cpu.set_flag(StatusFlag::Carry, data & 0x01 != 0);
    
    // Rotate right, putting old carry in bit 7
    data = (data >> 1) | (old_carry << 7);
    cpu.bus.write(addr, data);
    
    // Then do ADC
    let carry_in = cpu.get_carry_bit();
    let temp = cpu.a as u16 + data as u16 + carry_in as u16;
    
    // Set carry from ADC
    cpu.set_flag(StatusFlag::Carry, temp > 0xFF);
    
    // Set overflow
    let result = temp as u8;
    cpu.set_flag(StatusFlag::Overflow,
        ((cpu.a ^ result) & (data ^ result) & 0x80) != 0);
    
    cpu.a = result;
    cpu.set_zero_negative_flag(result);
    extra_cycles
}

fn dcp(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let mut data = cpu.bus.read(addr);
    data = data.wrapping_sub(1);
    cpu.bus.write(addr, data);
    let result = cpu.a.wrapping_sub(data);
    cpu.set_flag(StatusFlag::Carry, cpu.a >= data);
    cpu.set_flag(StatusFlag::Zero, result == 0);
    cpu.set_flag(StatusFlag::Negative, result & 0x80u8 != 0);
    cycles
}

fn isc(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, extra_cycles) = cpu.fetch_operand_addr(mode);
    
    // First increment memory
    let mut data = cpu.bus.read(addr);
    data = data.wrapping_add(1);
    cpu.bus.write(addr, data);
    
    // Then do SBC
    let carry = cpu.get_carry_bit();
    let value = !data;  // Invert the bits for subtraction
    
    let temp = cpu.a as u16 + value as u16 + carry as u16;
    
    // Set carry (note: carry flag is inverted for SBC)
    cpu.set_flag(StatusFlag::Carry, temp > 0xFF);
    
    let result = temp as u8;
    
    // Set overflow
    let overflow = ((cpu.a ^ result) & (cpu.a ^ data) & 0x80) != 0;
    cpu.set_flag(StatusFlag::Overflow, overflow);
    
    cpu.a = result;
    cpu.set_zero_negative_flag(result);

    extra_cycles
}

fn lxa(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    
    let mut total_cycles = lda(cpu, mode);
    total_cycles += tax(cpu, mode);
    total_cycles
}

fn las(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let data = cpu.bus.read(addr);
    let result = data & cpu.sp;

    cpu.a = result;
    cpu.x = result;
    cpu.sp = result;
    cpu.set_flag(StatusFlag::Zero, result == 0);
    cpu.set_flag(StatusFlag::Negative, result & 0x80 != 0);
    cycles
}

fn lax(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let data = cpu.bus.read(addr);
    cpu.a = data;
    cpu.x = cpu.a;
    cpu.set_zero_negative_flag(cpu.x);
    cycles
}

fn sbx(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    let operand = cpu.fetch_operand();
    let temp = cpu.a & cpu.x;
    let result = (temp).wrapping_sub(operand);
    cpu.x = result;

    cpu.set_flag(StatusFlag::Zero, result == 0);
    cpu.set_flag(StatusFlag::Negative, result & 0x80 != 0);
    cpu.set_flag(StatusFlag::Carry, temp >= operand);
    0
}

fn sha(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let value = cpu.a & cpu.x & ((addr >> 8) as u8 + 1);
    cpu.bus.write(addr, value);
    cycles
}

fn shx(cpu: &mut Cpu, mode: AddressingMode) -> u8 {
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let high_byte = (addr >> 8) as u8;
    let value = cpu.x & (high_byte.wrapping_add(1));
    
    // Calculate actual address, which can be affected by page crossing
    let effective_addr = if (addr & 0xFF) + (cpu.y as u16) > 0xFF {
        // Page boundary crossed - high byte becomes unstable
        (value as u16) << 8 | (addr & 0xFF) + (cpu.y as u16)
    } else {
        // No page boundary crossing
        addr
    };
    
    cpu.bus.write(effective_addr, value);
    cycles
}

fn shy(cpu: &mut Cpu, mode: AddressingMode) -> u8 {
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    let high_byte = (addr >> 8) as u8;
    let value = cpu.y & (high_byte.wrapping_add(1));
    
    // Calculate actual address, which can be affected by page crossing
    let effective_addr = if (addr & 0xFF) + (cpu.x as u16) > 0xFF {
        // Page boundary crossed - high byte becomes unstable
        (value as u16) << 8 | (addr & 0xFF) + (cpu.x as u16)
    } else {
        // No page boundary crossing
        addr
    };
    
    cpu.bus.write(effective_addr, value);
    cycles
}

fn tas(cpu: &mut Cpu, mode: AddressingMode) -> u8{
    let (addr, cycles) = cpu.fetch_operand_addr(mode);
    cpu.sp = cpu.a & cpu.x;
    let value = cpu.sp & ((addr >> 8) as u8 + 1);
    cpu.bus.write(addr, value);
    cycles
}

fn arr(cpu: &mut Cpu, _mode: AddressingMode) -> u8 {
    // Step 1: AND the accumulator with the operand
    let operand = cpu.fetch_operand();
    cpu.a &= operand;

    // Step 2: Rotate right, using old carry bit
    let old_carry = cpu.get_carry_bit();
    let rotated = (cpu.a >> 1) | if old_carry != 0 { 0x80 } else { 0x00 };
    cpu.a = rotated;

    // Step 3: Set carry flag based on bit 6 of result
    cpu.set_flag(StatusFlag::Carry, (cpu.a & 0x40) != 0);

    // Step 4: Set overflow flag based on XOR of bits 6 and 5
    cpu.set_flag(StatusFlag::Overflow, ((cpu.a >> 6) ^ (cpu.a >> 5)) & 0x01 != 0);

    // Step 5: Set zero and negative flags
    cpu.set_zero_negative_flag(cpu.a);

    0  // cycles
}

fn alr(cpu: &mut Cpu, _mode: AddressingMode) -> u8{
    let operand = cpu.fetch_operand();
    cpu.a &= operand;
    cpu.set_flag(StatusFlag::Carry, (cpu.a & 0x1) != 0);
    cpu.a >>= 1;
    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a & 0x80) != 0);
    0
}
#[derive(Debug)]
pub enum Instruction {
    ADC,
    AND,
    BIT,
    SBC,
    LDX,
    LDY,
    STX,
    STY,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    JMP,
    NOP,
    LDA,
    STA,
    JSR,
    RTS,
    CMP,
    BCS,
    BCC,
    BEQ,
    BNE,
    INC,
    INX,
    INY,
    TXS,
    TXA,
    TYA,
    TSX,
    CLC,
    CLD,
    CLI,
    CLV,
    SEC,
    SED,
    SEI,
    PHA,
    PLA,
    TAX,
    TAY,
    LSR,
    ROR,
    PHP,
    ASL,
    ROL,
}

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Accumulator,
    Implied,
    Immediate,
    Absolute,
    AbsoluteXIndexed,
    AbsoluteYIndexed,
    ZeroPage,
    Relative,
    AbsoluteIndirect,
    AbsoluteIndirectX,
    AbsoluteIndirectY,
    ZeroPageX,
    ZeroPageY,
    ZeroPageIndirectIndexedX,
    ZeroPageIndirectIndexedY,
}

fn get_addressing_mode_operand_length(mode: AddressingMode) -> u8 {
    match mode {
        AddressingMode::Accumulator => 0,
        AddressingMode::Implied => 0,
        AddressingMode::Immediate => 1,
        AddressingMode::Absolute => 2,
        AddressingMode::AbsoluteXIndexed => 2,
        AddressingMode::AbsoluteYIndexed => 2,
        AddressingMode::ZeroPage => 1,
        AddressingMode::Relative => 1,
        AddressingMode::AbsoluteIndirect => 2,
        AddressingMode::AbsoluteIndirectX => 2,
        AddressingMode::AbsoluteIndirectY => 2,
        AddressingMode::ZeroPageX => 1,
        AddressingMode::ZeroPageY => 1,
        AddressingMode::ZeroPageIndirectIndexedX => 1,
        AddressingMode::ZeroPageIndirectIndexedY => 1,
    }
}

fn get_instruction_length(mode: AddressingMode) -> u8 {
    1 + get_addressing_mode_operand_length(mode)
}

pub struct InstructionMetadata {
    pub mode: AddressingMode,
    pub instruction_type: Instruction,
    pub instruction_byte_length: u8,
}

impl InstructionMetadata {
    fn new(mode: AddressingMode, instruction: Instruction) -> InstructionMetadata {
        InstructionMetadata {
            mode,
            instruction_type: instruction,
            instruction_byte_length: get_instruction_length(mode),
        }
    }
}

pub fn get_opcode_metadata(opcode: u8) -> InstructionMetadata {
    match opcode {
        // ADC
        0x69 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::ADC),
        0x6d => InstructionMetadata::new(AddressingMode::Absolute, Instruction::ADC),
        0x7d => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::ADC),
        0x79 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::ADC),
        0x65 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::ADC),
        0x75 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::ADC),
        0x61 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedX, Instruction::ADC)
        }
        0x71 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedY, Instruction::ADC)
        }

        // AND
        0x29 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::AND),
        0x2d => InstructionMetadata::new(AddressingMode::Absolute, Instruction::AND),
        0x3d => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::AND),
        0x39 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::AND),
        0x25 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::AND),
        0x35 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::AND),
        0x21 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedX, Instruction::AND)
        }
        0x31 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedY, Instruction::AND)
        }

        // BIT
        0x2c => InstructionMetadata::new(AddressingMode::Absolute, Instruction::BIT),
        0x24 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::BIT),

        // SBC
        0xe9 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::SBC),
        0xed => InstructionMetadata::new(AddressingMode::Absolute, Instruction::SBC),
        0xfd => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::SBC),
        0xf9 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::SBC),
        0xe5 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::SBC),
        0xf5 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::SBC),
        0xe1 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedX, Instruction::ADC)
        }
        0xf1 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedY, Instruction::ADC)
        }

        // BCC
        0x90 => InstructionMetadata::new(AddressingMode::Relative, Instruction::BCC),

        // LDX
        0xa2 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::LDX),
        0xae => InstructionMetadata::new(AddressingMode::Absolute, Instruction::LDX),
        0xbe => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::LDX),
        0xa6 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::LDX),
        0xb6 => InstructionMetadata::new(AddressingMode::ZeroPageY, Instruction::LDX),

        // LDY
        0xa0 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::LDY),
        0xac => InstructionMetadata::new(AddressingMode::Absolute, Instruction::LDY),
        0xbc => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::LDY),
        0xa4 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::LDY),
        0xb4 => InstructionMetadata::new(AddressingMode::ZeroPageY, Instruction::LDY),

        // STX
        0x8e => InstructionMetadata::new(AddressingMode::Absolute, Instruction::STX),
        0x86 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::STX),
        0x96 => InstructionMetadata::new(AddressingMode::ZeroPageY, Instruction::STX),

        // STY
        0x8c => InstructionMetadata::new(AddressingMode::Absolute, Instruction::STY),
        0x84 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::STY),
        0x94 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::STY),

        // CPX
        0xe0 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::CPX),
        0xec => InstructionMetadata::new(AddressingMode::Absolute, Instruction::CPX),
        0xe4 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::CPX),

        // CPY
        0xc0 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::CPY),
        0xcc => InstructionMetadata::new(AddressingMode::Absolute, Instruction::CPY),
        0xc4 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::CPY),

        // DEC
        0xce => InstructionMetadata::new(AddressingMode::Absolute, Instruction::DEC),
        0xde => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::DEC),
        0xc6 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::DEC),
        0xd6 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::DEC),

        // DEX
        0xca => InstructionMetadata::new(AddressingMode::Implied, Instruction::DEX),

        // DEY
        0x88 => InstructionMetadata::new(AddressingMode::Implied, Instruction::DEY),

        // JMP
        0x4c => InstructionMetadata::new(AddressingMode::Absolute, Instruction::JMP),
        0x6c => InstructionMetadata::new(AddressingMode::AbsoluteIndirect, Instruction::JMP),

        // NOP
        0xea => InstructionMetadata::new(AddressingMode::Implied, Instruction::NOP),

        // LDA
        0xa9 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::LDA),
        0xad => InstructionMetadata::new(AddressingMode::Absolute, Instruction::LDA),
        0xbd => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::LDA),
        0xb9 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::LDA),
        0xa5 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::LDA),
        0xb5 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::LDA),
        0xa1 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedX, Instruction::LDA)
        }
        0xb1 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedY, Instruction::LDA)
        }

        // STA
        0x8d => InstructionMetadata::new(AddressingMode::Absolute, Instruction::STA),
        0x9d => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::STA),
        0x99 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::STA),
        0x85 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::STA),
        0x95 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::STA),
        0x81 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedX, Instruction::STA)
        }
        0x91 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedY, Instruction::STA)
        }

        // JSR
        0x20 => InstructionMetadata::new(AddressingMode::Absolute, Instruction::JSR),

        // RTS
        0x60 => InstructionMetadata::new(AddressingMode::Implied, Instruction::RTS),

        // CMP
        0xc9 => InstructionMetadata::new(AddressingMode::Immediate, Instruction::CMP),
        0xcd => InstructionMetadata::new(AddressingMode::Absolute, Instruction::CMP),
        0xdd => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::CMP),
        0xd9 => InstructionMetadata::new(AddressingMode::AbsoluteYIndexed, Instruction::CMP),
        0xc5 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::CMP),
        0xd5 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::CMP),
        0xc1 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedX, Instruction::CMP)
        }
        0xd1 => {
            InstructionMetadata::new(AddressingMode::ZeroPageIndirectIndexedY, Instruction::CMP)
        }

        // BCS
        0xb0 => InstructionMetadata::new(AddressingMode::Relative, Instruction::BCS),

        // BEQ
        0xf0 => InstructionMetadata::new(AddressingMode::Relative, Instruction::BEQ),

        // BNE
        0xd0 => InstructionMetadata::new(AddressingMode::Relative, Instruction::BNE),

        // INC
        0xee => InstructionMetadata::new(AddressingMode::Absolute, Instruction::INC),
        0xfe => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::INC),
        0xe6 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::INC),
        0xf6 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::INC),

        // INX
        0xe8 => InstructionMetadata::new(AddressingMode::Implied, Instruction::INX),

        // INY
        0xc8 => InstructionMetadata::new(AddressingMode::Implied, Instruction::INY),

        // TXS
        0x9a => InstructionMetadata::new(AddressingMode::Implied, Instruction::TXS),

        // TXA
        0x8a => InstructionMetadata::new(AddressingMode::Implied, Instruction::TXA),

        // TSX
        0xba => InstructionMetadata::new(AddressingMode::Implied, Instruction::TSX),

        // TYA
        0x98 => InstructionMetadata::new(AddressingMode::Implied, Instruction::TYA),

        // CLC
        0x18 => InstructionMetadata::new(AddressingMode::Implied, Instruction::CLC),

        // CLD
        0xd8 => InstructionMetadata::new(AddressingMode::Implied, Instruction::CLD),

        // CLI
        0x58 => InstructionMetadata::new(AddressingMode::Implied, Instruction::CLI),

        // CLV
        0xb8 => InstructionMetadata::new(AddressingMode::Implied, Instruction::CLV),

        // SEC
        0x38 => InstructionMetadata::new(AddressingMode::Implied, Instruction::SEC),

        // SED
        0xf8 => InstructionMetadata::new(AddressingMode::Implied, Instruction::SED),

        // SEI
        0x78 => InstructionMetadata::new(AddressingMode::Implied, Instruction::SEI),

        // PHA
        0x48 => InstructionMetadata::new(AddressingMode::Implied, Instruction::PHA),

        // PLA
        0x68 => InstructionMetadata::new(AddressingMode::Implied, Instruction::PLA),

        // TAX
        0xaa => InstructionMetadata::new(AddressingMode::Implied, Instruction::TAX),

        // TAY
        0xa8 => InstructionMetadata::new(AddressingMode::Implied, Instruction::TAY),

        // LSR
        0x4a => InstructionMetadata::new(AddressingMode::Accumulator, Instruction::LSR),
        0x4e => InstructionMetadata::new(AddressingMode::Absolute, Instruction::LSR),
        0x5e => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::LSR),
        0x46 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::LSR),
        0x56 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::LSR),

        // ROR
        0x6a => InstructionMetadata::new(AddressingMode::Accumulator, Instruction::ROR),
        0x6e => InstructionMetadata::new(AddressingMode::Absolute, Instruction::ROR),
        0x7e => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::ROR),
        0x66 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::ROR),
        0x76 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::ROR),

        // ROL
        0x2a => InstructionMetadata::new(AddressingMode::Accumulator, Instruction::ROL),
        0x2e => InstructionMetadata::new(AddressingMode::Absolute, Instruction::ROL),
        0x3e => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::ROL),
        0x26 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::ROL),
        0x36 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::ROL),

        // ASL
        0x0a => InstructionMetadata::new(AddressingMode::Accumulator, Instruction::ASL),
        0x0e => InstructionMetadata::new(AddressingMode::Absolute, Instruction::ASL),
        0x1e => InstructionMetadata::new(AddressingMode::AbsoluteXIndexed, Instruction::ASL),
        0x06 => InstructionMetadata::new(AddressingMode::ZeroPage, Instruction::ASL),
        0x16 => InstructionMetadata::new(AddressingMode::ZeroPageX, Instruction::ASL),

        // PHP
        0x08 => InstructionMetadata::new(AddressingMode::Implied, Instruction::PHP),
        _ => todo!("Missing instruction metadata for opcode 0x{:#>02x}", opcode),
    }
}

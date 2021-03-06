use crate::common::{Clocked, Addressable, join_bytes};
use crate::memory::{CpuMem};

mod opcodes {
    #[derive(Debug)]
    pub enum Operation {
        ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI,
        BNE, BPL, BRK, BVC, BVS, CLC, CLD, CLI,
        CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR,
        INC, INX, INY, JMP, JSR, LDA, LDX, LDY,
        LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL,
        ROR, RTI, RTS, SBC, SEC, SED, SEI, STA,
        STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,

        // Unofficial
        KIL, ISC, DCP, AXS, LAS, LAX, AHX, SAX,
        XAA, SHX, RRA, TAS, SHY, ARR, SRE, ALR,
        RLA, ANC, SLO,
    }

    #[derive(Debug, PartialEq)]
    pub enum AddressMode {
        Implicit,
        Accumulator,
        Immediate,
        ZeroPage,
        ZeroPageX,
        ZeroPageY,
        Relative,
        Absolute,
        AbsoluteX,
        AbsoluteY,
        Indirect,
        IndirectX,
        IndirectY
    }

    impl AddressMode {
        pub fn byte_count(&self) -> u16 {
            match self {
                // no arg
                Accumulator | Implicit => 1,

                // 1 byte arg
                ZeroPage | ZeroPageX | ZeroPageY | Relative => 2,
                Immediate | Indirect | IndirectX | IndirectY => 2,

                // 2 byte arg
                Absolute | AbsoluteX | AbsoluteY => 3,
            }
        }
    }

    use Operation::*;
    use AddressMode::*;

    type Cycles = u8;
    type AddIfPageBoundaryCrossed = bool;

    pub type Opcode = (Operation, AddressMode, Cycles, AddIfPageBoundaryCrossed);

    // http://www.oxyron.de/html/opcodes02.html
    const TABLE: [Opcode; 256] = [
        // 0x
        (BRK, Implicit, 7, false),
        (ORA, IndirectX, 6, false),
        (KIL, Implicit, 0, false),
        (SLO, IndirectX, 8, false),
        (NOP, ZeroPage, 3, false),
        (ORA, ZeroPage, 3, false),
        (ASL, ZeroPage, 5, false),
        (SLO, ZeroPage, 5, false),
        (PHP, Implicit, 3, false),
        (ORA, Immediate, 2, false),
        (ASL, Accumulator, 2, false),
        (ANC, Immediate, 2, false),
        (NOP, Absolute, 4, false),
        (ORA, Absolute, 4, false),
        (ASL, Absolute, 6, false),
        (SLO, Absolute, 6, false),

        // 1x
        (BPL, Relative, 2, true),
        (ORA, IndirectY, 5, true),
        (KIL, Implicit, 0, false),
        (SLO, IndirectY, 8, false),
        (NOP, ZeroPageX, 4, false),
        (ORA, ZeroPageX, 4, false),
        (ASL, ZeroPageX, 6, false),
        (SLO, ZeroPageX, 6, false),
        (CLC, Implicit, 2, false),
        (ORA, AbsoluteY, 4, true),
        (NOP, Implicit, 2, false),
        (SLO, AbsoluteY, 7, false),
        (NOP, AbsoluteX, 4, true),
        (ORA, AbsoluteX, 4, true),
        (ASL, AbsoluteX, 7, false),
        (SLO, AbsoluteX, 7, false),

        // 2x
        (JSR, Absolute, 6, false),
        (AND, IndirectX, 6, false),
        (KIL, Implicit, 0, false),
        (RLA, IndirectX, 8, false),
        (BIT, ZeroPage, 3, false),
        (AND, ZeroPage, 3, false),
        (ROL, ZeroPage, 5, false),
        (RLA, ZeroPage, 5, false),
        (PLP, Implicit, 4, false),
        (AND, Immediate, 2, false),
        (ROL, Accumulator, 2, false),
        (ANC, Immediate, 2, false),
        (BIT, Absolute, 4, false),
        (AND, Absolute, 4, false),
        (ROL, Absolute, 6, false),
        (RLA, Absolute, 6, false),

        // 3x
        (BMI, Relative, 2, true),
        (AND, IndirectY, 5, true),
        (KIL, Implicit, 0, false),
        (RLA, IndirectY, 8, false),
        (NOP, ZeroPageX, 4, false),
        (AND, ZeroPageX, 4, false),
        (ROL, ZeroPageX, 6, false),
        (RLA, ZeroPageX, 6, false),
        (SEC, Implicit, 2, false),
        (AND, AbsoluteY, 4, true),
        (NOP, Implicit, 2, false),
        (RLA, AbsoluteY, 7, false),
        (NOP, AbsoluteX, 4, true),
        (AND, AbsoluteX, 4, true),
        (ROL, AbsoluteX, 7, false),
        (RLA, AbsoluteX, 7, false),

        // 4x
        (RTI, Implicit, 6, false),
        (EOR, IndirectX, 6, false),
        (KIL, Implicit, 0, false),
        (SRE, IndirectX, 8, false),
        (NOP, ZeroPage, 3, false),
        (EOR, ZeroPage, 3, false),
        (LSR, ZeroPage, 5, false),
        (SRE, ZeroPage, 5, false),
        (PHA, Implicit, 3, false),
        (EOR, Immediate, 2, false),
        (LSR, Accumulator, 2, false),
        (ALR, Immediate, 2, false),
        (JMP, Absolute, 3, false),
        (EOR, Absolute, 4, false),
        (LSR, Absolute, 6, false),
        (SRE, Absolute, 6, false),

        // 5x
        (BVC, Relative, 2, true),
        (EOR, IndirectY, 5, true),
        (KIL, Implicit, 0, false),
        (SRE, IndirectY, 8, false),
        (NOP, ZeroPageX, 4, false),
        (EOR, ZeroPageX, 4, false),
        (LSR, ZeroPageX, 6, false),
        (SRE, ZeroPageX, 6, false),
        (CLI, Implicit, 2, false),
        (EOR, AbsoluteY, 4, true),
        (NOP, Implicit, 2, false),
        (SRE, AbsoluteY, 7, false),
        (NOP, AbsoluteX, 4, true),
        (EOR, AbsoluteX, 4, true),
        (LSR, AbsoluteX, 7, false),
        (SRE, AbsoluteX, 7, false),

        // 6x
        (RTS, Implicit, 6, false),
        (ADC, IndirectX, 6, false),
        (KIL, Implicit, 0, false),
        (RRA, IndirectX, 8, false),
        (NOP, ZeroPage, 3, false),
        (ADC, ZeroPage, 3, false),
        (ROR, ZeroPage, 5, false),
        (RRA, ZeroPage, 5, false),
        (PLA, Implicit, 4, false),
        (ADC, Immediate, 2, false),
        (ROR, Accumulator, 2, false),
        (ARR, Immediate, 2, false),
        (JMP, Indirect, 5, false),
        (ADC, Absolute, 4, false),
        (ROR, Absolute, 6, false),
        (RRA, Absolute, 6, false),

        // 7x
        (BVS, Relative, 2, true),
        (ADC, IndirectY, 5, true),
        (KIL, Implicit, 0, false),
        (RRA, IndirectY, 8, false),
        (NOP, ZeroPageX, 4, false),
        (ADC, ZeroPageX, 4, false),
        (ROR, ZeroPageX, 6, false),
        (RRA, ZeroPageX, 6, false),
        (SEI, Implicit, 2, false),
        (ADC, AbsoluteY, 4, true),
        (NOP, Implicit, 2, false),
        (RRA, AbsoluteY, 7, false),
        (NOP, AbsoluteX, 4, true),
        (ADC, AbsoluteX, 4, true),
        (ROR, AbsoluteX, 7, false),
        (RRA, AbsoluteX, 7, false),

        // 8x
        (NOP, Immediate, 2, false),
        (STA, IndirectX, 6, false),
        (NOP, Immediate, 2, false),
        (SAX, IndirectX, 6, false),
        (STY, ZeroPage, 3, false),
        (STA, ZeroPage, 3, false),
        (STX, ZeroPage, 3, false),
        (SAX, ZeroPage, 3, false),
        (DEY, Implicit, 2, false),
        (NOP, Immediate, 2, false),
        (TXA, Implicit, 2, false),
        (XAA, Immediate, 2, false),
        (STY, Absolute, 4, false),
        (STA, Absolute, 4, false),
        (STX, Absolute, 4, false),
        (SAX, Absolute, 4, false),

        // 9x
        (BCC, Relative, 2, true),
        (STA, IndirectY, 6, false),
        (KIL, Implicit, 0, false),
        (AHX, IndirectY, 6, false),
        (STY, ZeroPageX, 4, false),
        (STA, ZeroPageX, 4, false),
        (STX, ZeroPageY, 4, false),
        (SAX, ZeroPageY, 4, false),
        (TYA, Implicit, 2, false),
        (STA, AbsoluteY, 5, false),
        (TXS, Implicit, 2, false),
        (TAS, AbsoluteY, 5, false),
        (SHY, AbsoluteX, 5, false),
        (STA, AbsoluteX, 5, false),
        (SHX, AbsoluteY, 5, false),
        (AHX, AbsoluteY, 5, false),

        // Ax
        (LDY, Immediate, 2, false),
        (LDA, IndirectX, 6, false),
        (LDX, Immediate, 2, false),
        (LAX, IndirectX, 6, false),
        (LDY, ZeroPage, 3, false),
        (LDA, ZeroPage, 3, false),
        (LDX, ZeroPage, 3, false),
        (LAX, ZeroPage, 3, false),
        (TAY, Implicit, 2, false),
        (LDA, Immediate, 2, false),
        (TAX, Implicit, 2, false),
        (LAX, Immediate, 2, false),
        (LDY, Absolute, 4, false),
        (LDA, Absolute, 4, false),
        (LDX, Absolute, 4, false),
        (LAX, Absolute, 4, false),

        // Bx
        (BCS, Relative, 2, true),
        (LDA, IndirectY, 5, true),
        (KIL, Implicit, 0, false),
        (LAX, IndirectY, 5, true),
        (LDY, ZeroPageX, 4, false),
        (LDA, ZeroPageX, 4, false),
        (LDX, ZeroPageY, 4, false),
        (LAX, ZeroPageY, 4, false),
        (CLV, Implicit, 2, false),
        (LDA, AbsoluteY, 4, true),
        (TSX, Implicit, 2, false),
        (LAS, AbsoluteY, 4, true),
        (LDY, AbsoluteX, 4, true),
        (LDA, AbsoluteX, 4, true),
        (LDX, AbsoluteY, 4, true),
        (LAX, AbsoluteY, 4, true),

        // Cx
        (CPY, Immediate, 2, false),
        (CMP, IndirectX, 6, false),
        (NOP, Immediate, 2, false),
        (DCP, IndirectX, 8, false),
        (CPY, ZeroPage, 3, false),
        (CMP, ZeroPage, 3, false),
        (DEC, ZeroPage, 5, false),
        (DCP, ZeroPage, 5, false),
        (INY, Implicit, 2, false),
        (CMP, Immediate, 2, false),
        (DEX, Implicit, 2, false),
        (AXS, Immediate, 2, false),
        (CPY, Absolute, 4, false),
        (CMP, Absolute, 4, false),
        (DEC, Absolute, 6, false),
        (DCP, Absolute, 6, false),

        // Dx
        (BNE, Relative, 2, true),
        (CMP, IndirectY, 5, true),
        (KIL, Implicit, 0, false),
        (DCP, IndirectY, 8, false),
        (NOP, ZeroPageX, 4, false),
        (CMP, ZeroPageX, 4, false),
        (DEC, ZeroPageX, 6, false),
        (DCP, ZeroPageX, 6, false),
        (CLD, Implicit, 2, false),
        (CMP, AbsoluteY, 4, true),
        (NOP, Implicit, 2, false),
        (DCP, AbsoluteY, 7, false),
        (NOP, AbsoluteX, 4, true),
        (CMP, AbsoluteX, 4, true),
        (DEC, AbsoluteX, 7, false),
        (DCP, AbsoluteX, 7, false),

        // Ex
        (CPX, Immediate, 2, false),
        (SBC, IndirectX, 6, false),
        (NOP, Immediate, 2, false),
        (ISC, IndirectX, 8, false),
        (CPX, ZeroPage, 3, false),
        (SBC, ZeroPage, 3, false),
        (INC, ZeroPage, 5, false),
        (ISC, ZeroPage, 5, false),
        (INX, Implicit, 2, false),
        (SBC, Immediate, 2, false),
        (NOP, Implicit, 2, false),
        (SBC, Immediate, 2, false),
        (CPX, Absolute, 4, false),
        (SBC, Absolute, 4, false),
        (INC, Absolute, 6, false),
        (ISC, Absolute, 6, false),

        // Fx
        (BEQ, Relative, 2, true),
        (SBC, IndirectY, 5, true),
        (KIL, Implicit, 0, false),
        (ISC, IndirectY, 8, false),
        (NOP, ZeroPageX, 4, false),
        (SBC, ZeroPageX, 4, false),
        (INC, ZeroPageX, 6, false),
        (ISC, ZeroPageX, 6, false),
        (SED, Implicit, 2, false),
        (SBC, AbsoluteY, 4, true),
        (NOP, Implicit, 2, false),
        (ISC, AbsoluteY, 7, false),
        (NOP, AbsoluteX, 4, true),
        (SBC, AbsoluteX, 4, true),
        (INC, AbsoluteX, 7, false),
        (ISC, AbsoluteX, 7, false),
    ];

    pub fn resolve(code: u8) -> &'static Opcode {
        &TABLE[code as usize]
    }
}

bitflags! {
    struct Status: u8 {
        const CARRY = 0b0000_0001;
        const ZERO = 0b0000_0010;
        const INTERRUPT_DISABLE= 0b0000_0100;
        const BREAK = 0b0010_0000;
        const DECIMAL = 0b0000_1000;
        const OVERFLOW = 0b0100_0000;
        const NEGATIVE = 0b1000_0000;
    }
}

pub struct Cpu {
    // address space
    mem: Box<CpuMem>,

    // registers
    a: u8, // accumulator
    x: u8, // index X
    y: u8, // index Y
    pc: u16, // program counter
    s: u8, // stack
    p: Status, // flags

    nmi: bool,
    irq: bool,
    reset: bool,

    remaining_pause: u16,
    instruction_counter: u64,
}

use opcodes::Opcode;
use opcodes::Operation::*;
use opcodes::AddressMode::*;

const SIGN_BIT: u8 = 0b1000_0000;

const NMI_VECTOR: u16 = 0xFFFA;
const RESET_VECTOR: u16 = 0xFFFC;
const IRQ_VECTOR: u16 = 0xFFFE;

// The mask of bits that get turned on when the P register is represented on the stack.
const PHP_MASK: u8 = 0b0011_0000;

impl Cpu {
    pub fn new(mem: Box<CpuMem>, test_mode: bool) -> Cpu {
        // startup state: https://wiki.nesdev.com/w/index.php/CPU_power_up_state
        let mut out = Cpu {
            mem,
            a: 0,
            x: 0,
            y: 0,
            pc: 0x8000,
            s: 0xfd,
            p: Status::BREAK | Status::INTERRUPT_DISABLE,
            nmi: false,
            irq: false,
            reset: false,
            remaining_pause: 0,
            instruction_counter: 0,
        };
        if !test_mode {
            out.pc = join_bytes(out.mem.get(RESET_VECTOR + 1), out.mem.get(RESET_VECTOR));
        }
        out
    }

    fn next_byte(&self) -> u8 {
        self.mem.get(self.pc + 1)
    }

    /// Returns whether the addresses are on different 256-byte pages.
    fn _different_pages(&self, addr1: u16, addr2: u16) -> bool {
        (addr1 & 0xFF00) != (addr2 & 0xFF00)
    }

    fn absolute_lookup(&self, jump: Option<u8>) -> (u16, bool) {
        let origin = join_bytes(self.mem.get(self.pc + 2), self.mem.get(self.pc + 1));
        match jump {
            Some(value) => {
                let dest = origin.wrapping_add(value as u16);
                (dest, self._different_pages(origin, dest))
            }
            None => (origin, false),
        }
    }

    /// Returns the address in memory that the opcode points to, based on the current
    /// position of `PC` and the opcode's address mode. Panics if the opcode's address mode is
    /// `ACCUMULATOR` or `IMPLICIT`, because in both cases the handler needs to do something
    /// other than resolve an address in memory. Also returns a bool that is true if a page was
    /// crossed (for the purpose of deciding whether there's a page crossing penalty).
    fn resolve_addr(&self, op: &Opcode) -> (u16, bool) {
        match op.1 {
            Accumulator => (0, false),
            Implicit => (0, false),
            Immediate => (self.pc + 1, false),
            Absolute => self.absolute_lookup(None),
            AbsoluteX => self.absolute_lookup(Some(self.x)),
            AbsoluteY => self.absolute_lookup(Some(self.y)),
            ZeroPage => (join_bytes(0x0, self.next_byte()), false),
            ZeroPageX => (join_bytes(0x0, self.next_byte().wrapping_add(self.x)), false),
            ZeroPageY => (join_bytes(0x0, self.next_byte().wrapping_add(self.y)), false),
            Relative => {
                let origin = self.pc;
                let dest = origin.wrapping_add((self.next_byte() as i8) as u16);
                (dest, self._different_pages(origin, dest))
            }
            Indirect => {
                let addr = join_bytes(self.mem.get(self.pc + 2), self.mem.get(self.pc + 1));
                let high_byte_addr = match (addr & 0x00FF) == 0x00FF {
                    // crazy 6502 bug!
                    true => addr & 0xFF00,
                    false => addr + 1
                };
                (join_bytes(self.mem.get(high_byte_addr), self.mem.get(addr)), false)
            }
            IndirectX => {
                let arg = self.next_byte().wrapping_add(self.x);
                let low = self.mem.get(join_bytes(0x0, arg));
                let high = self.mem.get(join_bytes(0x0, arg.wrapping_add(1)));
                (join_bytes(high, low), false)
            },
            IndirectY => {
                let arg = self.next_byte();
                let low = self.mem.get(join_bytes(0x0, arg));
                let high = self.mem.get(join_bytes(0x0, arg.wrapping_add(1)));
                let origin = join_bytes(high, low);
                let dest = origin.wrapping_add(self.y as u16);
                (dest, self._different_pages(origin, dest))
            }
        }
    }

    fn stack_push(&mut self, datum: u8) {
        self.mem_write(join_bytes(0x01, self.s), datum);
        self.s = self.s.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.mem.get(join_bytes(0x01, self.s))
        // should this clear the byte it's pointing to? it'll be overwritten on next push
    }

    /// Triggers an interrupt, with the `P` flag on the stack or'ed with the `b_mask`, and
    /// pointing to the `vector` starting at the specified address.
    fn interrupt(&mut self, b_mask: u8, vector: u16) {
        self.stack_push((self.pc >> 8) as u8);
        self.stack_push(self.pc as u8);
        self.stack_push(self.p.bits() | b_mask);
        self.pc = join_bytes(self.mem.get(vector + 1), self.mem.get(vector));
    }

    fn irq(&mut self) {
        self.irq = false;
        if !self.interrupt_disabled() {
            self.interrupt(0b0011_0000, IRQ_VECTOR)
        }
    }

    fn nmi(&mut self) {
        self.nmi = false;
        self.interrupt(0b0011_0000, NMI_VECTOR)
    }

    fn reset(&mut self) {
        self.reset = false;
        self.interrupt(0b0011_0000, RESET_VECTOR)
    }

    fn page_crossed_penalty(&self, pause: u16, op: &Opcode, page_crossed: bool) -> u16 {
        if op.3 && page_crossed {
            pause + 1
        } else {
            pause
        }
    }

    /// Sets the remaining cycle pauses appropriately, and returns the number of bytes
    /// the program counter should advance. The cycle pause count should be _one lower than
    /// the documentation says_, because we're using up the first cycle executing the
    /// instruction in the first place.
    fn set_pause_and_return_shift(&mut self, pause: u16, op: &Opcode, page_crossed: bool) -> u16 {
        self.remaining_pause = self.page_crossed_penalty(pause, op, page_crossed);
        op.1.byte_count()
    }

    fn _group_1_pause_and_shift(&mut self, op: &Opcode, page_crossed: bool) -> u16 {
        // ZeroPageY isn't actually part of these, but LDX allows it and is otherwise identical
        match op.1 {
            Immediate => self.set_pause_and_return_shift(1, op, page_crossed),
            ZeroPage => self.set_pause_and_return_shift(2, op, page_crossed),
            ZeroPageX | ZeroPageY | Absolute => self.set_pause_and_return_shift(3, op, page_crossed),
            AbsoluteX | AbsoluteY => self.set_pause_and_return_shift(3, op, page_crossed),
            IndirectX => self.set_pause_and_return_shift(5, op, page_crossed),
            IndirectY => self.set_pause_and_return_shift(4, op, page_crossed),
            _ => unreachable!()
        }
    }

    fn _illegal_opcodes_pause_and_shift(&mut self, op: &Opcode, page_crossed: bool) -> u16 {
        match op.1 {
            Absolute => self.set_pause_and_return_shift(5, op, page_crossed),
            AbsoluteX | AbsoluteY => self.set_pause_and_return_shift(6, op, page_crossed),
            ZeroPage => self.set_pause_and_return_shift(4, op, page_crossed),
            ZeroPageX => self.set_pause_and_return_shift(5, op, page_crossed),
            IndirectX | IndirectY => self.set_pause_and_return_shift(7, op, page_crossed),
            _ => unreachable!()
        }
    }

    /// Executes the opcode, updating all registers appropriately.
    fn execute_opcode(&mut self, op: &Opcode) {
        self.pc += match op.0 {
            ADC => self.adc(op),
            AND => self.and(op),
            ASL => self.asl(op),
            BIT => self.bit(op),
            BRK => self.brk(op),
            EOR => self.eor(op),
            DEC => self.dec(op),
            DEX => self.dex(op),
            DEY => self.dey(op),
            INC => self.inc(op),
            INX => self.inx(op),
            INY => self.iny(op),
            JMP => self.jmp(op),
            JSR => self.jsr(op),
            LDA => self.lda(op),
            LDX => self.ldx(op),
            LDY => self.ldy(op),
            LSR => self.lsr(op),
            NOP => self.nop(op),
            ORA => self.ora(op),
            PHP => self.php(op),
            PHA => self.pha(op),
            PLA => self.pla(op),
            PLP => self.plp(op),
            ROL => self.rol(op),
            ROR => self.ror(op),
            RTI => self.rti(op),
            RTS => self.rts(op),
            SBC => self.sbc(op),
            STA => self.sta(op),
            STX => self.stx(op),
            STY => self.sty(op),

            // "illegal", and do weird special things
            LAX => self.lax(op),
            SAX => self.sax(op),

            // "illegal", and just do two regular things
            DCP => self.illegal_op(op, |cpu, opc| {cpu.dec(opc); cpu.compare_op(opc, cpu.a);}),
            ISC => self.illegal_op(op, |cpu, opc| {cpu.inc(opc); cpu.sbc(opc);}),
            SLO => self.illegal_op(op, |cpu, opc| {cpu.asl(opc); cpu.ora(opc);}),
            SRE => self.illegal_op(op, |cpu, opc| {cpu.lsr(opc); cpu.eor(opc);}),
            RRA => self.illegal_op(op, |cpu, opc| {cpu.ror(opc); cpu.adc(opc);}),
            RLA => self.illegal_op(op, |cpu, opc| {cpu.rol(opc); cpu.and(opc);}),

            // comparisons
            CMP => self.compare_op(op, self.a),
            CPX => self.compare_op(op, self.x),
            CPY => self.compare_op(op, self.y),

            // branches
            BCS => self.branch_op(op, self.carry()),
            BCC => self.branch_op(op, !self.carry()),
            BEQ => self.branch_op(op, self.zero()),
            BNE => self.branch_op(op, !self.zero()),
            BVS => self.branch_op(op, self.overflow()),
            BVC => self.branch_op(op, !self.overflow()),
            BMI => self.branch_op(op, self.negative()),
            BPL => self.branch_op(op, !self.negative()),

            // simple flag settings
            SEC => self.flag_op(|cpu| cpu.set_carry(true)),
            SED => self.flag_op(|cpu| cpu.set_decimal(true)),
            SEI => self.flag_op(|cpu| cpu.set_interrupt_disable(true)),
            CLC => self.flag_op(|cpu| cpu.set_carry(false)),
            CLD => self.flag_op(|cpu| cpu.set_decimal(false)),
            CLI => self.flag_op(|cpu| cpu.set_interrupt_disable(false)),
            CLV => self.flag_op(|cpu| cpu.set_overflow(false)),

            // transfers
            TAX => self.transfer_op(|cpu| { cpu.x = cpu.a; (cpu.x, true) }),
            TAY => self.transfer_op(|cpu| { cpu.y = cpu.a; (cpu.y, true) }),
            TXS => self.transfer_op(|cpu| { cpu.s = cpu.x; (cpu.s, false) }),
            TSX => self.transfer_op(|cpu| { cpu.x = cpu.s; (cpu.x, true) }),
            TXA => self.transfer_op(|cpu| { cpu.a = cpu.x; (cpu.a, true) }),
            TYA => self.transfer_op(|cpu| { cpu.a = cpu.y; (cpu.a, true) }),

            _ => unimplemented!("addr {:04X?} -> {:?}", self.pc, op)
        }
    }

    fn set_flag(&mut self, mask: Status, set_to: bool) {
        self.p.set(mask, set_to)
    }

    fn carry(&self) -> bool {
        self.p.contains(Status::CARRY)
    }

    fn zero(&self) -> bool {
        self.p.contains(Status::ZERO)
    }

    fn overflow(&self) -> bool {
        self.p.contains(Status::OVERFLOW)
    }

    fn negative(&self) -> bool {
        self.p.contains(Status::NEGATIVE)
    }

    fn interrupt_disabled(&self) -> bool {
        self.p.contains(Status::INTERRUPT_DISABLE)
    }

    fn set_carry(&mut self, carry: bool) {
        self.set_flag(Status::CARRY, carry);
    }

    fn set_zero(&mut self, zero: bool) {
        self.set_flag(Status::ZERO, zero);
    }

    fn set_interrupt_disable(&mut self, interrupt_disable: bool) {
        self.set_flag(Status::INTERRUPT_DISABLE, interrupt_disable);
    }

    fn set_decimal(&mut self, decimal: bool) {
        self.set_flag(Status::DECIMAL, decimal);
    }

    fn set_overflow(&mut self, overflow: bool) {
        self.set_flag(Status::OVERFLOW, overflow);
    }

    fn set_negative(&mut self, negative: bool) {
        self.set_flag(Status::NEGATIVE, negative);
    }

    fn set_value_flags(&mut self, val: u8) {
        self.set_negative((val & SIGN_BIT) != 0);
        self.set_zero(val == 0);
    }

    fn mem_write(&mut self, addr: u16, val: u8) {
        if addr == 0x4014 {
            let dma = self.mem.get_page(join_bytes(val, 0));
            self.mem.bus.borrow_mut().set_oamdma(dma);
            self.remaining_pause += 513;  // TODO odd cycle??
        } else {
            self.mem.set(addr, val);
        }
    }

    // Opcodes!

    fn flag_op(&mut self, func: fn(&mut Cpu) -> ()) -> u16 {
        func(self);
        self.remaining_pause = 2;
        1
    }

    fn illegal_op(&mut self, op: &Opcode, func: fn(&mut Cpu, &Opcode) -> ()) -> u16 {
        func(self, op);
        self.remaining_pause = 0;  // zero out pause from earlier functions
        // TODO handle illegal pause??
        self._illegal_opcodes_pause_and_shift(op, false)
    }

    fn adc(&mut self, op: &Opcode) -> u16 {
        let (addr, page_crossed) = self.resolve_addr(op);
        let value = self.mem.get(addr);
        let signed_sum = (value as i8 as i16) + (self.a as i8 as i16) + (self.carry() as i16);
        let (first_add, overflowing1) = self.a.overflowing_add(value);
        let (second_add, overflowing2) = first_add.overflowing_add(if self.carry() { 1 } else { 0 });
        self.a = second_add;
        self.set_carry(overflowing1 || overflowing2);
        self.set_value_flags(self.a);
        self.set_overflow(signed_sum < -128 || signed_sum > 127);
        self._group_1_pause_and_shift(op, page_crossed)
    }

    fn and(&mut self, op: &Opcode) -> u16 {
        let operand = self.mem.get(self.resolve_addr(op).0);
        self.a &= operand;
        self.set_value_flags(self.a);
        self._group_1_pause_and_shift(op, false)
    }

    fn asl(&mut self, op: &Opcode) -> u16 {
        if op.1 == Accumulator {
            let bit_7 = (self.a & 0b1000_0000) != 0;
            self.a <<= 1;
            self.set_carry(bit_7 as bool);
            self.set_value_flags(self.a);
        } else {
            let (addr, _) = self.resolve_addr(op);
            let mut value = self.mem.get(addr);
            let bit_7 = (value & 0b1000_0000) != 0;
            value <<= 1;
            self.set_carry(bit_7 as bool);
            self.set_value_flags(value);
            self.mem_write(addr, value);
        }
        match op.1 {
            Accumulator => self.set_pause_and_return_shift(1, op, false),
            ZeroPage => self.set_pause_and_return_shift(4, op, false),
            ZeroPageX | Absolute => self.set_pause_and_return_shift(5, op, false),
            AbsoluteX => self.set_pause_and_return_shift(6, op, false),

            // used by SLO, overridden by it
            AbsoluteY | IndirectX | IndirectY => 0,
            _ => unreachable!()
        }
    }

    fn bit(&mut self, op: &Opcode) -> u16 {
        let (addr, page_crossed) = self.resolve_addr(op);
        let value = self.mem.get(addr);
        self.set_negative((value & SIGN_BIT) != 0);
        self.set_overflow((value & 0b0100_0000) != 0);
        self.set_zero((value & self.a) == 0);
        match op.1 {
            ZeroPage => self.set_pause_and_return_shift(2, op, page_crossed),
            Absolute => self.set_pause_and_return_shift(3, op, page_crossed),
            _ => unreachable!()
        }
    }

    fn branch_op(&mut self, op: &Opcode, branch: bool) -> u16 {
        self.remaining_pause = 2;
        if branch {
            let (addr, page_crossed) = self.resolve_addr(op);
            self.pc = addr;
            self.remaining_pause += if page_crossed { 3 } else { 1 };
        }
        2 // account for the 2 bytes this instruction used
    }

    fn brk(&mut self, _op: &Opcode) -> u16 {
        self.pc += 1;
        if self.interrupt_disabled() {
            return 0;
        }
        self.remaining_pause = 6;
        self.interrupt(0b0011_0000, IRQ_VECTOR);
        0
    }

    fn compare_op(&mut self, op: &Opcode, to: u8) -> u16 {
        let (addr, page_crossed) = self.resolve_addr(op);
        let value = self.mem.get(addr);
        self.set_carry(to >= value);
        self.set_zero(to == value);
        let result = to.wrapping_sub(value);
        self.set_negative(result >= 128);
        self._group_1_pause_and_shift(op, page_crossed)
    }

    fn dec(&mut self, op: &Opcode) -> u16 {
        let (addr, _) = self.resolve_addr(op);
        let (new_val, shift) = self.increment(self.mem.get(addr), op, true);
        self.mem_write(addr, new_val);
        shift
    }

    fn dex(&mut self, op: &Opcode) -> u16 {
        let (new_val, shift) = self.increment(self.x, op, true);
        self.x = new_val;
        shift
    }

    fn dey(&mut self, op: &Opcode) -> u16 {
        let (new_val, shift) = self.increment(self.y, op, true);
        self.y = new_val;
        shift
    }

    fn eor(&mut self, op: &Opcode) -> u16 {
        let (addr, page_crossed) = self.resolve_addr(op);
        self.a ^= self.mem.get(addr);
        self.set_value_flags(self.a);
        self._group_1_pause_and_shift(op, page_crossed)
    }

    fn increment(&mut self, mut val: u8, op: &Opcode, decrement: bool) -> (u8, u16) {
        if decrement { val = val.wrapping_sub(1); } else { val = val.wrapping_add(1); }
        self.set_value_flags(val);
        (val, match op.1 {
            Implicit => self.set_pause_and_return_shift(1, op, false),
            ZeroPage => self.set_pause_and_return_shift(4, op, false),
            ZeroPageX | Absolute => self.set_pause_and_return_shift(5, op, false),
            AbsoluteX => self.set_pause_and_return_shift(6, op, false),
            _ => 0 // reachable only via illegal opcodes, which overwrite this anyway
        })
    }

    fn inc(&mut self, op: &Opcode) -> u16 {
        let (addr, _) = self.resolve_addr(op);
        let (new_val, shift) = self.increment(self.mem.get(addr), op, false);
        self.mem_write(addr, new_val);
        shift
    }

    fn inx(&mut self, op: &Opcode) -> u16 {
        let (new_val, shift) = self.increment(self.x, op, false);
        self.x = new_val;
        shift
    }

    fn iny(&mut self, op: &Opcode) -> u16 {
        let (new_val, shift) = self.increment(self.y, op, false);
        self.y = new_val;
        shift
    }

    fn jmp(&mut self, op: &Opcode) -> u16 {
        let (addr, _) = self.resolve_addr(op);
        self.pc = addr;
        match op.1 {
            Absolute => self.remaining_pause = 2,
            Indirect => self.remaining_pause = 4,
            _ => unreachable!()
        }
        0 // this is a jump, we don't advance normally
    }

    fn jsr(&mut self, op: &Opcode) -> u16 {
        let (addr, _) = self.resolve_addr(op);
        let bytes = (self.pc + 2).to_be_bytes();
        self.stack_push(bytes[0]);
        self.stack_push(bytes[1]);
        self.pc = addr;
        self.remaining_pause = 5;
        0 // this is a jump, we don't advance normally
    }

    fn lax(&mut self, op: &Opcode) -> u16 {
        self.lda(op);
        self.transfer_op(|cpu| { cpu.x = cpu.a; (cpu.x, true) });
        self.set_value_flags(self.x);

        // cycle crossing penalty handled by lda, which has the same cycle characteristics!
        match op.1 {
            IndirectX => self.set_pause_and_return_shift(0, op, false),
            ZeroPage => self.set_pause_and_return_shift(0, op, false),
            Absolute | ZeroPageY => self.set_pause_and_return_shift(3, op, false),
            IndirectY => self.set_pause_and_return_shift(0, op, false),
            AbsoluteY => self.set_pause_and_return_shift(0, op, false),
            _ => unreachable!()
        }
    }

    // TODO these only differ by register; is there some way to make them one func?

    fn lda(&mut self, op: &Opcode) -> u16 {
        let (addr, page_crossed) = self.resolve_addr(op);
        let value = self.mem.get(addr);
        self.a = value;
        self.set_value_flags(value);
        self._group_1_pause_and_shift(op, page_crossed)
    }

    fn ldx(&mut self, op: &Opcode) -> u16 {
        let (addr, page_crossed) = self.resolve_addr(op);
        let value = self.mem.get(addr);
        self.x = value;
        self.set_value_flags(value);
        self._group_1_pause_and_shift(op, page_crossed)
    }

    fn ldy(&mut self, op: &Opcode) -> u16 {
        let (addr, page_crossed) = self.resolve_addr(op);
        let value = self.mem.get(addr);
        self.y = value;
        self.set_value_flags(value);
        self._group_1_pause_and_shift(op, page_crossed)
    }

    fn lsr(&mut self, op: &Opcode) -> u16 {
        if op.1 == Accumulator {
            let bit_1 = (self.a & 0b1) != 0;
            self.a >>= 1;
            self.set_carry(bit_1 as bool);
            self.set_value_flags(self.a);
        } else {
            let (addr, _) = self.resolve_addr(op);
            let mut value = self.mem.get(addr);
            let bit_1 = (value & 0b1) != 0;
            value >>= 1;
            self.set_carry(bit_1 as bool);
            self.set_value_flags(value);
            self.mem_write(addr, value);
        }
        match op.1 {
            Accumulator => self.set_pause_and_return_shift(1, op, false),
            ZeroPage => self.set_pause_and_return_shift(4, op, false),
            ZeroPageX | Absolute => self.set_pause_and_return_shift(5, op, false),
            AbsoluteX => self.set_pause_and_return_shift(6, op, false),

            // used by SRE and overridden by it
            AbsoluteY | IndirectX | IndirectY => 0,
            _ => unreachable!()
        }
    }

    fn nop(&mut self, op: &Opcode) -> u16 {
        self.remaining_pause = 2;
        op.1.byte_count() // TODO this technically does a read and can take more cycles based on that!
    }

    fn ora(&mut self, op: &Opcode) -> u16 {
        let (addr, page_crossed) = self.resolve_addr(op);
        self.a |= self.mem.get(addr);
        self.set_value_flags(self.a);
        self._group_1_pause_and_shift(op, page_crossed)
    }

    fn php(&mut self, _op: &Opcode) -> u16 {
        self.stack_push(self.p.bits() | PHP_MASK);
        self.remaining_pause = 2;
        1
    }

    fn pha(&mut self, _op: &Opcode) -> u16 {
        self.stack_push(self.a);
        self.remaining_pause = 2;
        1
    }

    fn pla(&mut self, _op: &Opcode) -> u16 {
        self.a = self.stack_pop();
        self.set_value_flags(self.a);
        self.remaining_pause = 3;
        1
    }

    fn plp(&mut self, _op: &Opcode) -> u16 {
        self.p = Status::from_bits_truncate(self.stack_pop() & !PHP_MASK);
        self.remaining_pause = 3;
        1
    }

    fn rol_internal(&mut self, value: u8) -> u8 {
        let old_carry = self.carry();
        self.set_carry((value & 0b1000_0000) != 0);
        let mut out = value << 1;
        if old_carry {
            out ^= 0b0000_0001;
        };
        self.set_value_flags(out);
        out
    }

    fn rol(&mut self, op: &Opcode) -> u16 {
        if op.1 == Accumulator {
            self.a = self.rol_internal(self.a);
        } else {
            let (addr, _) = self.resolve_addr(op);
            let new_val = self.rol_internal(self.mem.get(addr));
            self.mem_write(addr, new_val);
        }
        match op.1 {
            Accumulator => self.set_pause_and_return_shift(1, op, false),
            ZeroPage => self.set_pause_and_return_shift(4, op, false),
            ZeroPageX | Absolute => self.set_pause_and_return_shift(5, op, false),
            AbsoluteX => self.set_pause_and_return_shift(6, op, false),

            // used by RLA, overridden by it
            AbsoluteY | IndirectX | IndirectY => 0,
            _ => unreachable!()
        }
    }

    fn ror_internal(&mut self, value: u8) -> u8 {
        let old_carry = self.carry();
        self.set_carry((value & 1) != 0);
        let mut out = value >> 1;
        if old_carry {
            out ^= 0b1000_0000;
        };
        self.set_value_flags(out);
        out
    }

    fn ror(&mut self, op: &Opcode) -> u16 {
        if op.1 == Accumulator {
            self.a = self.ror_internal(self.a);
        } else {
            let (addr, _) = self.resolve_addr(op);
            let new_val = self.ror_internal(self.mem.get(addr));
            self.mem_write(addr, new_val);
        }
        match op.1 {
            Accumulator => self.set_pause_and_return_shift(1, op, false),
            ZeroPage => self.set_pause_and_return_shift(4, op, false),
            ZeroPageX | Absolute => self.set_pause_and_return_shift(5, op, false),
            AbsoluteX => self.set_pause_and_return_shift(6, op, false),

            // used by RRA, overridden by it
            AbsoluteY | IndirectX | IndirectY => 0,
            _ => unreachable!()
        }
    }

    fn rti(&mut self, op: &Opcode) -> u16 {
        self.p = Status::from_bits_truncate(self.stack_pop());
        self.remaining_pause += 1;
        self.rts(op);
        0  // do not advance one byte!
    }

    fn rts(&mut self, _op: &Opcode) -> u16 {
        let low = self.stack_pop();
        let high = self.stack_pop();
        self.pc = join_bytes(high, low);
        self.remaining_pause += 5;  // increment so this can be called from #rti
        1  // advance once byte!
    }

    fn sax(&mut self, op: &Opcode) -> u16 {
        self.mem_write(self.resolve_addr(op).0, self.a & self.x);
        match op.1 {
            ZeroPage => self.set_pause_and_return_shift(2, op, false),
            Absolute | ZeroPageY => self.set_pause_and_return_shift(3, op, false),
            IndirectX => self.set_pause_and_return_shift(5, op, false),
            _ => unreachable!()
        }
    }

    fn sbc(&mut self, op: &Opcode) -> u16 {
        let (addr, page_crossed) = self.resolve_addr(op);
        let value = self.mem.get(addr);
        let signed_sum = (value as i8 as i16) - (self.a as i8 as i16) - (1 - (self.carry() as i16));
        let (first_sub, overflowing1) = self.a.overflowing_sub(value);
        let (second_sub, overflowing2) = first_sub.overflowing_sub(1 - (self.carry() as u8));
        self.a = second_sub;
        self.set_carry(!(overflowing1 || overflowing2));
        self.set_value_flags(self.a);
        self.set_overflow(signed_sum < -128 || signed_sum > 127);
        self._group_1_pause_and_shift(op, page_crossed)
    }

    fn store(&mut self, op: &Opcode, value: u8) -> u16 {
        self.mem_write(self.resolve_addr(op).0, value);
        match op.1 {
            ZeroPage => self.set_pause_and_return_shift(2, op, false),
            ZeroPageX | ZeroPageY | Absolute => self.set_pause_and_return_shift(3, op, false),
            AbsoluteX | AbsoluteY => self.set_pause_and_return_shift(4, op, false),
            IndirectX | IndirectY => self.set_pause_and_return_shift(5, op, false),
            _ => unreachable!()
        }
    }

    fn sta(&mut self, op: &Opcode) -> u16 {
        self.store(op, self.a)
    }

    fn stx(&mut self, op: &Opcode) -> u16 {
        self.store(op, self.x)
    }

    fn sty(&mut self, op: &Opcode) -> u16 {
        self.store(op, self.y)
    }

    fn transfer_op(&mut self, func: fn(&mut Cpu) -> (u8, bool)) -> u16 {
        let (new_val, update_flags) = func(self);
        if update_flags {
            self.set_value_flags(new_val);
        }
        self.remaining_pause = 1;
        1
    }

    pub fn flag_nmi(&mut self) {
        self.nmi = true;
    }

    pub fn flag_irq(&mut self) {
        self.irq = true;
    }

    pub fn flag_reset(&mut self) {
        self.reset = true;
    }
}

impl Clocked for Cpu {
    fn tick(&mut self) {
        if self.remaining_pause > 0 {
            self.remaining_pause -= 1;
            return
        } else if self.nmi {
            self.nmi();
            return
        } else if self.irq {
            self.irq();
            return
        } else if self.reset {
            self.reset();
            return
        }

        self.instruction_counter += 1;
        let op = opcodes::resolve(self.mem.get(self.pc));
        trace!("{:?} @ {:04X?} (A:{:02X?} X:{:02X?} Y:{:02X?} P:{:02X?} SP:{:02X?}): {:?}: {:04X?}",
               self.instruction_counter, self.pc, self.a, self.x, self.y, self.p.bits(), self.s,
               op, self.resolve_addr(op));
        self.execute_opcode(op);
    }
}

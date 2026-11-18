//! Ethernet II frame transmitter with an eight-byte payload.

use crate::catalog_bits::{equals, increment, mux_tree, register_bits};
use crate::catalog_digital::{DigitalEntry, finish};
use crate::{DigitalBuilder, NetId};

const OUTPUTS: [&str; 4] = ["tx", "tx_en", "busy", "done"];
const RESET_START: [&str; 2] = ["reset", "start"];
const DEST_NAMES: [&str; 48] = [
    "dest0", "dest1", "dest2", "dest3", "dest4", "dest5", "dest6", "dest7", "dest8", "dest9",
    "dest10", "dest11", "dest12", "dest13", "dest14", "dest15", "dest16", "dest17", "dest18",
    "dest19", "dest20", "dest21", "dest22", "dest23", "dest24", "dest25", "dest26", "dest27",
    "dest28", "dest29", "dest30", "dest31", "dest32", "dest33", "dest34", "dest35", "dest36",
    "dest37", "dest38", "dest39", "dest40", "dest41", "dest42", "dest43", "dest44", "dest45",
    "dest46", "dest47",
];
const SRC_NAMES: [&str; 48] = [
    "src0", "src1", "src2", "src3", "src4", "src5", "src6", "src7", "src8", "src9", "src10",
    "src11", "src12", "src13", "src14", "src15", "src16", "src17", "src18", "src19", "src20",
    "src21", "src22", "src23", "src24", "src25", "src26", "src27", "src28", "src29", "src30",
    "src31", "src32", "src33", "src34", "src35", "src36", "src37", "src38", "src39", "src40",
    "src41", "src42", "src43", "src44", "src45", "src46", "src47",
];
const TYPE_NAMES: [&str; 16] = [
    "type0", "type1", "type2", "type3", "type4", "type5", "type6", "type7", "type8", "type9",
    "type10", "type11", "type12", "type13", "type14", "type15",
];
const DATA_NAMES: [&str; 64] = [
    "data0", "data1", "data2", "data3", "data4", "data5", "data6", "data7", "data8", "data9",
    "data10", "data11", "data12", "data13", "data14", "data15", "data16", "data17", "data18",
    "data19", "data20", "data21", "data22", "data23", "data24", "data25", "data26", "data27",
    "data28", "data29", "data30", "data31", "data32", "data33", "data34", "data35", "data36",
    "data37", "data38", "data39", "data40", "data41", "data42", "data43", "data44", "data45",
    "data46", "data47", "data48", "data49", "data50", "data51", "data52", "data53", "data54",
    "data55", "data56", "data57", "data58", "data59", "data60", "data61", "data62", "data63",
];

const IDLE: u32 = 0;
const PREAMBLE: u32 = 1;
const HEADER: u32 = 2;
const PAYLOAD: u32 = 3;
const PAD: u32 = 4;
const FCS: u32 = 5;
const DONE: u32 = 6;
const POLY: u32 = 0xedb8_8320;

#[derive(Clone, Copy)]
struct EthernetInputs {
    reset: NetId,
    start: NetId,
    dest: [NetId; 48],
    src: [NetId; 48],
    etype: [NetId; 16],
    payload: [NetId; 64],
}

struct EthernetState {
    phase: [NetId; 3],
    byte: [NetId; 6],
    bit: [NetId; 3],
    crc: [NetId; 32],
}

#[derive(Clone, Copy)]
struct EthernetPhase {
    reset: NetId,
    flags: [NetId; 7],
    start_load: NetId,
}

impl EthernetPhase {
    fn idle(&self) -> NetId {
        self.flags[IDLE as usize]
    }
    fn preamble(&self) -> NetId {
        self.flags[PREAMBLE as usize]
    }
    fn header(&self) -> NetId {
        self.flags[HEADER as usize]
    }
    fn payload(&self) -> NetId {
        self.flags[PAYLOAD as usize]
    }
    fn pad(&self) -> NetId {
        self.flags[PAD as usize]
    }
    fn fcs(&self) -> NetId {
        self.flags[FCS as usize]
    }
    fn done(&self) -> NetId {
        self.flags[DONE as usize]
    }
}

struct EthernetNext {
    phase: [NetId; 3],
    byte: [NetId; 6],
    bit: [NetId; 3],
    crc: [NetId; 32],
}

pub(crate) fn ethernet_entry() -> DigitalEntry {
    finish("mcu.serial.ethernet", |builder| {
        let inputs = ethernet_inputs(builder);
        let state = ethernet_state(builder);
        ethernet_registers(builder, inputs, &state);
        ethernet_outputs(builder, inputs, &state);
        (input_names(), OUTPUTS.to_vec())
    })
}

fn input_names() -> Vec<&'static str> {
    let mut names = RESET_START.to_vec();
    names.extend_from_slice(&DEST_NAMES);
    names.extend_from_slice(&SRC_NAMES);
    names.extend_from_slice(&TYPE_NAMES);
    names.extend_from_slice(&DATA_NAMES);
    names
}

fn ethernet_inputs(builder: &mut DigitalBuilder) -> EthernetInputs {
    EthernetInputs {
        reset: builder.input("reset"),
        start: builder.input("start"),
        dest: std::array::from_fn(|index| builder.input(DEST_NAMES[index])),
        src: std::array::from_fn(|index| builder.input(SRC_NAMES[index])),
        etype: std::array::from_fn(|index| builder.input(TYPE_NAMES[index])),
        payload: std::array::from_fn(|index| builder.input(DATA_NAMES[index])),
    }
}

fn ethernet_state(builder: &mut DigitalBuilder) -> EthernetState {
    EthernetState {
        phase: [builder.net(), builder.net(), builder.net()],
        byte: std::array::from_fn(|_| builder.net()),
        bit: [builder.net(), builder.net(), builder.net()],
        crc: std::array::from_fn(|_| builder.net()),
    }
}

fn ethernet_phase(
    builder: &mut DigitalBuilder,
    inputs: EthernetInputs,
    state: &EthernetState,
) -> EthernetPhase {
    let flags = std::array::from_fn(|index| equals(builder, &state.phase, index as u32));
    let idle = flags[IDLE as usize];
    let done = flags[DONE as usize];
    let ready = builder.or(idle, done);
    let start_load = builder.and(inputs.start, ready);
    EthernetPhase {
        reset: inputs.reset,
        flags,
        start_load,
    }
}

fn ethernet_registers(builder: &mut DigitalBuilder, inputs: EthernetInputs, state: &EthernetState) {
    let phase = ethernet_phase(builder, inputs, state);
    let tx = tx_bit(builder, inputs, state);
    let next = ethernet_next(builder, state, &phase, tx);
    ethernet_store(builder, state, next);
}

fn ethernet_next(
    builder: &mut DigitalBuilder,
    state: &EthernetState,
    phase: &EthernetPhase,
    tx: NetId,
) -> EthernetNext {
    let (phase_next, byte) = phase_and_byte(builder, state, phase);
    let bit = bit_next(builder, state, phase);
    let crc = crc_next(builder, state, phase, tx);
    EthernetNext {
        phase: phase_next,
        byte,
        bit,
        crc,
    }
}

fn phase_and_byte(
    builder: &mut DigitalBuilder,
    state: &EthernetState,
    phase: &EthernetPhase,
) -> ([NetId; 3], [NetId; 6]) {
    let events = phase_events(builder, state, phase);
    let phase_next = std::array::from_fn(|bit| phase_bit(builder, state, phase, &events, bit));
    let byte = byte_next(builder, state, phase, &events);
    (phase_next, byte)
}

#[derive(Clone, Copy)]
struct PhaseEvents {
    preamble_end: NetId,
    header_end: NetId,
    payload_end: NetId,
    pad_end: NetId,
    fcs_end: NetId,
}

fn phase_events(
    builder: &mut DigitalBuilder,
    state: &EthernetState,
    phase: &EthernetPhase,
) -> PhaseEvents {
    let last = equals(builder, &state.bit, 7);
    PhaseEvents {
        preamble_end: end_at(builder, phase.preamble(), last, &state.byte, 7),
        header_end: end_at(builder, phase.header(), last, &state.byte, 13),
        payload_end: end_at(builder, phase.payload(), last, &state.byte, 7),
        pad_end: end_at(builder, phase.pad(), last, &state.byte, 37),
        fcs_end: end_at(builder, phase.fcs(), last, &state.byte, 3),
    }
}

fn end_at(
    builder: &mut DigitalBuilder,
    flag: NetId,
    last: NetId,
    byte: &[NetId; 6],
    value: u32,
) -> NetId {
    let at = equals(builder, byte, value);
    let end = builder.and(at, last);
    builder.and(flag, end)
}

fn phase_bit(
    builder: &mut DigitalBuilder,
    state: &EthernetState,
    phase: &EthernetPhase,
    events: &PhaseEvents,
    bit: usize,
) -> NetId {
    let mut value = state.phase[bit];
    value = phase_mux(builder, value, PREAMBLE, bit, phase.start_load);
    value = phase_mux(builder, value, HEADER, bit, events.preamble_end);
    value = phase_mux(builder, value, PAYLOAD, bit, events.header_end);
    value = phase_mux(builder, value, PAD, bit, events.payload_end);
    value = phase_mux(builder, value, FCS, bit, events.pad_end);
    value = phase_mux(builder, value, DONE, bit, events.fcs_end);
    phase_mux(builder, value, IDLE, bit, phase.reset)
}

fn phase_mux(
    builder: &mut DigitalBuilder,
    value: NetId,
    code: u32,
    bit: usize,
    select: NetId,
) -> NetId {
    let constant = const_bit(builder, code, bit);
    builder.mux(value, constant, select)
}

fn byte_next(
    builder: &mut DigitalBuilder,
    state: &EthernetState,
    phase: &EthernetPhase,
    events: &PhaseEvents,
) -> [NetId; 6] {
    let incremented = increment(builder, &state.byte);
    let active = active_phases(builder, phase);
    let last = equals(builder, &state.bit, 7);
    let advance = builder.and(active, last);
    let clear = byte_clear(builder, phase, events);
    let zero = builder.constant(false);
    std::array::from_fn(|bit| {
        let value = builder.mux(state.byte[bit], incremented[bit], advance);
        builder.mux(value, zero, clear)
    })
}

fn byte_clear(builder: &mut DigitalBuilder, phase: &EthernetPhase, events: &PhaseEvents) -> NetId {
    let mut clear = builder.or(phase.start_load, events.preamble_end);
    clear = builder.or(clear, events.header_end);
    clear = builder.or(clear, events.payload_end);
    clear = builder.or(clear, events.pad_end);
    clear = builder.or(clear, events.fcs_end);
    builder.or(clear, phase.reset)
}

fn bit_next(
    builder: &mut DigitalBuilder,
    state: &EthernetState,
    phase: &EthernetPhase,
) -> [NetId; 3] {
    let incremented = increment(builder, &state.bit);
    let active = active_phases(builder, phase);
    let last = equals(builder, &state.bit, 7);
    let zero = builder.constant(false);
    std::array::from_fn(|bit| {
        let value = builder.mux(state.bit[bit], incremented[bit], active);
        let value = builder.mux(value, zero, last);
        let value = builder.mux(value, zero, phase.start_load);
        builder.mux(value, zero, phase.reset)
    })
}

fn active_phases(builder: &mut DigitalBuilder, phase: &EthernetPhase) -> NetId {
    let mut active = builder.or(phase.preamble(), phase.header());
    active = builder.or(active, phase.payload());
    active = builder.or(active, phase.pad());
    builder.or(active, phase.fcs())
}

fn crc_next(
    builder: &mut DigitalBuilder,
    state: &EthernetState,
    phase: &EthernetPhase,
    tx: NetId,
) -> [NetId; 32] {
    let active = builder.or(phase.header(), phase.payload());
    let active = builder.or(active, phase.pad());
    let updated = crc_update(builder, &state.crc, tx);
    let one = builder.constant(true);
    std::array::from_fn(|bit| {
        let value = builder.mux(state.crc[bit], updated[bit], active);
        let value = builder.mux(value, one, phase.start_load);
        builder.mux(value, one, phase.reset)
    })
}

fn crc_update(builder: &mut DigitalBuilder, crc: &[NetId; 32], tx: NetId) -> [NetId; 32] {
    let shifted = shift_right(builder, crc);
    let feedback = builder.xor(crc[0], tx);
    std::array::from_fn(|index| {
        let base = shifted[index];
        if POLY & (1 << index) != 0 {
            builder.xor(base, feedback)
        } else {
            base
        }
    })
}

fn shift_right(builder: &mut DigitalBuilder, crc: &[NetId; 32]) -> [NetId; 32] {
    let zero = builder.constant(false);
    std::array::from_fn(|index| if index == 31 { zero } else { crc[index + 1] })
}

fn tx_bit(builder: &mut DigitalBuilder, inputs: EthernetInputs, state: &EthernetState) -> NetId {
    let byte = tx_byte(builder, inputs, state);
    let level: Vec<NetId> = byte.to_vec();
    mux_tree(builder, level, &state.bit)
}

fn tx_byte(
    builder: &mut DigitalBuilder,
    inputs: EthernetInputs,
    state: &EthernetState,
) -> [NetId; 8] {
    let phase = ethernet_phase(builder, inputs, state);
    let pre = preamble_byte(builder, state);
    let header = select_byte(builder, &header_bytes(inputs), &state.byte[..4]);
    let payload = select_byte(builder, &payload_bytes(inputs), &state.byte[..3]);
    let inverted = fcs_bytes(builder, &state.crc);
    let fcs = select_byte(builder, &inverted, &state.byte[..2]);
    let mut value = zero_byte(builder);
    value = phase_byte(builder, value, pre, phase.preamble());
    value = phase_byte(builder, value, header, phase.header());
    value = phase_byte(builder, value, payload, phase.payload());
    phase_byte(builder, value, fcs, phase.fcs())
}

fn phase_byte(
    builder: &mut DigitalBuilder,
    current: [NetId; 8],
    value: [NetId; 8],
    select: NetId,
) -> [NetId; 8] {
    std::array::from_fn(|bit| builder.mux(current[bit], value[bit], select))
}

fn preamble_byte(builder: &mut DigitalBuilder, state: &EthernetState) -> [NetId; 8] {
    let last = equals(builder, &state.byte, 7);
    let one = builder.constant(true);
    let zero = builder.constant(false);
    std::array::from_fn(|bit| {
        let normal = if 0x55 & (1 << bit) != 0 { one } else { zero };
        let sfd = if 0xd5 & (1 << bit) != 0 { one } else { zero };
        builder.mux(normal, sfd, last)
    })
}

fn header_bytes(inputs: EthernetInputs) -> Vec<[NetId; 8]> {
    let mut bytes = chunk_bytes(&inputs.dest, 6);
    bytes.extend(chunk_bytes(&inputs.src, 6));
    bytes.extend(chunk_bytes(&inputs.etype, 2));
    bytes
}

fn payload_bytes(inputs: EthernetInputs) -> Vec<[NetId; 8]> {
    chunk_bytes(&inputs.payload, 8)
}

fn chunk_bytes(bits: &[NetId], count: usize) -> Vec<[NetId; 8]> {
    (0..count)
        .map(|index| std::array::from_fn(|bit| bits[index * 8 + bit]))
        .collect()
}

fn fcs_bytes(builder: &mut DigitalBuilder, crc: &[NetId; 32]) -> Vec<[NetId; 8]> {
    let inverted: [NetId; 32] = std::array::from_fn(|index| builder.not(crc[index]));
    (0..4)
        .map(|byte| std::array::from_fn(|bit| inverted[byte * 8 + bit]))
        .collect()
}

fn select_byte(builder: &mut DigitalBuilder, bytes: &[[NetId; 8]], select: &[NetId]) -> [NetId; 8] {
    let zero = builder.constant(false);
    std::array::from_fn(|bit| {
        let mut level: Vec<NetId> = bytes.iter().map(|byte| byte[bit]).collect();
        while level.len() < (1 << select.len()) {
            level.push(zero);
        }
        mux_tree(builder, level, select)
    })
}

fn zero_byte(builder: &mut DigitalBuilder) -> [NetId; 8] {
    std::array::from_fn(|_| builder.constant(false))
}

fn ethernet_outputs(builder: &mut DigitalBuilder, inputs: EthernetInputs, state: &EthernetState) {
    let phase = ethernet_phase(builder, inputs, state);
    let tx = tx_bit(builder, inputs, state);
    let idle_done = builder.or(phase.idle(), phase.done());
    let tx_en = builder.not(idle_done);
    builder.output("tx", tx);
    builder.output("tx_en", tx_en);
    builder.output("busy", tx_en);
    builder.output("done", phase.done());
}

fn ethernet_store(builder: &mut DigitalBuilder, state: &EthernetState, next: EthernetNext) {
    let one = builder.constant(true);
    register_bits(builder, &state.phase, &next.phase, one);
    register_bits(builder, &state.byte, &next.byte, one);
    register_bits(builder, &state.bit, &next.bit, one);
    register_bits(builder, &state.crc, &next.crc, one);
}

fn const_bit(builder: &mut DigitalBuilder, value: u32, index: usize) -> NetId {
    builder.constant(value & (1 << index) != 0)
}

#[cfg(test)]
mod tests {
    use super::ethernet_entry;

    fn set_bits(entry: &mut super::DigitalEntry, prefix: &str, value: &[u8], count: usize) {
        for index in 0..count {
            let bit = value[index / 8] & (1 << (index % 8)) != 0;
            entry
                .circuit
                .set_input(&format!("{prefix}{index}"), bit)
                .expect("input");
        }
    }

    fn load(entry: &mut super::DigitalEntry) {
        set_bits(entry, "dest", &[0x02, 0, 0, 0, 0, 1], 48);
        set_bits(entry, "src", &[0x02, 0, 0, 0, 0, 2], 48);
        set_bits(entry, "type", &[0x08, 0x00], 16);
        set_bits(
            entry,
            "data",
            &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
            64,
        );
    }

    fn collect_byte(entry: &mut super::DigitalEntry) -> u8 {
        let mut value = 0u8;
        for bit in 0..8 {
            if entry.circuit.output("tx") == Some(true) {
                value |= 1 << bit;
            }
            entry.circuit.tick().expect("bit");
        }
        value
    }

    fn collect(entry: &mut super::DigitalEntry) -> Vec<u8> {
        entry.circuit.set_input("start", true).expect("start");
        entry.circuit.settle().expect("settle");
        entry.circuit.tick().expect("start");
        entry.circuit.set_input("start", false).expect("start");
        let mut bytes = Vec::new();
        for _ in 0..80 {
            if entry.circuit.output("done") == Some(true) {
                break;
            }
            bytes.push(collect_byte(entry));
        }
        bytes
    }

    fn assert_frame(bytes: &[u8]) {
        assert_eq!(bytes.len(), 72);
        assert_eq!(
            &bytes[..8],
            &[0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0xd5]
        );
        assert_eq!(
            &bytes[8..22],
            &[0x02, 0, 0, 0, 0, 1, 0x02, 0, 0, 0, 0, 2, 0x08, 0x00]
        );
        assert_eq!(
            &bytes[22..30],
            &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]
        );
        assert_eq!(&bytes[30..68], &[0u8; 38]);
        assert_eq!(&bytes[68..72], &[0x38, 0xf0, 0x01, 0x66]);
    }

    #[test]
    fn ethernet_frame_has_preamble_padding_and_fcs() {
        let mut entry = ethernet_entry();
        load(&mut entry);
        let bytes = collect(&mut entry);
        assert_frame(&bytes);
        assert_eq!(entry.circuit.output("done"), Some(true));
    }
}

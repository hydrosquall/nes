use crate::common::{Shared, shared, Clocked, CLOCKS_PER_FRAME, SAMPLES_PER_FRAME};
use crate::apu::pulse::Pulse;

mod components;
mod pulse;

const SAMPLE_RATE: f32 = (CLOCKS_PER_FRAME / SAMPLES_PER_FRAME / 2.0) - 1f32;

bitflags! {
    struct EnabledChannels: u8 {
        const PULSE_1 =  0b0000_0001;
        const PULSE_2 =  0b0000_0010;
        const TRIANGLE = 0b0000_0100;
        const NOISE =    0b0000_1000;
        const DMC =      0b0001_0000;
    }
}
bitflags! {
    struct FrameCounter: u8 {
        const FIVE_STEP = 0b1000_0000;
        const IRQ_INHIBIT = 0b0100_0000;
    }
}

trait Channel: Clocked {
    fn set_register(&mut self, addr: u16, value: u8);
    fn sample(&mut self) -> Option<f32>;
}

pub struct Apu {
    cycle: u16,
    sample_step: f32,
    samples: Vec<f32>,
    pulse1: Pulse,
    pulse2: Pulse,
    enabled: EnabledChannels,
    frame_counter: FrameCounter,
}

impl Apu {
    pub fn new() -> Shared<Apu> {
        shared(Apu {
            cycle: 0,
            sample_step: 0f32,
            samples: Vec::with_capacity(SAMPLES_PER_FRAME as usize),
            pulse1: Pulse::default(),
            pulse2: Pulse::default(),
            enabled: EnabledChannels::empty(),
            frame_counter: FrameCounter::empty(),
        })
    }

    fn set_enabled_flags(&mut self, value: u8) {
        self.enabled = EnabledChannels::from_bits_truncate(value);
        if !self.enabled.contains(EnabledChannels::PULSE_1) {
            self.pulse1.length_counter.length = 0;
        }
        if !self.enabled.contains(EnabledChannels::PULSE_2) {
            self.pulse2.length_counter.length = 0;
        }
    }

    pub fn set_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000 ... 0x4003 => self.pulse1.set_register(addr, value),
            0x4004 ... 0x4007 => self.pulse2.set_register(addr, value),
            0x4015 => self.set_enabled_flags(value),
            0x4017 => self.frame_counter = FrameCounter::from_bits_truncate(value),
            _ => warn!("Unimplemented APU register: {:04X} -> {:02X}", addr, value)
        }
    }

    fn sample(&mut self) {
        // https://wiki.nesdev.com/w/index.php/APU_Mixer
        let pulse_1 = match self.enabled.contains(EnabledChannels::PULSE_1) {
            true => self.pulse1.sample(),
            false => None
        }.unwrap_or(0f32);
        let pulse_2 = match self.enabled.contains(EnabledChannels::PULSE_2) {
            true => self.pulse2.sample(),
            false => None
        }.unwrap_or(0f32);
        let triangle = 0f32;
        let noise = 0f32;
        let dmc = 0f32;

        // TODO triangle, noise, dmc
        let pulse = 0.00752 * (pulse_1 + pulse_2);
        let tri_noise_dmc = 0.00851 * triangle + 0.00494 * noise + 0.00335 * dmc;
        self.samples.push(pulse + tri_noise_dmc)
    }

    pub fn samples(&mut self) -> &mut Vec<f32> {
        &mut self.samples
    }

    fn clock_channels(&mut self, half_frame: bool) {
        if half_frame {
            self.pulse1.length_counter.tick();
            self.pulse2.length_counter.tick();
            // clock sweep units
        }
        self.pulse1.envelope.tick();
        self.pulse2.envelope.tick();
        // clock triangle
    }
}

impl Clocked for Apu {
    fn tick(&mut self) {
        // https://wiki.nesdev.com/w/index.php/APU_Frame_Counter
        // I am treating CPU and APU cycles as equivalent, so these are multiplied by 2!
        if (self.cycle & 1) == 0 {
            self.pulse1.tick();
            self.pulse2.tick();
        }
        match self.cycle {
            7457 => self.clock_channels(false),
            14913 => self.clock_channels(true),
            22371 => self.clock_channels(false),
            29828 => {
                if self.frame_counter.bits() == 0 {
                    // irq
                }
            },
            29829 => {
                if !self.frame_counter.contains(FrameCounter::FIVE_STEP) {
                    self.clock_channels(true);
                    self.cycle = 0;
                }
            }
            37281 => {
                if self.frame_counter.contains(FrameCounter::FIVE_STEP) {
                    self.clock_channels(true);
                    self.cycle = 0;
                }
            }
            _ => {}
        }
        if (self.cycle & 1) == 0 {
            if self.sample_step <= 0f32 {
                self.sample();
                self.sample_step += SAMPLE_RATE;
            } else {
                self.sample_step -= 1f32;
            }
        }
        self.cycle += 1;
    }
}

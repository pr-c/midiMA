use std::error::Error;

use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

pub struct MidiController {
    connection_tx: MidiOutputConnection,
}

impl MidiController {
    pub fn new() -> Result<MidiController, Box<dyn Error>> {
        let mut midi_out = MidiOutput::new("MidiMA output")?;
        let port = MidiController::find_midi_input_controller(&mut midi_out)?;

        let connection_tx = midi_out.connect(&port, "MidiMa output")?;

        Ok(MidiController { connection_tx })
    }

    pub fn set_fader_position(&mut self, fader: u8, value: f32) -> Result<(), Box<dyn Error>> {
        if fader > 8 {
            Err("Fader out of bounds")?
        }
        let midi_value: u8 = (value * 127.0) as u8;
        self.connection_tx.send(&[fader + 224, 0, midi_value])?;
        Ok(())
    }

    fn find_midi_input_controller(
        midi_out: &mut MidiOutput,
    ) -> Result<MidiOutputPort, Box<dyn Error>> {
        for port in midi_out.ports() {
            println!("{}", midi_out.port_name(&port)?);
            if midi_out
                .port_name(&port)?
                .eq_ignore_ascii_case("X-TOUCH COMPACT:X-TOUCH COMPACT MIDI 1 28:0")
            {
                return Ok(port);
            }
        }
        Err("The midi input couldn't be found.")?
    }
}

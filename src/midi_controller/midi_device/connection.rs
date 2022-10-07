use std::error::Error;
use std::future;
use futures_util::StreamExt;
use midir::{MidiInput, MidiInputConnection, MidiIO, MidiOutput, MidiOutputConnection};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::UnboundedReceiverStream;
use crate::config::MidiDeviceConfig;
use crate::midi_controller::midi_message::MidiMessage;

pub struct Connection {
    _midi_connection_rx: MidiInputConnection<()>,
    sender_task: JoinHandle<()>,
}

pub struct TxRxChannels {
    pub sender: UnboundedSender<MidiMessage>,
    pub receiver: UnboundedReceiver<MidiMessage>,
}

impl Connection {
    pub fn new(config: &MidiDeviceConfig) -> Result<(Self, TxRxChannels), Box<dyn Error>> {
        let mut midi_out = MidiOutput::new(&("MidiMA out ".to_owned() + &config.midi_out_port_name))?;
        let mut midi_in = MidiInput::new(&("MidiMA in ".to_owned() + &config.midi_in_port_name))?;
        let port_in = Self::find_midi_port(&mut midi_in, &config.midi_in_port_name)?;
        let port_out = Self::find_midi_port(&mut midi_out, &config.midi_out_port_name)?;

        let (midi_rx_sender, midi_rx_receiver) = unbounded_channel();

        let midi_connection_tx = midi_out.connect(&port_out, &config.midi_out_port_name)?;
        let midi_connection_rx = midi_in.connect(&port_in, &config.midi_in_port_name, move |_stamp, message, _| {
            if let Ok(message) = MidiMessage::from_slice(message) {
                let _ = midi_rx_sender.send(message);
            }
        }, ())?;

        let (midi_tx_sender, midi_tx_receiver) = unbounded_channel();
        let sender_task = tokio::spawn(Self::forward_tx_messages(midi_tx_receiver, midi_connection_tx));

        Ok(
            (
                Self {
                    _midi_connection_rx: midi_connection_rx,
                    sender_task,
                },
                TxRxChannels {
                    receiver: midi_rx_receiver,
                    sender: midi_tx_sender,
                }
            )
        )
    }

    fn find_midi_port<T: MidiIO>(midi: &mut T, port_name: &str) -> Result<T::Port, Box<dyn Error>> {
        for port in midi.ports() {
            if midi.port_name(&port)?.eq_ignore_ascii_case(port_name) {
                return Ok(port);
            }
        }
        Err("The midi port couldn't be found.")?
    }

    async fn forward_tx_messages(source: UnboundedReceiver<MidiMessage>, mut sink_connection: MidiOutputConnection) {
        let source_stream = UnboundedReceiverStream::new(source);
        source_stream.for_each(|message| {
            let _ = sink_connection.send(&message.data);
            future::ready(())
        }).await;
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.sender_task.abort();
    }
}
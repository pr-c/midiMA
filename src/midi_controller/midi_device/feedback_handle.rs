use tokio::sync::mpsc::UnboundedSender;
use crate::ma_interface::Update;
use crate::midi_controller::midi_message::MidiMessage;

#[derive(Clone)]
pub struct ModelFeedbackHandle {
    pub ma: UnboundedSender<Update>,
    pub midi: UnboundedSender<MidiMessage>,
}

impl ModelFeedbackHandle {
    pub fn new(ma: UnboundedSender<Update>, midi: UnboundedSender<MidiMessage>) -> Self {
        Self {
            ma,
            midi,
        }
    }
}

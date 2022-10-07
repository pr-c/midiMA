use std::array::TryFromSliceError;

pub struct MidiMessage {
    pub data: [u8; 3],
}


impl MidiMessage {
    pub fn from_slice(slice: &[u8]) -> Result<MidiMessage, TryFromSliceError> {
        let data_result = slice.try_into();
        match data_result {
            Ok(data) => {
                Ok(MidiMessage { data })
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}
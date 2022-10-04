use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::interval;

pub struct PeriodicUpdateSender<T: Copy + Send + 'static> {
    value: Arc<Mutex<Option<T>>>,
    channel: UnboundedSender<T>,
    period: Duration,
    sender_task: Option<JoinHandle<Result<(), ChannelClosedError>>>,
}


impl<T: Copy + Send + 'static> PeriodicUpdateSender<T> {
    pub fn new(channel: UnboundedSender<T>, period: Duration) -> Result<Self, Box<dyn Error>> {
        if channel.is_closed() {
            return Err(Box::new(ChannelClosedError {}));
        }
        Ok(PeriodicUpdateSender {
            channel,
            value: Arc::new(Mutex::new(None)),
            period,
            sender_task: None,
        })
    }

    pub async fn set_value(&mut self, value: T) -> Result<(), Box<dyn Error>> {
        let mut value_lock = self.value.lock().await;
        *value_lock = Some(value);
        drop(value_lock);

        if self.sender_task.is_none() {
            self.start_sender_task();
        } else if self.sender_task.as_ref().unwrap().is_finished() {
            let task = std::mem::replace(&mut self.sender_task, None).unwrap();
            let result = task.await;
            if let Ok(Err(e)) = result {
                return Err(e.into());
            } else {
                self.start_sender_task();
            }
        }
        Ok(())
    }

    fn start_sender_task(&mut self) {
        self.sender_task = Some(tokio::spawn(Self::sender_loop(self.channel.clone(), self.value.clone(), self.period)));
    }

    async fn sender_loop(channel: UnboundedSender<T>, value: Arc<Mutex<Option<T>>>, period: Duration) -> Result<(), ChannelClosedError> {
        let mut interval = interval(period);
        loop {
            interval.tick().await;
            let mut value_lock = value.lock().await;
            if let Some(t) = value_lock.deref() {
                let send_result = channel.send(*t);
                *value_lock = None;
                if send_result.is_err() {
                    return Err(ChannelClosedError {});
                }
            } else {
                return Ok(());
            }
        }
    }
}

impl<T: Copy + Send + 'static> Drop for PeriodicUpdateSender<T> {
    fn drop(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct ChannelClosedError;

impl Display for ChannelClosedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sending channel closed.")
    }
}

impl Error for ChannelClosedError {}
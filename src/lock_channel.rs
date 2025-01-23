use crate::fatal::fatal;
use std::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

pub(crate) struct LockChannel<T> {
    tx: Mutex<Sender<T>>,
    rx: Mutex<Receiver<T>>,
}

impl<T> LockChannel<T> {
    pub(crate) fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<T>();

        Self {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }

    pub(crate) fn emit(&self, value: T) {
        let tx = self
            .tx
            .lock()
            .unwrap_or_else(|err| fatal!("poisoned lock: {:?}", err));

        if let Err(err) = tx.send(value) {
            log::error!("failed to send: {:?}", err);
        }
    }

    pub(crate) fn try_recv(&self) -> Option<T> {
        let rx = self
            .rx
            .lock()
            .unwrap_or_else(|err| fatal!("poisoned lock: {:?}", err));

        rx.try_recv().ok()
    }
}

use crate::Event;
use pyo3::{Bound, IntoPyObject, Python, ffi::PyObject, py_run};

pub(crate) struct Subscriptions {
    list: Vec<*mut PyObject>,
}

impl Subscriptions {
    pub(crate) fn new() -> Self {
        Self { list: vec![] }
    }

    pub(crate) fn push(&mut self, sub: *mut PyObject) {
        self.list.push(sub);
    }

    pub(crate) fn notify_each(&self, event: &Event) {
        for sub in self.list.iter() {
            Python::with_gil(|py| {
                let sub = unsafe { Bound::from_borrowed_ptr(py, *sub) };
                match event.clone().into_pyobject(py) {
                    Ok(event) => py_run!(py, sub event, "sub.on_event(event)"),
                    Err(err) => {
                        log::error!("failed to convert event {event:?} to python: {err:?}")
                    }
                }
            });
        }
    }
}

unsafe impl Send for Subscriptions {}
unsafe impl Sync for Subscriptions {}

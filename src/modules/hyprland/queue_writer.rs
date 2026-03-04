use crate::{
    modules::hyprland::{
        oneshot_writer::OneshotWriter,
        resources::{WriterReply, WriterResource},
    },
    unix_socket::new_unix_socket,
};
use anyhow::Result;
use libc::sockaddr_un;
use std::collections::VecDeque;

pub(crate) struct QueueWriter {
    current: Option<Box<OneshotWriter>>,
    queue: VecDeque<Box<dyn WriterResource>>,
    addr: sockaddr_un,
}

impl QueueWriter {
    pub(crate) fn new(xdg_runtime_dir: String, hyprland_instance_signature: String) -> Self {
        let addr = new_unix_socket(
            format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket.sock").as_bytes(),
        );

        Self {
            current: None,
            queue: VecDeque::new(),
            addr,
        }
    }

    pub(crate) fn enqueue(&mut self, resource: Box<dyn WriterResource>) {
        if self.current.is_none() {
            let mut current = OneshotWriter::new(self.addr, resource);
            current.init();
            self.current = Some(current);
        } else {
            self.queue.push_back(resource);
        }
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Result<Option<WriterReply>> {
        let mut out = None;

        if let Some(current) = self.current.as_mut() {
            if let Some(reply) = current.process(op, res)? {
                out = Some(reply);
                self.current = None;
            }
        }

        if self.current.is_none()
            && let Some(resource) = self.queue.pop_front()
        {
            let mut next = OneshotWriter::new(self.addr, resource);
            next.init();
            self.current = Some(next);
        }

        Ok(out)
    }
}

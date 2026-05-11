use crate::{
    modules::FallibleModule,
    sansio::{Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;

pub(crate) struct DedupModule<M> {
    module: M,
    last_socket: Option<u64>,
    last_connect: Option<u64>,
    last_read: Option<u64>,
    last_write: Option<u64>,
    last_open_at: Option<u64>,
    last_close: Option<u64>,
}

impl<M> DedupModule<M> {
    pub(crate) const fn new(module: M) -> Self {
        Self {
            module,
            last_socket: None,
            last_connect: None,
            last_read: None,
            last_write: None,
            last_open_at: None,
            last_close: None,
        }
    }
}

impl<M> FallibleModule for DedupModule<M>
where
    M: FallibleModule,
{
    const MODULE_ID: ModuleId = M::MODULE_ID;
    type Output = M::Output;

    fn wants(&mut self) -> Option<Wants> {
        let wants = self.module.wants()?;

        match wants {
            Wants::Socket { seq, .. } => {
                if self.last_socket == Some(seq) {
                    None
                } else {
                    self.last_socket = Some(seq);
                    Some(wants)
                }
            }

            Wants::Connect { seq, .. } => {
                if self.last_connect == Some(seq) {
                    None
                } else {
                    self.last_connect = Some(seq);
                    Some(wants)
                }
            }

            Wants::Read { seq, .. } => {
                if self.last_read == Some(seq) {
                    None
                } else {
                    self.last_read = Some(seq);
                    Some(wants)
                }
            }

            Wants::Write { seq, .. } => {
                if self.last_write == Some(seq) {
                    None
                } else {
                    self.last_write = Some(seq);
                    Some(wants)
                }
            }

            Wants::ReadWrite {
                fd,
                readbuf,
                readlen,
                readseq,
                writebuf,
                writelen,
                writeseq,
            } => {
                let read_is_new = self.last_read != Some(readseq);
                if read_is_new {
                    self.last_read = Some(readseq);
                }

                let write_is_new = self.last_write != Some(writeseq);
                if write_is_new {
                    self.last_write = Some(writeseq);
                }

                match (read_is_new, write_is_new) {
                    (true, true) => Some(Wants::ReadWrite {
                        fd,
                        readbuf,
                        readlen,
                        readseq,
                        writebuf,
                        writelen,
                        writeseq,
                    }),
                    (true, false) => Some(Wants::Read {
                        fd,
                        buf: readbuf,
                        len: readlen,
                        seq: readseq,
                    }),
                    (false, true) => Some(Wants::Write {
                        fd,
                        buf: writebuf,
                        len: writelen,
                        seq: writeseq,
                    }),
                    (false, false) => None,
                }
            }

            Wants::OpenAt { seq, .. } => {
                if self.last_open_at == Some(seq) {
                    None
                } else {
                    self.last_open_at = Some(seq);
                    Some(wants)
                }
            }

            Wants::Close { seq, .. } => {
                if self.last_close == Some(seq) {
                    None
                } else {
                    self.last_close = Some(seq);
                    Some(wants)
                }
            }
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        self.module.try_satisfy(satisfy, res)
    }

    fn try_tick(&mut self, tick: u64) -> Result<()> {
        self.module.try_tick(tick)
    }
}

impl<M> std::ops::Deref for DedupModule<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl<M> std::ops::DerefMut for DedupModule<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.module
    }
}

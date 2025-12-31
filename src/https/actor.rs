use super::{Response, connection::HttpsConnection};
use crate::{
    Event,
    liburing::{Actor, Cqe, IoUring, Pending},
};
use anyhow::Result;

pub(crate) struct HttpsActor {
    conn: HttpsConnection,
}

impl HttpsActor {
    pub(crate) fn get(
        hostname: &str,
        port: u16,
        path: &str,
        socket_user_data: u64,
        connect_user_data: u64,
        read_user_data: u64,
        write_user_data: u64,
        close_user_data: u64,
    ) -> Result<Self> {
        let conn = HttpsConnection::get(
            hostname,
            port,
            path,
            socket_user_data,
            connect_user_data,
            read_user_data,
            write_user_data,
            close_user_data,
        )?;
        Ok(Self { conn })
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.conn.is_closed()
    }

    pub(crate) fn drain(
        &mut self,
        ring: &mut IoUring,
        pending: &mut Pending,
    ) -> Result<(bool, Option<Response>)> {
        if self.conn.is_closed() {
            return Ok((false, None));
        }

        self.conn.push_sqes(ring, pending)
    }

    pub(crate) fn feed(&mut self, _ring: &mut IoUring, cqe: Cqe) -> Result<()> {
        self.conn.process_cqe(cqe)?;
        Ok(())
    }
}

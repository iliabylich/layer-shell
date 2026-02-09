mod name;
mod request;
mod response;

use libc::{AF_INET, SOCK_DGRAM, in_addr, sockaddr, sockaddr_in};
use request::Request;
use response::Response;

use crate::{
    liburing::IoUring,
    macros::define_op,
    user_data::{ModuleId, UserData},
};

const DNS_SERVER: u32 = 0x08_08_08_08;
const DNS_PORT: u16 = 53;
const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;
const MAX_DNS_PACKET: usize = 512;

pub(crate) struct DnsResolver {
    addr: sockaddr_in,
    fd: i32,
    healthy: bool,
    module_id: ModuleId,

    request: Request,
    reply: [u8; MAX_DNS_PACKET],

    answer: Option<sockaddr_in>,
}

define_op!("DNS Resolver", Socket, Connect, Write, Read, Close);

impl DnsResolver {
    pub(crate) fn new(module_id: ModuleId, domain: &'static [u8]) -> Box<Self> {
        Box::new(Self {
            addr: unsafe { std::mem::zeroed() },
            fd: -1,
            healthy: true,
            module_id,

            request: Request::new(domain, TYPE_A, 0xABCD),
            reply: [0; _],

            answer: None,
        })
    }

    fn schedule_socket(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_socket(AF_INET, SOCK_DGRAM, 0, 0);
        sqe.set_user_data(UserData::new(self.module_id, Op::Socket));
    }

    fn schedule_connect(&mut self) {
        self.addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: DNS_PORT.to_be(),
            sin_addr: in_addr {
                s_addr: DNS_SERVER.to_be(),
            },
            sin_zero: [0; 8],
        };

        let mut sqe = IoUring::get_sqe();
        sqe.prep_connect(
            self.fd,
            (&self.addr as *const sockaddr_in).cast::<sockaddr>(),
            std::mem::size_of::<libc::sockaddr_in>() as u32,
        );
        sqe.set_user_data(UserData::new(self.module_id, Op::Connect));
    }

    fn schedule_write(&self) {
        let mut sqe = IoUring::get_sqe();
        let buf = self.request.as_bytes();
        sqe.prep_write(self.fd, buf.as_ptr(), buf.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::Write));
    }

    fn schedule_read(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.reply.as_mut_ptr(), MAX_DNS_PACKET);
        sqe.set_user_data(UserData::new(self.module_id, Op::Read));
    }

    fn schedule_close(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_close(self.fd);
        sqe.set_user_data(UserData::new(self.module_id, Op::Close));
    }

    pub(crate) fn init(&self) {
        self.schedule_socket();
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Option<sockaddr_in> {
        let op = Op::from(op);

        macro_rules! assert_or_unhealthy {
            ($cond:expr, $($arg:tt)*) => {
                if !$cond {
                    log::error!("DnsResolver({:?})::{op:?}", self.module_id);
                    log::error!($($arg)*);
                    self.healthy = false;
                    return None;
                }
            };
        }

        match op {
            Op::Socket => {
                assert_or_unhealthy!(res > 0, "res = {res}");
                self.fd = res;
                self.schedule_connect();
                None
            }
            Op::Connect => {
                assert_or_unhealthy!(res >= 0, "res = {res}");
                self.schedule_write();
                None
            }
            Op::Write => {
                assert_or_unhealthy!(res > 0, "res = {res}");
                self.schedule_read();
                None
            }
            Op::Read => {
                assert_or_unhealthy!(res > 0, "res = {res}");
                let len = res as usize;
                self.answer = Response::parse(&self.reply[..len]);
                self.schedule_close();
                None
            }
            Op::Close => self.answer.take(),
        }
    }
}

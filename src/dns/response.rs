use crate::dns::{TYPE_A, name::DnsName};
use libc::sockaddr_in;

pub(crate) struct Response;

impl Response {
    pub(crate) fn parse(data: &[u8]) -> Option<sockaddr_in> {
        let mut p = ResponseParser { data, pos: 0 };

        let _id = p.read_u16();
        let flags = p.read_u16();
        let qdcount = p.read_u16();
        let ancount = p.read_u16();
        let _nscount = p.read_u16();
        let _arcount = p.read_u16();

        if flags & 0x8000 == 0 {
            return None;
        }
        if flags & 0x000F != 0 {
            return None;
        }

        for _ in 0..qdcount {
            let _ = p.read_name();
            p.skip(4);
        }

        for _ in 0..ancount {
            let _name = p.read_name();
            let rtype = p.read_u16();
            let _rclass = p.read_u16();
            let _ttl = p.read_u32();
            let rdlength = p.read_u16();

            if rtype == TYPE_A && rdlength == 4 {
                return Some(sockaddr_in {
                    sin_family: libc::AF_INET as libc::sa_family_t,
                    sin_port: 0,
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(p.read_bytes()),
                    },
                    sin_zero: [0; 8],
                });
            } else {
                p.skip(rdlength as usize);
            }
        }

        None
    }
}

struct ResponseParser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ResponseParser<'a> {
    fn read_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut buf = [0u8; N];
        buf.copy_from_slice(&self.data[self.pos..self.pos + N]);
        self.pos += N;
        buf
    }

    fn read_u16(&mut self) -> u16 {
        u16::from_be_bytes(self.read_bytes())
    }

    fn read_u32(&mut self) -> u32 {
        u32::from_be_bytes(self.read_bytes())
    }

    fn label_at(&self, pos: usize) -> &[u8] {
        let len = self.data[pos] as usize;
        &self.data[pos + 1..pos + 1 + len]
    }

    fn read_name(&mut self) -> DnsName {
        let mut name = DnsName::new();

        loop {
            match self.data[self.pos] {
                0 => {
                    self.pos += 1;
                    break;
                }
                byte if byte & 0xC0 == 0xC0 => {
                    let offset = ((byte & 0x3F) as usize) << 8 | self.data[self.pos + 1] as usize;
                    self.pos += 2;
                    let mut pos = offset;
                    loop {
                        let len = self.data[pos] as usize;
                        if len == 0 {
                            break;
                        }
                        name.push_label(self.label_at(pos));
                        pos += 1 + len;
                    }
                    break;
                }
                len => {
                    self.pos += 1;
                    name.push_label(&self.data[self.pos..self.pos + len as usize]);
                    self.pos += len as usize;
                }
            }
        }

        name
    }

    fn skip(&mut self, n: usize) {
        self.pos += n;
    }
}

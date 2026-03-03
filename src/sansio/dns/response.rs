use crate::sansio::dns::{TYPE_A, name::DnsName};
use anyhow::{Result, bail};
use libc::sockaddr_in;

pub(crate) struct Response;

impl Response {
    pub(crate) fn read(buf: &[u8]) -> Result<sockaddr_in> {
        let mut pos = 0;

        let _id = read_u16(buf, &mut pos);
        let flags = read_u16(buf, &mut pos);
        let qdcount = read_u16(buf, &mut pos);
        let ancount = read_u16(buf, &mut pos);
        let _nscount = read_u16(buf, &mut pos);
        let _arcount = read_u16(buf, &mut pos);

        if flags & 0x8000 == 0 {
            bail!("invalid DNS response");
        }
        if flags & 0x000F != 0 {
            bail!("invalid DNS response");
        }

        for _ in 0..qdcount {
            let _ = read_name(buf, &mut pos);
            pos += 4;
        }

        for _ in 0..ancount {
            let _name = read_name(buf, &mut pos);
            let rtype = read_u16(buf, &mut pos);
            let _rclass = read_u16(buf, &mut pos);
            let _ttl = read_u32(buf, &mut pos);
            let rdlength = read_u16(buf, &mut pos);

            if rtype == TYPE_A && rdlength == 4 {
                return Ok(sockaddr_in {
                    sin_family: libc::AF_INET as libc::sa_family_t,
                    sin_port: 0,
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(read_bytes(buf, &mut pos)),
                    },
                    sin_zero: [0; 8],
                });
            } else {
                pos += rdlength as usize;
            }
        }

        bail!("no A record in the DNS response")
    }
}

fn read_bytes<const N: usize>(buf: &[u8], pos: &mut usize) -> [u8; N] {
    let mut out = [0u8; N];
    out.copy_from_slice(&buf[*pos..*pos + N]);
    *pos += N;
    out
}

fn read_u16(buf: &[u8], pos: &mut usize) -> u16 {
    u16::from_be_bytes(read_bytes(buf, pos))
}

fn read_u32(buf: &[u8], pos: &mut usize) -> u32 {
    u32::from_be_bytes(read_bytes(buf, pos))
}

fn label_at(buf: &[u8], pos: usize) -> &[u8] {
    let len = buf[pos] as usize;
    &buf[pos + 1..pos + 1 + len]
}

fn read_name(buf: &[u8], pos: &mut usize) -> DnsName {
    let mut name = DnsName::new();

    loop {
        match buf[*pos] {
            0 => {
                *pos += 1;
                break;
            }
            byte if byte & 0xC0 == 0xC0 => {
                let offset = ((byte & 0x3F) as usize) << 8 | buf[*pos + 1] as usize;
                *pos += 2;
                let mut pos = offset;
                loop {
                    let len = buf[pos] as usize;
                    if len == 0 {
                        break;
                    }
                    name.push_label(label_at(buf, pos));
                    pos += 1 + len;
                }
                break;
            }
            len => {
                *pos += 1;
                name.push_label(&buf[*pos..*pos + len as usize]);
                *pos += len as usize;
            }
        }
    }

    name
}

use crate::sansio::dns::{TYPE_A, name::DnsName};
use anyhow::{Context as _, Result, bail};
use libc::sockaddr_in;

pub(crate) struct Response;

impl Response {
    pub(crate) fn read(buf: &[u8]) -> Result<sockaddr_in> {
        let mut pos = 0;

        let _id = read_u16(buf, &mut pos)?;
        let flags = read_u16(buf, &mut pos)?;
        let qdcount = read_u16(buf, &mut pos)?;
        let ancount = read_u16(buf, &mut pos)?;
        let _nscount = read_u16(buf, &mut pos)?;
        let _arcount = read_u16(buf, &mut pos)?;

        if flags & 0x8000 == 0 || flags & 0x000F != 0 {
            bail!("invalid DNS response");
        }

        for _ in 0..qdcount {
            let _ = read_name(buf, &mut pos)?;
            let _ = read_bytes::<4>(buf, &mut pos)?;
        }

        for _ in 0..ancount {
            let _name = read_name(buf, &mut pos)?;
            let rtype = read_u16(buf, &mut pos)?;
            let _rclass = read_u16(buf, &mut pos)?;
            let _ttl = read_u32(buf, &mut pos)?;
            let rdlength = read_u16(buf, &mut pos)?;

            if rtype == TYPE_A && rdlength == 4 {
                return Ok(sockaddr_in {
                    sin_family: libc::AF_INET as libc::sa_family_t,
                    sin_port: 0,
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(read_bytes(buf, &mut pos)?),
                    },
                    sin_zero: [0; 8],
                });
            } else {
                let _ = read_slice(buf, &mut pos, rdlength as usize)?;
            }
        }

        bail!("no A record in the DNS response")
    }
}

fn read_bytes<const N: usize>(buf: &[u8], pos: &mut usize) -> Result<[u8; N]> {
    let end = *pos + N;
    let bytes = buf
        .get(*pos..end)
        .context("malformed DNS response: truncated packet")?;
    let mut out = [0u8; N];
    out.copy_from_slice(bytes);
    *pos = end;
    Ok(out)
}

fn read_u16(buf: &[u8], pos: &mut usize) -> Result<u16> {
    Ok(u16::from_be_bytes(read_bytes(buf, pos)?))
}

fn read_u32(buf: &[u8], pos: &mut usize) -> Result<u32> {
    Ok(u32::from_be_bytes(read_bytes(buf, pos)?))
}

fn label_at(buf: &[u8], pos: usize) -> Result<&[u8]> {
    let len = *buf
        .get(pos)
        .context("malformed DNS response: truncated label")? as usize;
    buf.get(pos + 1..pos + 1 + len)
        .context("malformed DNS response: truncated label")
}

fn read_name(buf: &[u8], pos: &mut usize) -> Result<DnsName> {
    let mut name = DnsName::new();

    loop {
        let byte = *buf
            .get(*pos)
            .context("malformed DNS response: truncated name")?;
        match byte {
            0 => {
                *pos += 1;
                break;
            }
            byte if byte & 0xC0 == 0xC0 => {
                let low = *buf
                    .get(*pos + 1)
                    .context("malformed DNS response: truncated compression pointer")?;
                let offset = ((byte & 0x3F) as usize) << 8 | low as usize;
                *pos += 2;
                let mut pos = offset;
                loop {
                    let len = *buf
                        .get(pos)
                        .context("malformed DNS response: bad compression pointer")?
                        as usize;
                    if len == 0 {
                        break;
                    }
                    name.push_label(label_at(buf, pos)?);
                    pos += 1 + len;
                }
                break;
            }
            len => {
                *pos += 1;
                let label = buf
                    .get(*pos..*pos + len as usize)
                    .context("malformed DNS response: truncated name label")?;
                name.push_label(label);
                *pos += len as usize;
            }
        }
    }

    Ok(name)
}

fn read_slice<'a>(buf: &'a [u8], pos: &mut usize, len: usize) -> Result<&'a [u8]> {
    let end = *pos + len;
    let out = buf
        .get(*pos..end)
        .context("malformed DNS response: truncated rdata")?;
    *pos = end;
    Ok(out)
}

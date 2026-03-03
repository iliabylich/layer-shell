use crate::sansio::dns::{CLASS_IN, MAX_DNS_PACKET};

pub(crate) struct Request;

impl Request {
    pub(crate) fn write(buf: &mut [u8; MAX_DNS_PACKET], domain: &[u8], qtype: u16) -> usize {
        let mut len = 0;

        // https://datatracker.ietf.org/doc/html/rfc1035

        // ID
        write_u16(buf, &mut len, 0xABCD);

        // QR + Opcode + AA + TC + RD + RA + Z + RCODE
        // Query + Recursion Desired
        write_u16(buf, &mut len, 0x0100);

        // QDCOUNT
        write_u16(buf, &mut len, 1);

        // ANCOUNT
        write_u16(buf, &mut len, 0);

        // NSCOUNT
        write_u16(buf, &mut len, 0);

        // ARCOUNT
        write_u16(buf, &mut len, 0);

        // QNAME
        for label in domain.split(|byte| *byte == b'.') {
            write_u8(buf, &mut len, label.len() as u8);
            for byte in label {
                write_u8(buf, &mut len, *byte);
            }
        }
        write_u8(buf, &mut len, 0);

        // QTYPE
        write_u16(buf, &mut len, qtype);

        // QCLASS
        write_u16(buf, &mut len, CLASS_IN);

        len
    }
}

fn write_u8(buf: &mut [u8; MAX_DNS_PACKET], len: &mut usize, byte: u8) {
    buf[*len] = byte;
    *len += 1;
}

fn write_u16(buf: &mut [u8; MAX_DNS_PACKET], len: &mut usize, dbyte: u16) {
    let bytes = dbyte.to_be_bytes();
    buf[*len] = bytes[0];
    buf[*len + 1] = bytes[1];
    *len += 2;
}

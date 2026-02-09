use crate::dns::{CLASS_IN, MAX_DNS_PACKET};

#[derive(Debug)]
pub(crate) struct Request {
    buf: [u8; MAX_DNS_PACKET],
    len: usize,
}

impl Request {
    pub(crate) fn new(domain: &[u8], qtype: u16, txid: u16) -> Self {
        let mut packet = Request {
            buf: [0u8; MAX_DNS_PACKET],
            len: 0,
        };

        // https://datatracker.ietf.org/doc/html/rfc1035

        // ID
        packet.write_u16(txid);

        // QR + Opcode + AA + TC + RD + RA + Z + RCODE
        // Query + Recursion Desired
        packet.write_u16(0x0100);

        // QDCOUNT
        packet.write_u16(1);

        // ANCOUNT
        packet.write_u16(0);

        // NSCOUNT
        packet.write_u16(0);

        // ARCOUNT
        packet.write_u16(0);

        // QNAME
        for label in domain.split(|byte| *byte == b'.') {
            packet.write_u8(label.len() as u8);
            for byte in label {
                packet.write_u8(*byte);
            }
        }
        packet.write_u8(0);

        // QTYPE
        packet.write_u16(qtype);

        // QCLASS
        packet.write_u16(CLASS_IN);

        packet
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    fn write_u8(&mut self, val: u8) {
        self.buf[self.len] = val;
        self.len += 1;
    }

    fn write_u16(&mut self, val: u16) {
        let bytes = val.to_be_bytes();
        self.buf[self.len] = bytes[0];
        self.buf[self.len + 1] = bytes[1];
        self.len += 2;
    }
}

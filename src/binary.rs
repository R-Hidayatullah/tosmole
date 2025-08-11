use std::io::{self, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};

#[derive(Copy, Clone, PartialEq)]
pub enum Endian {
    Little,
    Big,
}

pub struct BinaryReader<R: Read + Seek> {
    inner: BufReader<R>,
    default_endian: Endian,
}

pub struct BinaryWriter<W: Write + Seek> {
    inner: BufWriter<W>,
    default_endian: Endian,
}

impl<R: Read + Seek> BinaryReader<R> {
    pub fn new(reader: R, endian: Endian) -> Self {
        Self {
            inner: BufReader::new(reader),
            default_endian: endian,
        }
    }

    pub fn read_exact<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let mut buf = [0u8; N];
        self.inner.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn read_with_endian<const N: usize>(&mut self, endian: Endian) -> io::Result<[u8; N]> {
        let mut buf = self.read_exact::<N>()?;
        if endian != self.default_endian {
            buf.reverse();
        }
        Ok(buf)
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        Ok(self.read_exact::<1>()?[0])
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        self.read_u16_with(self.default_endian)
    }

    pub fn read_u16_with(&mut self, endian: Endian) -> io::Result<u16> {
        let buf = self.read_with_endian::<2>(endian)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        self.read_u32_with(self.default_endian)
    }

    pub fn read_u32_with(&mut self, endian: Endian) -> io::Result<u32> {
        let buf = self.read_with_endian::<4>(endian)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_f32(&mut self) -> io::Result<f32> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub fn read_f32_with(&mut self, endian: Endian) -> io::Result<f32> {
        Ok(f32::from_bits(self.read_u32_with(endian)?))
    }

    pub fn read_string(&mut self, len: usize) -> io::Result<String> {
        let mut buf = vec![0u8; len];
        self.inner.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }

    pub fn read_vec(&mut self, len: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.inner.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }

    pub fn position(&mut self) -> io::Result<u64> {
        self.inner.stream_position()
    }
}

impl<W: Write + Seek> BinaryWriter<W> {
    pub fn new(writer: W, endian: Endian) -> Self {
        Self {
            inner: BufWriter::new(writer),
            default_endian: endian,
        }
    }

    fn write_with_endian<const N: usize>(
        &mut self,
        mut buf: [u8; N],
        endian: Endian,
    ) -> io::Result<()> {
        if endian != self.default_endian {
            buf.reverse();
        }
        self.inner.write_all(&buf)
    }

    pub fn write_u8(&mut self, value: u8) -> io::Result<()> {
        self.inner.write_all(&[value])
    }

    pub fn write_u16(&mut self, value: u16) -> io::Result<()> {
        self.write_u16_with(value, self.default_endian)
    }

    pub fn write_u16_with(&mut self, value: u16, endian: Endian) -> io::Result<()> {
        self.write_with_endian(value.to_le_bytes(), endian)
    }

    pub fn write_u32(&mut self, value: u32) -> io::Result<()> {
        self.write_u32_with(value, self.default_endian)
    }

    pub fn write_u32_with(&mut self, value: u32, endian: Endian) -> io::Result<()> {
        self.write_with_endian(value.to_le_bytes(), endian)
    }

    pub fn write_f32(&mut self, value: f32) -> io::Result<()> {
        self.write_u32(value.to_bits())
    }

    pub fn write_f32_with(&mut self, value: f32, endian: Endian) -> io::Result<()> {
        self.write_u32_with(value.to_bits(), endian)
    }

    pub fn write_string(&mut self, s: &str) -> io::Result<()> {
        self.inner.write_all(s.as_bytes())
    }

    pub fn write_vec(&mut self, data: &[u8]) -> io::Result<()> {
        self.inner.write_all(data)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    pub fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }

    pub fn position(&mut self) -> io::Result<u64> {
        self.inner.stream_position()
    }
}

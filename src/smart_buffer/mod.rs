use anyhow::{anyhow, bail, Result};

pub struct SmartBuffer<T: AsRef<[u8]>> {
    buf: T,
    pos: usize,
}

const BUFFER_OVERFLOW_ERROR_MSG: &str = "buffer overflow";
const POSSIBLY_LOOP_IN_QUESTION_ERROR_MSG: &str = "possibly loop in the question";
const WRONG_LABEL_LENGTH: &str = "length of label is greater than 63 bytes";

impl<T: AsRef<[u8]>> SmartBuffer<T> {
    pub fn new(buf: T) -> Self {
        Self { buf, pos: 0 }
    }

    fn get(&mut self, pos: usize) -> Result<u8> {
        self.buf
            .as_ref()
            .get(pos)
            .cloned()
            .ok_or(anyhow!(BUFFER_OVERFLOW_ERROR_MSG))
    }

    fn get_slice(&mut self, start: usize, len: usize) -> Result<&[u8]> {
        if start + len >= self.buf.as_ref().len() {
            bail!(BUFFER_OVERFLOW_ERROR_MSG);
        }

        Ok(&self.buf.as_ref()[start..start + len])
    }

    fn next_byte(&mut self) -> Result<u8> {
        let byte = *self
            .buf
            .as_ref()
            .get(self.pos)
            .ok_or(anyhow!(BUFFER_OVERFLOW_ERROR_MSG))?;
        self.pos += 1;
        Ok(byte)
    }

    fn seek(&mut self, pos: usize) -> Result<()> {
        if pos >= self.buf.as_ref().len() {
            bail!(BUFFER_OVERFLOW_ERROR_MSG);
        }

        self.pos = pos;

        Ok(())
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(((self.next_byte()? as u16) << 8) | (self.next_byte()? as u16))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(((self.next_byte()? as u32) << 24)
            | ((self.next_byte()? as u32) << 16)
            | ((self.next_byte()? as u32) << 8)
            | (self.next_byte()? as u32))
    }

    pub fn read_slice(&mut self, len: usize) -> Result<&[u8]> {
        if self.pos + len > self.buf.as_ref().len() {
            bail!(BUFFER_OVERFLOW_ERROR_MSG);
        }

        let res = &self.buf.as_ref()[self.pos..self.pos + len];
        self.pos += len;

        Ok(res)
    }

    pub fn read_qname(&mut self) -> Result<String> {
        let mut result = String::with_capacity(16);

        let mut pos = self.pos;

        let mut jump = false;
        let mut jumps_cnt = 0;

        loop {
            if jumps_cnt > 10 {
                bail!(POSSIBLY_LOOP_IN_QUESTION_ERROR_MSG);
            }

            let len = self.get(pos)?;
            pos += 1;

            if (len & 0xC0) == 0xC0 {
                if !jump {
                    self.seek(pos + 1)?;
                }
                pos = ((((len as u16) ^ 0xC0) << 8) | (self.get(pos)? as u16)) as usize;

                jump = true;
                jumps_cnt += 1;

                continue;
            } else {
                if len == 0 {
                    break;
                }

                if !result.is_empty() {
                    // delimiter
                    result.push('.');
                }

                result.push_str(
                    &String::from_utf8_lossy(self.get_slice(pos, len as usize)?).to_lowercase(),
                );

                pos += len as usize;
            }
        }

        if !jump {
            self.seek(pos)?;
        }

        Ok(result)
    }
}

impl<T: AsMut<[u8]> + AsRef<[u8]>> SmartBuffer<T> {
    fn write_byte(&mut self, byte: u8) -> Result<()> {
        if self.pos >= self.buf.as_ref().len() {
            bail!(BUFFER_OVERFLOW_ERROR_MSG)
        }

        self.buf.as_mut()[self.pos] = byte;
        self.pos += 1;

        Ok(())
    }

    pub fn write_u8(&mut self, val: u8) -> Result<()> {
        self.write_byte(val)?;

        Ok(())
    }

    pub fn write_u16(&mut self, val: u16) -> Result<()> {
        self.write_byte((val >> 8) as u8)?;
        self.write_byte((val & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_u32(&mut self, val: u32) -> Result<()> {
        self.write_byte(((val >> 24) & 0xFF) as u8)?;
        self.write_byte(((val >> 16) & 0xFF) as u8)?;
        self.write_byte(((val >> 8) & 0xFF) as u8)?;
        self.write_byte((val & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_slice<S: AsRef<[u8]>>(&mut self, slice: S) -> Result<()> {
        for byte in slice.as_ref().iter() {
            self.write_byte(*byte)?;
        }

        Ok(())
    }

    pub fn write_qname<S: AsRef<str>>(&mut self, qname: S) -> Result<()> {
        for label in qname.as_ref().split('.') {
            if label.len() > 63 {
                bail!(WRONG_LABEL_LENGTH)
            }

            self.write_u8(label.len() as u8)?;
            for byte in label.as_bytes() {
                self.write_u8(*byte)?;
            }
        }

        self.write_u8(0)?;

        Ok(())
    }
}

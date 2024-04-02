use crate::models::enums::{MessageType, OpCode, ResultCode};
use crate::smart_buffer::SmartBuffer;
use anyhow::Result;

#[derive(Debug)]
pub struct Header {
    pub(in crate::models) id: u16,
    pub(in crate::models) message_type: MessageType,
    pub(in crate::models) opcode: OpCode,
    pub(in crate::models) authoritative_answer: bool,
    pub(in crate::models) truncation: bool,
    pub(in crate::models) recursion_desired: bool,
    pub(in crate::models) recursion_available: bool,
    pub(in crate::models) result_code: ResultCode,
    pub(in crate::models) question_entities_count: u16,
    pub(in crate::models) answer_entities_count: u16,
    pub(in crate::models) authority_entities_count: u16,
    pub(in crate::models) additional_entities_count: u16,
}

impl Header {
    pub(in crate::models) fn from_bytes<T: AsRef<[u8]>>(
        smart_buf: &mut SmartBuffer<T>,
    ) -> Result<Self> {
        let id = smart_buf.read_u16()?;

        let flags = smart_buf.read_u16()?;

        let message_type = if (flags & (1 << 15)) > 0 {
            MessageType::Response
        } else {
            MessageType::Query
        };
        let opcode = OpCode::from(((flags >> 11) & 0x0F) as u8);
        let authoritative_answer = (flags & (1 << 10)) > 0;
        let truncation = (flags & (1 << 9)) > 0;
        let recursion_desired = (flags & (1 << 8)) > 0;
        let recursion_available = (flags & (1 << 7)) > 0;
        let result_code = ResultCode::try_from((flags & 0x0F) as u8)?;

        // z == flags & (0x7 << 4)

        let question_entities_count = smart_buf.read_u16()?;
        let answer_entities_count = smart_buf.read_u16()?;
        let authority_entities_count = smart_buf.read_u16()?;
        let additional_entities_count = smart_buf.read_u16()?;

        Ok(Self {
            id,
            message_type,
            opcode,
            authoritative_answer,
            truncation,
            recursion_desired,
            recursion_available,
            result_code,
            question_entities_count,
            answer_entities_count,
            authority_entities_count,
            additional_entities_count,
        })
    }

    pub(in crate::models) fn to_bytes<T: AsMut<[u8]> + AsRef<[u8]>>(
        &self,
        smart_buf: &mut SmartBuffer<T>,
    ) -> Result<()> {
        smart_buf.write_u16(self.id)?;

        smart_buf.write_u8(
            (self.recursion_desired as u8)
                | ((self.truncation as u8) << 1)
                | ((self.authoritative_answer as u8) << 2)
                | (u8::from(self.opcode) << 3)
                | ((self.message_type as u8) << 7),
        )?;

        smart_buf.write_u8((self.result_code as u8) | ((self.recursion_available as u8) << 7))?;

        smart_buf.write_u16(self.question_entities_count)?;
        smart_buf.write_u16(self.answer_entities_count)?;
        smart_buf.write_u16(self.authority_entities_count)?;
        smart_buf.write_u16(self.additional_entities_count)?;

        Ok(())
    }

    pub fn question_entities_count(&self) -> u16 {
        self.question_entities_count
    }

    pub fn answer_entities_count(&self) -> u16 {
        self.answer_entities_count
    }

    pub fn authority_entities_count(&self) -> u16 {
        self.authority_entities_count
    }

    pub fn additional_entities_count(&self) -> u16 {
        self.additional_entities_count
    }
}

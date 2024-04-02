use crate::models::header::Header;
use crate::models::question::Question;
use crate::models::record::RawRecord;
use crate::models::ResultCode;
use crate::smart_buffer::SmartBuffer;
use anyhow::Result;

#[derive(Debug)]
pub struct DnsPacket {
    pub(in crate::models) meta: DnsPacketMeta,
    pub(in crate::models) base: DnsPacketBase,
}

#[derive(Debug)]
pub struct DnsPacketMeta {
    pub(in crate::models) header: Header,
    pub(in crate::models) questions: Vec<Question>,
}

#[derive(Clone, Debug)]
pub struct DnsPacketBase {
    pub(in crate::models) answers: Vec<RawRecord>,
    pub(in crate::models) authorities: Vec<RawRecord>,
    pub(in crate::models) additional: Vec<RawRecord>,
}

impl DnsPacket {
    pub fn from_bytes<B: AsRef<[u8]>>(buf: B) -> Result<Self> {
        let buf = &buf.as_ref();
        let mut smart_buf = SmartBuffer::new(buf);

        let header = Header::from_bytes(&mut smart_buf)?;

        let mut questions = Vec::with_capacity(header.question_entities_count() as usize);
        for _ in 0..header.question_entities_count() {
            let question = Question::from_bytes(&mut smart_buf)?;
            questions.push(question);
        }

        let mut answers = Vec::with_capacity(header.answer_entities_count() as usize);
        for _ in 0..header.answer_entities_count() {
            let record = RawRecord::from_bytes(&mut smart_buf)?;
            answers.push(record);
        }

        let mut authorities = Vec::with_capacity(header.authority_entities_count() as usize);
        for _ in 0..header.authority_entities_count() {
            let record = RawRecord::from_bytes(&mut smart_buf)?;
            authorities.push(record);
        }

        let mut additional = Vec::with_capacity(header.additional_entities_count() as usize);
        for _ in 0..header.additional_entities_count() {
            let record = RawRecord::from_bytes(&mut smart_buf)?;
            additional.push(record);
        }

        Ok(Self {
            meta: DnsPacketMeta { header, questions },
            base: DnsPacketBase {
                answers,
                authorities,
                additional,
            },
        })
    }

    pub fn to_bytes<B: AsMut<[u8]> + AsRef<[u8]>>(&self, buf: B) -> Result<()> {
        let mut smart_buf = SmartBuffer::new(buf);

        self.meta.header.to_bytes(&mut smart_buf)?;

        for question in &self.meta.questions {
            question.to_bytes(&mut smart_buf)?;
        }

        for answer in &self.base.answers {
            answer.to_bytes(&mut smart_buf)?;
        }

        for authority in &self.base.authorities {
            authority.to_bytes(&mut smart_buf)?;
        }

        for additional in &self.base.additional {
            additional.to_bytes(&mut smart_buf)?;
        }

        Ok(())
    }

    pub fn id(&self) -> u16 {
        self.meta.header.id
    }

    pub fn questions(&self) -> &[Question] {
        self.meta.questions.as_slice()
    }

    pub fn recursion_desired(&self) -> bool {
        self.meta.header.recursion_desired
    }

    pub fn min_ttl(&self) -> Option<u32> {
        let ttl1 = self.base.answers.iter().map(|a| a.ttl).min();
        let ttl2 = self.base.authorities.iter().map(|a| a.ttl).min();
        let ttl3 = self.base.additional.iter().map(|a| a.ttl).min();

        if let Some(default) = ttl1.or(ttl2).or(ttl3) {
            let ttl1 = ttl1.unwrap_or(default);
            let ttl2 = ttl2.unwrap_or(default);
            let ttl3 = ttl3.unwrap_or(default);

            Some(ttl1.min(ttl2).min(ttl3))
        } else {
            None
        }
    }

    pub fn base(&self) -> &DnsPacketBase {
        &self.base
    }

    pub fn result_code(&self) -> ResultCode {
        self.meta.header.result_code
    }
}

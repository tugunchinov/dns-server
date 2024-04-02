use crate::models::enums::{MessageType, OpCode, QueryClass, QueryType, ResultCode};
use crate::models::header::Header;
use crate::models::packet::{DnsPacketBase, DnsPacketMeta};
use crate::models::question::Question;
use crate::models::record::RawRecord;
use crate::models::DnsPacket;
use anyhow::{anyhow, Result};
use rand::random;

#[derive(Default)]
pub struct DnsPacketBuilder {
    id: Option<u16>,
    recursion_desired: bool,
    recursion_available: bool,
    result_code: Option<ResultCode>,
    message_type: Option<MessageType>,
    questions: Vec<Question>,
    base: Option<DnsPacketBase>,
    answers: Vec<RawRecord>,
    authorities: Vec<RawRecord>,
    additional: Vec<RawRecord>,
}

pub struct RawRecordBuilder<S: Into<String>> {
    packet_builder: DnsPacketBuilder,
    name: Option<S>,
    query_type: Option<QueryType>,
    query_class: Option<QueryClass>,
    ttl: Option<u32>,
    rdata: Option<Vec<u8>>,
}

pub enum RawRecordType {
    Answer,
    _Authority,
    _Additional,
}

impl<S: Into<String>> RawRecordBuilder<S> {
    pub fn name(mut self, name: S) -> Self {
        self.name = Some(name);
        self
    }

    pub fn query_type(mut self, query_type: QueryType) -> Self {
        self.query_type = Some(query_type);
        self
    }

    pub fn query_class(mut self, query_class: QueryClass) -> Self {
        self.query_class = Some(query_class);
        self
    }

    pub fn ttl(mut self, ttl: u32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn rdata(mut self, rdata: Vec<u8>) -> Self {
        self.rdata = Some(rdata);
        self
    }

    pub fn add_raw_record(mut self, record_type: RawRecordType) -> Result<DnsPacketBuilder> {
        let rdata = self.rdata.unwrap_or_default();

        let raw_record = RawRecord {
            name: self
                .name
                .ok_or(anyhow!("record name can't be empty"))?
                .into(),
            query_type: self.query_type.unwrap_or(QueryType::A),
            query_class: self.query_class.unwrap_or(QueryClass::IN),
            ttl: self.ttl.unwrap_or(15),
            rdata_length: rdata.len() as u16,
            rdata,
        };

        match record_type {
            RawRecordType::Answer => self.packet_builder.answers.push(raw_record),
            RawRecordType::_Authority => self.packet_builder.authorities.push(raw_record),
            RawRecordType::_Additional => self.packet_builder.additional.push(raw_record),
        }

        Ok(self.packet_builder)
    }
}

// TODO: remove
#[allow(dead_code)]
#[derive(Default)]
pub struct QuestionBuilder<S: Into<String>> {
    packet_builder: DnsPacketBuilder,
    q_type: Option<QueryType>,
    q_class: Option<QueryClass>,
    q_name: Option<S>,
}

// TODO: remove
#[allow(dead_code)]
impl<S: Into<String>> QuestionBuilder<S> {
    pub fn q_type(mut self, q_type: QueryType) -> Self {
        self.q_type = Some(q_type);
        self
    }

    pub fn q_class(mut self, q_class: QueryClass) -> Self {
        self.q_class = Some(q_class);
        self
    }

    pub fn q_name(mut self, q_name: S) -> Self {
        self.q_name = Some(q_name);
        self
    }

    pub fn add_question(mut self) -> Result<DnsPacketBuilder> {
        self.packet_builder.questions.push(Question {
            q_name: self.q_name.ok_or(anyhow!("empty q_name"))?.into(),
            q_type: self.q_type.unwrap_or(QueryType::A),
            q_class: self.q_class.unwrap_or(QueryClass::IN),
        });

        Ok(self.packet_builder)
    }
}

impl DnsPacketBuilder {
    pub fn recursion_desired(mut self, recursion_desired: bool) -> Self {
        self.recursion_desired = recursion_desired;
        self
    }

    pub fn recursion_available(mut self, recursion_available: bool) -> Self {
        self.recursion_available = recursion_available;
        self
    }

    pub fn id(mut self, id: u16) -> Self {
        self.id = Some(id);
        self
    }

    pub fn message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    pub fn result_code(mut self, result_code: ResultCode) -> Self {
        self.result_code = Some(result_code);
        self
    }

    pub fn _new_question<S: Into<String>>(self) -> QuestionBuilder<S> {
        QuestionBuilder {
            packet_builder: self,
            q_type: None,
            q_class: None,
            q_name: None,
        }
    }

    pub fn new_raw_record<S: Into<String>>(self) -> RawRecordBuilder<S> {
        RawRecordBuilder {
            packet_builder: self,
            name: None,
            query_type: None,
            query_class: None,
            ttl: None,
            rdata: None,
        }
    }

    pub fn with_question(mut self, question: Question) -> Self {
        self.questions.push(question);
        self
    }

    pub fn with_base(mut self, base: DnsPacketBase) -> Self {
        self.base = Some(base);
        self
    }

    pub fn build(self) -> DnsPacket {
        let base = self.base.unwrap_or(DnsPacketBase {
            answers: self.answers,
            authorities: self.authorities,
            additional: self.additional,
        });

        DnsPacket {
            meta: DnsPacketMeta {
                header: Header {
                    id: if let Some(id) = self.id { id } else { random() },
                    message_type: self.message_type.unwrap_or(MessageType::Query),
                    opcode: OpCode::Query,
                    authoritative_answer: false,
                    truncation: false,
                    recursion_desired: self.recursion_desired,
                    recursion_available: self.recursion_available,
                    result_code: self.result_code.unwrap_or(ResultCode::NoError),
                    question_entities_count: self.questions.len() as u16,
                    answer_entities_count: base.answers.len() as u16,
                    authority_entities_count: base.authorities.len() as u16,
                    additional_entities_count: base.additional.len() as u16,
                },
                questions: self.questions,
            },
            base,
        }
    }
}

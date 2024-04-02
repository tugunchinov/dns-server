use anyhow::{anyhow, bail};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResultCode {
    NoError = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5,
}

impl TryFrom<u8> for ResultCode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoError),
            1 => Ok(Self::FormatError),
            2 => Ok(Self::ServerFailure),
            3 => Ok(Self::NameError),
            4 => Ok(Self::NotImplemented),
            5 => Ok(Self::Refused),
            _ => Err(anyhow!("unsupported result code")),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MessageType {
    Query = 0,
    Response = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum OpCode {
    Query = 0,
    IQuery = 1,
    Status = 2,
    Reserved(u8),
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Query,
            1 => Self::IQuery,
            2 => Self::Status,
            value => Self::Reserved(value),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::Query => 0,
            OpCode::IQuery => 1,
            OpCode::Status => 2,
            OpCode::Reserved(value) => value,
        }
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug)]
pub enum QueryType {
    A = 1,
    Unknown(u16),
}

impl From<u16> for QueryType {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::A,
            value => Self::Unknown(value),
        }
    }
}

impl TryFrom<&str> for QueryType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "A" => Ok(Self::A),
            _ => bail!("unknown query type"),
        }
    }
}

impl From<QueryType> for u16 {
    fn from(value: QueryType) -> Self {
        match value {
            QueryType::A => 1,
            QueryType::Unknown(value) => value,
        }
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug)]
pub enum QueryClass {
    IN = 1,
    Unknown(u16),
}

impl TryFrom<&str> for QueryClass {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "IN" => Ok(Self::IN),
            _ => bail!("unknown query class"),
        }
    }
}

impl From<QueryClass> for u16 {
    fn from(value: QueryClass) -> Self {
        match value {
            QueryClass::IN => 1,
            QueryClass::Unknown(value) => value,
        }
    }
}

impl From<u16> for QueryClass {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::IN,
            value => Self::Unknown(value),
        }
    }
}

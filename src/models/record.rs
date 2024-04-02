use crate::models::enums::{QueryClass, QueryType};
use crate::smart_buffer::SmartBuffer;
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct RawRecord {
    pub(in crate::models) name: String,
    pub(in crate::models) query_type: QueryType,
    pub(in crate::models) query_class: QueryClass,
    pub(in crate::models) ttl: u32,
    pub(in crate::models) rdata_length: u16,
    pub(in crate::models) rdata: Vec<u8>,
}

impl RawRecord {
    pub(in crate::models) fn from_bytes<T: AsRef<[u8]>>(
        smart_buf: &mut SmartBuffer<T>,
    ) -> Result<Self> {
        let name = smart_buf.read_qname()?;

        let query_type = QueryType::from(smart_buf.read_u16()?);
        let query_class = QueryClass::from(smart_buf.read_u16()?);
        let ttl = smart_buf.read_u32()?;
        let rdata_length = smart_buf.read_u16()?;
        let rdata = smart_buf.read_slice(rdata_length as usize)?.to_vec();

        Ok(Self {
            name,
            query_type,
            query_class,
            ttl,
            rdata_length,
            rdata,
        })
    }

    pub(in crate::models) fn to_bytes<T: AsMut<[u8]> + AsRef<[u8]>>(
        &self,
        smart_buf: &mut SmartBuffer<T>,
    ) -> Result<()> {
        smart_buf.write_qname(&self.name)?;
        smart_buf.write_u16(u16::from(self.query_type))?;
        smart_buf.write_u16(u16::from(self.query_class))?;
        smart_buf.write_u32(self.ttl)?;
        smart_buf.write_u16(self.rdata_length)?;
        smart_buf.write_slice(&self.rdata)?;

        Ok(())
    }
}

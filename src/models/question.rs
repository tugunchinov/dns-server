use crate::models::enums::{QueryClass, QueryType};
use crate::smart_buffer::SmartBuffer;
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct Question {
    pub(in crate::models) q_name: String,
    pub(in crate::models) q_type: QueryType,
    pub(in crate::models) q_class: QueryClass,
}

impl Question {
    pub(in crate::models) fn from_bytes<T: AsRef<[u8]>>(
        smart_buf: &mut SmartBuffer<T>,
    ) -> Result<Self> {
        let q_name = smart_buf.read_qname()?;
        let q_type = QueryType::from(smart_buf.read_u16()?);
        let q_class = QueryClass::from(smart_buf.read_u16()?);

        Ok(Self {
            q_name,
            q_type,
            q_class,
        })
    }

    pub(in crate::models) fn to_bytes<T: AsMut<[u8]> + AsRef<[u8]>>(
        &self,
        smart_buf: &mut SmartBuffer<T>,
    ) -> Result<()> {
        smart_buf.write_qname(&self.q_name)?;
        smart_buf.write_u16(u16::from(self.q_type))?;
        smart_buf.write_u16(u16::from(self.q_class))?;
        Ok(())
    }

    pub fn name(&self) -> &String {
        &self.q_name
    }
}

use crate::utils::Result;
use crate::query_type::QueryType;
use crate::byte_bucket_buffer::BytePacketBuffer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    pub name: String,
    pub qtype: QueryType,
}

impl Question {
    pub fn new(name: String, qtype: QueryType) -> Question {
        Question {
            name,
            qtype
        }
    }

    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.read_qname(&mut self.name)?;
        self.qtype = QueryType::from_num(buffer.read_u16()?);
        let _ = buffer.read_u16()?;
        Ok(())
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.write_qname(&self.name)?;
        let typenum = self.qtype.to_num();
        buffer.write_u16(typenum)?;
        buffer.write_u16(1)?;
        Ok(())
    }
}
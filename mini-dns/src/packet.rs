use crate::utils::Result;
use crate::header::Header;
use crate::question::Question;
use crate::record::Record;
use crate::query_type::QueryType;
use crate::byte_bucket_buffer::BytePacketBuffer;

#[derive(Clone, Debug)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub resources: Vec<Record>,
}

impl Packet {
    pub fn new() -> Packet {
        Packet {
            header: Header::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn from_buffer(buffer: &mut BytePacketBuffer) -> Result<Packet> {
        let mut result = Packet::new();
        result.header.read(buffer)?;
        for _ in 0..result.header.questions {
            let mut question = Question::new(String::new(), QueryType::UNKNOWN(0));
            question.read(buffer)?;
            result.questions.push(question);
        }
        for _ in 0..result.header.answers {
            let rec = Record::read(buffer)?;
            result.answers.push(rec);
        }
        for _ in 0..result.header.authoritative_entries {
            let rec = Record::read(buffer)?;
            result.authorities.push(rec);
        }
        for _ in 0..result.header.resource_entries {
            let rec = Record::read(buffer)?;
            result.resources.push(rec);
        }
        Ok(result)
    }

    pub fn write(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        self.header.questions = self.questions.len() as u16;
        self.header.answers = self.answers.len() as u16;
        self.header.authoritative_entries = self.authorities.len() as u16;
        self.header.resource_entries = self.resources.len() as u16;
        self.header.write(buffer)?;
        for question in &self.questions {
            question.write(buffer)?;
        }
        for rec in &self.answers {
            rec.write(buffer)?;
        }
        for rec in &self.authorities {
            rec.write(buffer)?;
        }
        for rec in &self.resources {
            rec.write(buffer)?;
        }
        Ok(())
    }
}
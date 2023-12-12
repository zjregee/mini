mod utils;
mod record;
mod header;
mod packet;
mod question;
mod query_type;
mod byte_bucket_buffer;

use std::net::UdpSocket;
use crate::utils::{Result, ResultCode};
use crate::query_type::QueryType;
use crate::packet::Packet;
use crate::question::Question;
use crate::byte_bucket_buffer::BytePacketBuffer;

fn lookup(qname: &str, qtype: QueryType) -> Result<Packet> {
    let server = ("8.8.8.8", 53);
    let socket = UdpSocket::bind(("0.0.0.0", 43210))?;
    let mut packet = Packet::new();
    packet.header.id = 6666;
    packet.header.questions = 1;
    packet.header.recursion_desired = true;
    packet.questions.push(Question::new(qname.to_string(), qtype));
    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer)?;
    socket.send_to(&req_buffer.buf[0..req_buffer.pos], server)?;
    let mut res_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut res_buffer.buf)?;
    Packet::from_buffer(&mut res_buffer)
}

fn handle_query(socket: &UdpSocket) -> Result<()> {
    let mut req_buffer = BytePacketBuffer::new();
    let (_, src) = socket.recv_from(&mut req_buffer.buf)?;
    let mut request = Packet::from_buffer(&mut req_buffer)?;
    let mut packet = Packet::new();
    packet.header.id = request.header.id;
    packet.header.recursion_desired = true;
    packet.header.recursion_available = true;
    packet.header.response = true;
    if let Some(question) = request.questions.pop() {
        println!("Received query: {:?}", question);
        if let Ok(result) = lookup(&question.name, question.qtype) {
            packet.questions.push(question);
            packet.header.rescode = result.header.rescode;
            for rec in result.answers {
                println!("Answer: {:?}", rec);
                packet.answers.push(rec);
            }
            for rec in result.authorities {
                println!("Authority: {:?}", rec);
                packet.authorities.push(rec);
            }
            for rec in result.resources {
                println!("Resource: {:?}", rec);
                packet.resources.push(rec);
            }
        } else {
            packet.header.rescode = ResultCode::SERVFAIL;
        }
    } else {
        packet.header.rescode = ResultCode::FORMERR;
    }
    let mut res_buffer = BytePacketBuffer::new();
    packet.write(&mut res_buffer)?;
    let len = res_buffer.pos();
    let data = res_buffer.get_range(0, len)?;
    socket.send_to(data, src)?;
    Ok(())
}

fn main() -> Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", 2053))?;
    loop {
        match handle_query(&socket) {
            Ok(_) => {}
            Err(e) => eprintln!("An error occurred: {}", e),
        }
    }
}
use crate::cache::{CacheItemPolicy, MemoryCache};
use crate::models::{
    new_packet_buffer, DnsPacket, DnsPacketBase, DnsPacketBuilder, MessageType, QueryClass,
    QueryType, Question, RawRecordType, ResultCode,
};
use anyhow::{bail, Context, Result};
use crossbeam::channel as mpmc;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const BROKEN_BIND_FILE_ERROR_MSG: &str = "broken bind file";

pub struct DnsServer {
    socket: UdpSocket,
    cache: MemoryCache<String, DnsPacketBase>,
}

impl DnsServer {
    pub fn new() -> Result<Self> {
        // TODO: port from config
        let socket = UdpSocket::bind(("0.0.0.0", 53))?;
        let cache = MemoryCache::new();

        Ok(Self { socket, cache })
    }

    pub fn run(self, num_workers: usize) -> Result<()> {
        let (tx, rx) = mpmc::unbounded();

        let this = Arc::new(self);
        let mut join_handles = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            let rx = rx.clone();
            let this = Arc::clone(&this);
            join_handles.push(thread::spawn(|| this.lookup_job(rx)));
        }

        this.handle_requests(tx)?;

        for handle in join_handles {
            if let Err(e) = handle.join().expect("failed joining thread") {
                eprintln!("error while handling requests: {e}");
            }
        }

        Ok(())
    }

    // TODO: recursive-lookup
    fn lookup_redirect(&self, id: u16, question: &Question) -> Result<DnsPacket> {
        println!("LOOKING REDIRECT");

        // TODO: from config
        let server = ("8.8.8.8", 53);
        let socket = UdpSocket::bind(("0.0.0.0", 43210))?;

        let request = DnsPacketBuilder::default()
            .id(id)
            .recursion_desired(true)
            .with_question(question.clone())
            .build();

        let mut buf = new_packet_buffer();
        request.to_bytes(&mut buf)?;

        socket.send_to(&buf, server)?;

        socket.recv_from(&mut buf)?;

        DnsPacket::from_bytes(&buf)
    }

    fn lookup_local(&self, request: &DnsPacket) -> Result<Option<DnsPacket>> {
        println!("LOOKING LOCAL");

        let question = request.questions().first().unwrap();

        let mut response_builder = Self::default_response_request_builder_from(request);

        // TODO: move path to bind in config
        let file = File::open("bind.txt")?;
        let mut reader = BufReader::new(file);

        let mut found = false;
        let mut buf = String::with_capacity(512);
        while reader.read_line(&mut buf)? > 0 {
            let tokens = buf.split_whitespace().collect::<Vec<_>>();

            if tokens.is_empty() || tokens[0] == "#" {
                continue;
            }

            if tokens.len() != 4 {
                bail!(BROKEN_BIND_FILE_ERROR_MSG);
            }

            if tokens[0] == question.name() {
                let name = tokens[0];
                let q_class =
                    QueryClass::try_from(tokens[1]).context(BROKEN_BIND_FILE_ERROR_MSG)?;
                let q_type = QueryType::try_from(tokens[2]).context(BROKEN_BIND_FILE_ERROR_MSG)?;

                // TODO: A-records now only, add more
                let rdata = tokens[3]
                    .parse::<Ipv4Addr>()
                    .context(BROKEN_BIND_FILE_ERROR_MSG)?
                    .octets();

                response_builder = response_builder
                    .new_raw_record()
                    .name(name)
                    .query_class(q_class)
                    .query_type(q_type)
                    .ttl(300)
                    .rdata(rdata.to_vec())
                    .add_raw_record(RawRecordType::Answer)?;

                found = true;
            }

            buf.clear();
        }

        if found {
            Ok(Some(response_builder.build()))
        } else {
            Ok(None)
        }
    }

    fn lookup_cache(&self, request: &DnsPacket) -> Option<DnsPacket> {
        println!("LOOKING CACHE");

        let question = request.questions().first().unwrap();

        self.cache.get(question.name()).map(|cached| {
            Self::default_response_request_builder_from(request)
                .with_base(cached.as_ref().clone())
                .build()
        })
    }

    fn try_lookup(&self, request: &DnsPacket) -> Result<DnsPacket> {
        let question = request.questions().first().unwrap();

        let response = if let Some(result) = self.lookup_cache(request) {
            result
        } else if let Some(result) = self.lookup_local(request)? {
            self.cache_response_without_ttl(question, &result);
            result
        } else {
            let result = self.lookup_redirect(request.id(), question)?;

            if result.result_code() == ResultCode::NoError {
                self.cache_response_with_ttl(question, &result);
            }
            result
        };

        Ok(response)
    }

    fn lookup(&self, request: DnsPacket, src: SocketAddr) -> Result<()> {
        let response = if !request.questions().is_empty() {
            match self.try_lookup(&request) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("failed looking-up: {e}");
                    Self::default_response_request_builder_from(&request)
                        .result_code(ResultCode::ServerFailure)
                        .build()
                }
            }
        } else {
            Self::default_response_request_builder_from(&request)
                .result_code(ResultCode::FormatError)
                .build()
        };

        let mut buf = new_packet_buffer();
        response.to_bytes(&mut buf)?;

        self.socket.send_to(&buf, src)?;

        Ok(())
    }

    fn lookup_job(
        self: Arc<Self>,
        rx_requests: mpmc::Receiver<(DnsPacket, SocketAddr)>,
    ) -> Result<()> {
        loop {
            match rx_requests.recv() {
                Ok((request, src)) => self.lookup(request, src)?,
                Err(e) => bail!("channel disconnected: {e:#?}"),
            }
        }
    }

    fn process_request(&self, tx: &mpmc::Sender<(DnsPacket, SocketAddr)>) -> Result<()> {
        let mut buf = new_packet_buffer();
        let (_, src) = self.socket.recv_from(&mut buf)?;

        let request = DnsPacket::from_bytes(&buf)?;

        Ok(tx.send((request, src))?)
    }

    fn handle_requests(&self, tx: mpmc::Sender<(DnsPacket, SocketAddr)>) -> Result<()> {
        loop {
            if let Err(e) = self.process_request(&tx) {
                eprintln!("error handling request: {e}");
            }
        }
    }

    fn default_response_request_builder_from(request: &DnsPacket) -> DnsPacketBuilder {
        DnsPacketBuilder::default()
            .id(request.id())
            .recursion_desired(request.recursion_desired())
            .recursion_available(false)
            .message_type(MessageType::Response)
    }

    fn cache_response_with_ttl(&self, question: &Question, response: &DnsPacket) {
        if let Some(ttl) = response.min_ttl() {
            self.cache.add(
                question.name().clone(),
                response.base().clone(),
                CacheItemPolicy::AbsoluteExpiration(Duration::from_secs(ttl as u64)),
            );
        }
    }

    fn cache_response_without_ttl(&self, question: &Question, response: &DnsPacket) {
        self.cache.add(
            question.name().clone(),
            response.base().clone(),
            CacheItemPolicy::NoExpiration,
        );
    }
}

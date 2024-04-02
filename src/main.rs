// TODO: remove anyhow
// TODO: use env_logger

mod cache;
mod helpers;
mod models;
mod server;
mod smart_buffer;

#[cfg(test)]
mod tests;

fn main() {
    server::DnsServer::new()
        .unwrap()
        .run(
            std::thread::available_parallelism()
                .expect("failed getting cpu number")
                .get(),
        )
        .unwrap();
}

// TODO: remove anyhow

mod cache;
mod helpers;
mod models;
mod server;
mod smart_buffer;

#[cfg(test)]
mod tests;

fn main() {
    env_logger::init();

    server::DnsServer::new()
        .unwrap()
        .run(
            std::thread::available_parallelism()
                .expect("failed getting cpu number")
                .get(),
        )
        .unwrap();
}

use crate::server::DnsServer;
use rand::distributions::Alphanumeric;
use rand::{random, Rng};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::process::Command;
use std::thread;

#[test]
fn stress_test() {
    thread::spawn(|| DnsServer::new().unwrap().run(12).unwrap());

    let test_output_file_path: &'static str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/target/stress_test_output.txt");

    File::create(test_output_file_path).unwrap();

    let jh1 = thread::spawn(move || {
        let mut file = OpenOptions::new()
            .append(true)
            .open(test_output_file_path)
            .unwrap();

        for _ in 0..100 {
            let output = Command::new("dig")
                .args(["@127.0.0.1", "-p", "53"])
                .arg(match random::<u8>() {
                    0 => "facebook.com",
                    1 => "vk.com",
                    13 => "instagram.com",
                    42 => "mit.edu",
                    _ => "google.com",
                })
                .output()
                .expect("Failed to execute command");

            let text_output = std::str::from_utf8(&output.stdout).unwrap();

            writeln!(file, "{text_output}").unwrap();
        }
    });

    let jh2 = thread::spawn(move || {
        let mut file = OpenOptions::new()
            .append(true)
            .open(test_output_file_path)
            .unwrap();

        for _ in 0..100 {
            let output = Command::new("dig")
                .args(["@127.0.0.1", "-p", "53"])
                .arg(match random::<u8>() {
                    1 => "example.com",
                    2 => "ns1",
                    3 => "mail",
                    4 => "joe",
                    _ => "www",
                })
                .output()
                .expect("Failed to execute command");

            let text_output = std::str::from_utf8(&output.stdout).unwrap();

            writeln!(file, "{text_output}").unwrap();
        }
    });

    let jh3 = thread::spawn(move || {
        let mut file = OpenOptions::new()
            .append(true)
            .open(test_output_file_path)
            .unwrap();

        for _ in 0..100 {
            let stupid_domain: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(random::<u8>() as usize)
                .map(char::from)
                .collect();

            let output = Command::new("dig")
                .args(["@127.0.0.1", "-p", "53"])
                .arg(stupid_domain)
                .output()
                .expect("Failed to execute command");

            let text_output = std::str::from_utf8(&output.stdout).unwrap();

            writeln!(file, "{text_output}").unwrap();
        }
    });

    jh1.join().unwrap();
    jh2.join().unwrap();
    jh3.join().unwrap();
}

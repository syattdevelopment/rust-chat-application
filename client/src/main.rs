use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

extern crate time;

fn main() {
    let mut current_time = time::get_time();
    let mut start = (current_time.sec as i64 * 1000) +
        (current_time.nsec as i64 / 1000 / 1000);
    println!("Client Started at {:?}", start);

    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect.");
    client.set_nonblocking(true).expect("Failed to initiate non-blocking.");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                let first_one: Vec<char> = msg.chars().take(1).collect();
                let code = first_one[0];

                if code.to_string() == "1" {
                    let sent = &msg[1..(msg.len())].to_string();
                    println!("Message sent from server: {:?}", sent);
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    client.write_all(&buff).expect("Writing to socket failed.");
                } else {
                    current_time = time::get_time();
                    let mut end = (current_time.sec as i64 * 1000) +
                        (current_time.nsec as i64 / 1000 / 1000);
                    let _msg_to_owned = msg.to_owned();
                    let sent = &msg[1..(msg.len())].to_string();
                    println!("Server has received your message: {:?}", sent);
                    println!("Round-trip for acknowledgement was {:?} milliseconds", (end - start));
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed.");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let code = "0".to_string();
                let sent = code + &msg.to_string();
                current_time = time::get_time();
                start = (current_time.sec as i64 * 1000) +
                    (current_time.nsec as i64 / 1000 / 1000);
                let mut buff = sent.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing to socket failed.");

                println!("Message sent to server: {:?}", msg);
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a Message:");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Reading from stdin failed.");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() { break; }
    }
    println!("Client quit.");
}

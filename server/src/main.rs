use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{self};
use std::thread;

extern crate time;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server.set_nonblocking(true).expect("failed to initialize non-blocking");

    let current_time = time::get_time();
    let start = (current_time.sec as i64 * 1000) +
        (current_time.nsec as i64 / 1000 / 1000);
    println!("Server Started at {:?}", start);

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);
            println!("Write a Message");

            let tx = tx.clone();

            clients.push(socket.try_clone().expect("failed to clone client"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                let re_tx = tx.clone();

                thread::spawn(move || {
                    let mut buff = String::new();
                    io::stdin().read_line(&mut buff).expect("Reading from stdin failed.");
                    let msg = buff.trim().to_string();
                    println!("Message sent to client: {:?}", msg);
                    re_tx.send(msg)
                });

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                        let first_one: Vec<char> = msg.chars().take(1).collect();
                        let code = first_one[0];

                        let _msg_to_owned = msg.to_owned();
                        let sent = &msg[1..(msg.len())].to_string();

                        if code.to_string() == "1" {
                            println!("Client ({}) has received your message: {:?}", addr, sent);
                        } else {
                            let mut buff = msg.clone().into_bytes();
                            buff.resize(MSG_SIZE, 0);
                            println!("Message sent from client ({}): {:?}", addr, sent);
                            socket.write_all(&buff).expect("Writing to socket failed.");
                        }
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }
        if let Ok(_msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let code = "1";
                let msg = code.to_string() + &_msg.to_string();
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }

        sleep();
    }
}

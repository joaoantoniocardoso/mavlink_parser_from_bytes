use std::io::{Read, Write};

use mavlink::ardupilotmega::MavMessage;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let filename = args.get(1).expect("Missing argument <input_file>");

    println!("Input filename: {filename:?}");

    let mut file = std::fs::File::open(filename).unwrap();

    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    // parsing input file:

    let bytes = buf
        .split(", ")
        .enumerate()
        .map(|(p, s)| {
            s.parse::<u8>()
                .unwrap_or_else(|_| panic!("Failed at position {p}, character: {s}"))
        })
        .collect::<Vec<u8>>();

    // parsing mavlink

    let mut outfile = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("{filename}.parsed"))
        .unwrap();

    let mut read = mavlink::peek_reader::PeekReader::new(bytes.as_slice());
    loop {
        let message = match mavlink::read_v2_msg::<MavMessage, _>(&mut read) {
            Ok(message) => message,
            Err(error) => {
                if let mavlink::error::MessageReadError::Io(io_error) = &error {
                    if io_error.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    }
                };

                eprintln!("Failed parsing: {error:?}");
                continue;
            }
        };

        let mut serialized = serde_json::to_string(&message).unwrap();
        std::fmt::Write::write_char(&mut serialized, '\n').unwrap();

        // wirte out
        outfile.write_all(serialized.as_bytes()).unwrap();
    }

    println!("Terminated");
}

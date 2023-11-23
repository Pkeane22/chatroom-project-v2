use std::io::{Write, stdin, stdout};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use rustyline_async::{Readline, ReadlineEvent};

const BUF_MAX: usize = 512;

#[tokio::main]
pub async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();

    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    let mut buf_read = Vec::with_capacity(BUF_MAX);
    reader.read_until(b'\0', &mut buf_read).await.unwrap();
    print!("{}", String::from_utf8(buf_read).unwrap());
    stdout().flush().unwrap();
    let mut buf_read = String::with_capacity(BUF_MAX);
    stdin().read_line(&mut buf_read).unwrap();
    writer.write_all(&buf_read.as_bytes()).await.unwrap();

    let (mut rl, mut stdout) = Readline::new("You: ".to_owned()).unwrap();
    rl.should_print_line_on(false, false);

    loop {
        tokio::select! {
                    result = reader.read_line(&mut buf_read) => {
                        let result = result.unwrap();
                        if result == 0 {
                            println!("Server disconnected");
                            break;
                        }

                        writeln!(stdout, "{}", &buf_read[..result-1]).unwrap();
                        buf_read.clear();
                    }
                    result = rl.readline() => match result.unwrap() {
                        ReadlineEvent::Line(buf_write) => {
                            
                            writer.write_all([&buf_write, "\n"].concat().as_bytes()).await.unwrap();
                            writeln!(stdout, "You: {}", buf_write).unwrap();
                        },
                        _ => { break; }
        
                    }
                }
    }
    stream.shutdown().await.unwrap();
}

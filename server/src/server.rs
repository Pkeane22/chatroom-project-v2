use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

const BUF_MAX: usize = 512;

#[tokio::main]
pub async fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    let (tx, _rx) = broadcast::channel(5);
    run_server(listener, tx).await;
}

async fn run_server(listener: TcpListener, tx: broadcast::Sender<(String, SocketAddr)>) {
    loop {
        let (stream, addr) = listener.accept().await.unwrap();

        let client = Client::new(stream, addr, &tx);
        tokio::spawn(async move {
            println!("Client connected");
            handle_connection(client).await;
        });
    }
}

async fn handle_connection(mut client: Client) {
    let (reader, mut writer) = client.stream.split();
    let mut reader = BufReader::new(reader);
    let mut buf = String::with_capacity(BUF_MAX);

    writer
        //.write_all("Enter Username: \n".as_bytes())
        .write_all("Enter Username: \0".as_bytes())
        .await
        .unwrap();
    let result = reader.read_line(&mut buf).await.unwrap();
    if result < 1 {
        println!("Client disconnected");
        return;
    }

    client.username = |buf: &mut String| -> String {
        while buf.ends_with('\n') || buf.ends_with('\r') {
            buf.pop();
        }
        buf.clone()
    }(&mut buf);
    buf.clear();

    loop {
        tokio::select! {
            result = reader.read_line(&mut buf) => {
                let result = result.unwrap();
                if result < 1 || (result > 1 && (&buf[..2] == "q\n" || &buf[..2] == "q\r")) {
                    println!("Client disconnected");
                    break;
                }

                client.tx.send((format!("{}: {}", client.username, buf), client.addr)).unwrap();
                print!("{}: {}", client.username, buf);
                buf.clear();
            }

            result = client.rx.recv() => {
                let (msg, other_addr) = result.unwrap();

                if client.addr != other_addr {
                    writer.write_all(msg.as_bytes()).await.unwrap();
                }
            }
        }
    }
    client.stream.shutdown().await.unwrap();
}

struct Client {
    stream: TcpStream,
    addr: SocketAddr,
    tx: broadcast::Sender<(String, SocketAddr)>,
    rx: broadcast::Receiver<(String, SocketAddr)>,
    username: String,
}

impl Client {
    fn new(
        stream: TcpStream,
        addr: SocketAddr,
        tx: &broadcast::Sender<(String, SocketAddr)>,
    ) -> Self {
        Client {
            stream,
            addr,
            tx: tx.clone(),
            rx: tx.subscribe(),
            username: String::new(),
        }
    }
}

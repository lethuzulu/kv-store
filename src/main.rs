use kv_store::server::TcpServer;

fn main() {
    let tcp_server = TcpServer::new("localhost:5000").unwrap();
    tcp_server.listen();
}

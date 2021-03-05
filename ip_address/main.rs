use std::net::UdpSocket;

/// get the local ip address, return an `Option<String>`. when it fail, return `None`.
fn get() -> String {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("1.1.1.1:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string()
            }
        };
    }

    return String::from("");
}

fn main() {
    print!("{}", get())
}

extern mod core;
extern mod std;

use core::comm::*;
use comm = core::comm;

use std::net::*;
use ip = net_ip;
use tcp = net_tcp;
use std::uv;

enum MyResult {
    MyErr(int),
    MyQueryResult(~[u8]),
}

fn on_establish_cb(kill_ch : comm::SharedChan<Option<tcp::TcpErrData>>) {
    // nothing really
    println(fmt!("set up server"));
}

/**
 * Processes request which comes in bytes. In general, this may consist
 * of parsing the converted request, carrying out different operations,
 * depending on the request, etc. Returns either an error flag or a
 * result (in bytes).
 *
 * # Arguments
 *
 * * `request` - request in bytes; whatever came from the client
 *
 * # Returns
 *
 * either a MyError(int) flag
 * * 0 : disconnect
 * or the result of the request, in byte format
 *
 */
fn process_request(request : ~[u8]) -> MyResult {
    let from_client = str::from_bytes(request);
    match from_client {
        // if "q", then disconnect normally (exit(0))
        ~"q" => MyErr(0),
        // otherwise, the only processing we do here is change the message a bit
        _ => MyQueryResult(str::to_bytes(fmt!("server did some computation on \"%s\"", from_client))),
    }
}

/**
 * Does main work for establishing new connections, returning early
 * if there is a error relating to TCP operations, and exiting with
 * no error otherwise.
 *
 * Normally, we would probably not want to fail fatally in so many
 * of these error cases; for now we do.
 */
fn new_connect_cb_single(new_conn : &tcp::TcpNewConnection, kill_ch : &comm::SharedChan<Option<tcp::TcpErrData>>)
            -> result::Result<(), tcp::TcpErrData> {
    // accept the connection; abort if unable, extract socket otherwise
    let sock = match tcp::accept(*new_conn) {
        result::Err(e) => return result::Err(e),
        result::Ok(s) => s,
    };

    // print info on client that connected
    let client_addr = sock.get_peer_addr();
    let client_ip = ip::format_addr(&client_addr);
    let client_port = ip::get_port(&client_addr);
    println(fmt!("server connected to %s:%u", client_ip, client_port));

    // begin continuous read; abort if unable, extract port over which to read otherwise
    let read_po = match sock.read_start() {
        result::Err(e) => return result::Err(e),
        result::Ok(r) => r,
    };

    loop {
        // get a request
        let request = match read_po.recv() {
            result::Err(e) => return result::Err(e),
            result::Ok(v) => v,
        };

        // process the request
        match process_request(request) {
            MyErr(0) => {
                // exit(0), so stop reading from socket and disconnect
                sock.read_stop();
                println(fmt!("server disconnected from %s:%u", client_ip, client_port));
                break;
            }
            MyErr(x) => {
                // shouldn't get here for now...
                debug!("ERR unknown error %d!", x);
            }
            MyQueryResult(result) => {
                // write result asynchronously ("future") to the socket; abort if unable
                let w = sock.write_future(result).get();
                if result::is_err(&w) { return result::Err(result::get_err(&w)); }
            }
        }
    }

    // normal exit
    result::Ok(())
}

fn new_connect_cb(new_conn : tcp::TcpNewConnection, kill_ch : comm::SharedChan<Option<tcp::TcpErrData>>) {
    // spawn a new task for this connection that will not kill the parent if anything happens to fail
    do task::spawn_supervised {
        match new_connect_cb_single(&new_conn, &kill_ch) {
            result::Err(e) => println(fmt!("ERR %s : %s", e.err_name, e.err_msg)),
            _ => (),
        }
    }
}

/**
 * Start server at given IP and port. In general, might also take options
 * with which to configure server.
 *
 * # Arguments
 *
 * * `server_ip_str` - server IP, as a string
 * * `server_port` - server port, as a uint
 */
fn start_server(server_ip_str : ~str, server_port : uint) {
    let server_ip = ip::v4::parse_addr(server_ip_str);
    let iotask = uv::global_loop::get();

    tcp::listen(server_ip, server_port, 128, &iotask, on_establish_cb, new_connect_cb);
}

fn main() {
    println("runs on loop; can connect to several clients at once");
    // start server with given IP and port, and options [currently none]
    start_server(~"127.0.0.1", 8888u);
}

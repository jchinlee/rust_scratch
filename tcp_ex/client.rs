extern mod std;
extern mod core;

use std::net::*;
use ip = net_ip;
use tcp = net_tcp;
use std::uv;

/**
 * Parse user input string and transform into bytes.
 *
 * # Arguments
 *
 * * `input` - input as a string
 *
 * # Returns
 *
 * byte representation of input for transmitting over socket
 */
fn process_input(input : ~str) -> ~[u8] {
    // no parsing, actually ; just direct conversion
    str::to_bytes(input)
}

/**
 * Actually send request in byte form to server and get result.
 *
 * # Arguments
 *
 * * `sock` - the socket over which to send the request
 * * `request` - the request, in byte form
 * * `port` - the port over which responses will be sent
 *
 * # Returns
 *
 * result of write
 */
fn send_request(sock : &tcp::TcpSocket, request : ~[u8], port : @Port<result::Result<~[u8], tcp::TcpErrData>>)
            -> result::Result<(), tcp::TcpErrData> {
    // send request to server (asynchronously; "future")
    let mut write_result_future = sock.write_future(copy request);
    match write_result_future.get() {
        result::Err(e) => return result::Err(e),    // error, so return
        _ => result::Ok(()),
    }
}

/**
 * Get response from server and parse.
 * Right now, no parsing; just byte conversion
 */
fn get_result(port : @Port<result::Result<~[u8], tcp::TcpErrData>>) -> result::Result<~str, tcp::TcpErrData> {
    // block until receive message from server over port
    match port.recv() {
        result::Err(e) => result::Err(e),
        result::Ok(response) => result::Ok(str::from_bytes(response)),
    }
}

fn connect_to_server_wrap(server_ip_str : ~str, server_port : uint) -> result::Result<(), tcp::TcpErrData> {
    let server_ip = ip::v4::parse_addr(server_ip_str);
    let iotask = uv::global_loop::get();

    // connect to server; abort if unable, extract socket otherwise
    let sock = match tcp::connect(server_ip, server_port, &iotask) {
        result::Err(e) => {
            // rewrap the error : gross
            return match e {
                tcp::GenericConnectErr(ename, emsg) => result::Err(tcp::TcpErrData { err_name : ename, err_msg : emsg }),
                tcp::ConnectionRefused => result::Err(tcp::TcpErrData  { err_name : ~"EHOSTNOTFOUND", err_msg : ~"Invalid IP or port"}),
            }
        }
        result::Ok(s) => s,
    };

    // begin continuous read; abort if unable, extract port over which to read otherwise
    let read_po = match sock.read_start() {
        result::Err(e) => return result::Err(e),
        result::Ok(rs) => rs,
    };
    let reader = io::stdin();

    loop {
        // get user request
        print("> ");
        let line = reader.read_line();

        let req = process_input(copy line);

        // send request to server; abort if able
        //      in reality, would probably not abort but handle error appropriately
        let w = sock.write_future(req).get();
        if result::is_err(&w) { return result::Err(result::get_err(&w)); }

        // if request to disconnect, disconnect
        if line == ~"q" {
            sock.read_stop();
            println("bye!");
            break;
        }

        // otherwise, get response (if applicable; here, always applicable)
        //      in reality might not always want response either
        let resp = get_result(read_po);
        if result::is_err(&resp) { return result::Err(result::get_err(&resp)); }
        println(fmt!("server says: %s", result::unwrap(resp)));
    }

    // normal exit
    result::Ok(())
}

fn main() {
    let server_ip_str = ~"127.0.0.1";
    let server_port = 8888u;
    let iotask = uv::global_loop::get();
    let reader = io::stdin();

    println("Server will echo everything you say; q to disconnect.");
    match connect_to_server_wrap(server_ip_str, server_port) {
        result::Err(e) => println(fmt!("ERR %s : %s", e.err_name, e.err_msg)),
        _ => (),
    }
}

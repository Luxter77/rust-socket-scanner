use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::sync::{Mutex, Arc};
use ipnet::Ipv4AddrRange;
use std::time::Duration;
use std::thread::sleep;
use std::thread;


const CONNECTION_TIME: u64  = 2;
const PORTS: &'static [u16] = &[21];
const CORES: usize = 100;
const QUESTIONMARK: &'static str = "?";
const NET_BUFFER: usize = 4096;

fn load_work_into_queue(ip_queue: Arc<Mutex<Vec<Ipv4AddrRange>>>) -> () {
    
    let mut rdr = csv::Reader::from_path("./extra/connlist.csv").expect("Can't open connlist file");
    
    for result in rdr.records() {
        let record = result.expect("Can't read range line.");
        // println!("IN QUEUE -> {:?}", record);
        ip_queue.lock().unwrap().push(Ipv4AddrRange::new(record[0].parse().unwrap(), record[1].parse().unwrap()));
    }
}

fn query_socket(ip: Ipv4Addr, port: u16, r: &mut [u8]) -> bool {
    let stream = TcpStream::connect_timeout(&SocketAddr::new(IpAddr::V4(ip), port), Duration::new(CONNECTION_TIME, 0));
    
    if !stream.is_ok() {
        return false
    } else {
        let resoult = std::io::Read::read(&mut stream.unwrap(), r);
        if resoult.is_ok() {
            return true;
        } else {
            return false;
        };
    };
}

fn proc_range(input: Arc<Mutex<Vec<Ipv4AddrRange>>>, output: Arc<Mutex<Vec<(Ipv4Addr, u16, [u8; NET_BUFFER])>>>, counter: Arc<Mutex<[u64; 2]>>) -> () {
    loop {
        let range = input.lock().unwrap().pop();
        if range.is_none() { break };
        for ip in range.unwrap() {
            for port in PORTS {
                // print!("{:?}:{:?}... ", ip.clone(), port.clone());
                let mut r: [u8; NET_BUFFER] = [0; NET_BUFFER];
                if query_socket(ip.clone(), port.clone(), &mut r) {
                    output.lock().unwrap().push((ip.clone(), port.clone(), r)); // println!("HIT!") 
                    {
                        let mut c = counter.lock().unwrap();
                        c[0] = c[0] + 1;
                        c[1] = c[1] + 1;
                    };
                } else {
                    let mut c = counter.lock().unwrap();
                    c[0] = c[0] + 1;
                };
            }
        }
    }
}

fn format_record(record: (Ipv4Addr, u16, [u8; NET_BUFFER]), out: &mut [String; 3]) -> () {
    let octets = record.0.octets();
    out[0] = format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3]);
    out[1] = format!("{}", record.1);
    out[2] = format!("{}", 
        match std::str::from_utf8(&record.2) {
            Ok(v) => v,
            Err(_) => QUESTIONMARK.clone(),
        }.trim_matches(char::from(0))
    );
}

fn main() {
    let mut writer = csv::Writer::from_path("./extra/scannout.csv").expect("Can't write out file.");
    
    let write_queue:    Arc<Mutex<Vec<(Ipv4Addr, u16, [u8; NET_BUFFER])>>>  = Arc::new(Mutex::new(Vec::new()));
    let ip_queue:       Arc<Mutex<Vec<Ipv4AddrRange>>>                      = Arc::new(Mutex::new(Vec::new()));
    let counter:        Arc<Mutex<[u64; 2]>>                                = Arc::new(Mutex::new([0, 0]));

    load_work_into_queue(ip_queue.clone());

    for _ in 0..(CORES*4) {
        let (_i, _o, _c) = (ip_queue.clone(), write_queue.clone(), counter.clone());
        thread::spawn(move || { proc_range(_i, _o, _c) });
    }

    let mut w_batch: bool  = true;
    let mut l_count: usize = 0;
    let mut l_len:   usize;

    loop {
	if write_queue.lock().unwrap().is_empty() {
            if ip_queue.lock().unwrap().is_empty() {
                break;
            } else {
                w_batch = true;
                sleep(Duration::new(10*CONNECTION_TIME, 0));
            }
        } else {
            let mut formatted_record: [String; 3] = [String::with_capacity(15), String::with_capacity(6), String::with_capacity(NET_BUFFER)];
            let last = write_queue.lock().unwrap().pop().unwrap();
            format_record(last.clone(),  &mut formatted_record);
            writer.write_record(formatted_record.clone()).unwrap();
            writer.flush().unwrap();
            let c_display: String;
            if w_batch {
                l_len = write_queue.lock().unwrap().len() + 1;
                let c = counter.lock().unwrap().clone();
                c_display = format!("[c:{}, f:{}, b:{}] ", c[0], c[1], l_len);
                l_count = c_display.chars().count();
                w_batch = false;
            } else {
                c_display = " ".repeat(l_count);
	    }
            println!("{}{:?}", c_display, formatted_record);
        }
    }
}

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::sync::{Mutex, Arc};
use ipnet::Ipv4AddrRange;
use std::time::Duration;
use std::thread::sleep;
use std::thread;


const CONNECTION_TIME: u64  = 2;
const PORTS: &'static [u16] = &[21, 22, 80, 443, 1021, 1022];
const CORES: usize = 10;

fn load_work_into_queue(ip_queue: Arc<Mutex<Vec<Ipv4AddrRange>>>) -> () {
    
    let mut rdr = csv::Reader::from_path("./extra/connlist.csv").expect("Can't open connlist file");
    
    for result in rdr.records() {
        let record = result.expect("Can't read range line.");
        // println!("IN QUEUE -> {:?}", record);
        ip_queue.lock().unwrap().push(Ipv4AddrRange::new(record[0].parse().unwrap(), record[1].parse().unwrap()));
    }
}

fn query_socket(ip: Ipv4Addr, port: u16) -> bool {
    return TcpStream::connect_timeout(&SocketAddr::new(IpAddr::V4(ip), port), Duration::new(CONNECTION_TIME, 0)).is_ok();
}

fn proc_range(input: Arc<Mutex<Vec<Ipv4AddrRange>>>, output: Arc<Mutex<Vec<(Ipv4Addr, u16)>>>) -> () {
    loop {
        let range = input.lock().unwrap().pop();
        if range.is_none() { break };
        for ip in range.unwrap() {
            for port in PORTS {
                // print!("{:?}:{:?}... ", ip.clone(), port.clone());
                if query_socket(ip.clone(), port.clone()) {
                    output.lock().unwrap().push((ip.clone(), port.clone())); // println!("HIT!") 
                } // else    { println!("boo") }
            }
        }
    }
}

fn format_record(record: (Ipv4Addr, u16), out: &mut [String; 2]) -> () {
    let octets = record.0.octets();
    out[0] = format!("{:03}.{:03}.{:03}.{:03}", octets[0], octets[1], octets[2], octets[3]);
    out[1] = format!("{:06}", record.1);
}

fn main() {
    let mut writer = csv::Writer::from_path("./extra/scannout.csv").expect("Can't write out file.");
    
    let write_queue:    Arc<Mutex<Vec<(Ipv4Addr, u16)>>>    = Arc::new(Mutex::new(Vec::new()));
    let ip_queue:       Arc<Mutex<Vec<Ipv4AddrRange>>>      = Arc::new(Mutex::new(Vec::new()));
    
    load_work_into_queue(ip_queue.clone());

    for _ in 0..(CORES*4) {
        let (_i, _o) = (ip_queue.clone(), write_queue.clone());
        thread::spawn(move || { proc_range(_i, _o) });
    }

    loop {
        if write_queue.lock().unwrap().is_empty() {
            if ip_queue.lock().unwrap().is_empty() {
                break;
            } else {
                sleep(Duration::new(10*CONNECTION_TIME, 0));
            }
        } else {
            let mut formatted_record: [String; 2] = [String::with_capacity(15), String::with_capacity(6)];
            let last = write_queue.lock().unwrap().pop().unwrap();
            format_record(last.clone(),  &mut formatted_record);
            writer.write_record(formatted_record).unwrap();
            writer.flush().unwrap();
            println!("{:?}", last);
        }
    }
}

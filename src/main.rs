use std::env;
use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::str::FromStr; // FromStr trait allows conversion of String to IpAddr type
use std::process; // manages the way program shuts down/terminates
use std::sync::mpsc::{Sender, channel};
use std::thread;

const MAX: u16 = 65535; // max port # that can be sniffed

struct Arguments { // struct to define & hold arguments' type
    flag: String,
    ipaddress: IpAddr,
    threads: u16,
}

impl Arguments { // implementation block to allow instantiation of Arguments struct
    // method 'new' takes in args (reference to vector) and returns the Arguments struct in its result  
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 { // minimum 2 arguments required
            return Err("ERROR - Not enough arguments");
        } 
        else if args.len() > 4 {
            return Err("ERROR - Too many arguments");
        }
        let first = args[1].clone(); // create variable to look at first index of vector
        // if-let binding to destruct IpAddr and return a result
        if let Ok(ipaddress) = IpAddr::from_str(&first) {
            return Ok(Arguments{flag: String::from(""), ipaddress, threads: 4});
        }
        else { // else if converting 'first' to IP address fails
            let flag = args[1].clone();
            if flag.contains("-h") || flag.contains("-help") && args.len() == 2 {
                println!("Usage: -t to select number of threads\r\n-h or -help to show help message");
                return Err("Help");
            }
            else if flag.contains("-h") || flag.contains("-help") {
                return Err("ERROR - Too many arguments");
            }
            else if flag.contains("-t") {
                // match on turning arguments[3] to an IP address & bind it to variable 'ipaddr'
                let ipaddress = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("ERROR - Not a valid IPADDR; must be IPv4 or IPv6")
                };
                let threads = match args[2].parse::<u16>() { // change string input to u16 using parse()
                    Ok(s) => s,
                    Err(_) => return Err("ERROR - Failed to parse thread number")
                };
                return Ok(Arguments{threads, flag, ipaddress});
            }
            else {
                return Err("ERROR - Invalid syntax");
            }
        }
    }
}

// start_port argument allows scan function to scale based on number of threads passed in
fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1; // start looking from port 1 and not 0
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => { // if port is open
                print!(".");
                io::stdout().flush().unwrap(); // flush method sends print statements to a mutex of shared data
                tx.send(port).unwrap(); // sends the # of open port back to rx
            }
            Err(_) => {} // return an empty expression in case of error
        }
        if (MAX - port) <= num_threads { // ensures to break loop when max port is reached
            break;
        }
        port += num_threads; // iterate port by the number of thread
    }
}

fn main() {
    let args: Vec<String> = env::args().collect(); // take all arguments passed and place them in a Vec
    let program = args[0].clone();
    // create variable 'arguments' & call unwrap_or_else method on it to handle an error
    let arguments = Arguments::new(&args).unwrap_or_else(
        |err| {
            if err.contains("help") {
                process::exit(0); // call process::exit() & pass code 0 to avoid panic
            }
            else {
                eprintln!("{} problem parsing arguments: {}", program, err);
                process::exit(0);
            }
        }
    );
    
    let num_threads = arguments.threads; // bind arguments.threads to variable 'num_threads'
    let addr = arguments.ipaddress; // create variable 'addr' which corresponds to arguments.ipaddress
    let (tx, rx) = channel(); // instantiate a channel, destruct the tuple which is returned into tx, rx

    for i in 0..num_threads { // iterate from 0 to number of threads
        let tx = tx.clone(); // bind 'tx' to a separate tx, ensure each thread has its own transmitter

        thread::spawn(move || {
            scan(tx, i, addr, num_threads);
        });
    }
    
    let mut out = vec![];
    drop(tx); // call drop() method on tx to ensure it is not in the main thread
    for j in rx { // get output from receiver by iterating over it and pushing values to 'out' vector
        out.push(j);
    }
    println!("\n\nScan report for {}", addr);
    println!("");
    out.sort(); // sort output in order
    for k in out {
        println!("Port {} is open", k);
    }
}

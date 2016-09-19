use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;

fn main() {
    
    let current_path = Path::new(".");
    let original_dir = match current_path.canonicalize() {
        Err(why) => panic!("couldn't open current directory: {}",why.description()),
        Ok(original_dir) => original_dir,
    };

    let args: Vec<String> = env::args().collect();
    let dirs = args.iter().skip(1);
    for dir in dirs {
        check_supervise(Path::new(dir)).unwrap();
    }

    env::set_current_dir(&original_dir).unwrap();
}

#[derive(Debug,Default)]
struct Service
{
    name:        String,
    path:        String,
    pid:         Option<u32>,
    is_up:       bool,
    normally_up: bool,
    is_paused:   bool,
    duration:    u32,

}

fn check_supervise(dir: &Path) -> io::Result<()> {
    
    if let Err(_) = env::set_current_dir(&dir) {
        println!("unable to chdir: {}", dir.display());
    }

    let mut normally_up = false;
    if let Err(e) = std::fs::metadata("down") {
        if e.kind() == io::ErrorKind::NotFound {
            normally_up = true;
        } else {
            println!("unable to stat down: {}", e);
        }
    }

    if let Err(e) = std::fs::metadata("supervise/ok") {
        if e.kind() == io::ErrorKind::AddrNotAvailable {
            println!("supervise not running");
            return Ok(())
        }
        println!("unable to open supervise/ok: {}", e.description());
        return Ok(())
    }

    let mut status_buf: [u8;18] = [0;18]; 
    {
        let mut status_file = match File::open("supervise/status") {
            Ok(status_file) => status_file,
            Err(e)          => {
                println!("unable to open supervise/status: {}", e.description());
                return Ok(())
            }
        };
        let read_bytes = status_file.read(&mut status_buf[..]);
        let base = "unable to read supervise/status:";
        match read_bytes {
            Ok(n) if n == status_buf.len() => {},
            Ok(_)   => println!("{} bad format", base) ,
            Err(e)  => println!("{} {}", base, e.description()),
        };
    }
    
    let pid = get_pid(&status_buf[12..16]);
    //let mut  pid:u32     = status_buf[15] as u32;
    //pid = pid << 8; pid += status_buf[14] as u32;
    //pid = pid << 8; pid += status_buf[13] as u32;
    //pid = pid << 8; pid += status_buf[12] as u32;

    let want = status_buf[17] as char;
    let paused = status_buf[16] as char;

    let service = Service { 
            name: dir.file_name().unwrap().to_str().unwrap().to_string(),
            path: String::from(dir.parent().unwrap().to_str().unwrap()),
            is_up: if pid != 0 { true } else { false },
            normally_up: normally_up,
            is_paused: if paused as u8 != 0 { true } else {false},
            duration: 0,
            pid: if pid != 0 { Some(pid) } else {None}
        };

    println!("{:?}", service);
    Ok(())
}

fn get_pid(pid_slice:&[u8]) -> u32 {
     
    let mut  pid:u32     = pid_slice[3] as u32;
    pid = pid << 8; pid += pid_slice[2] as u32;
    pid = pid << 8; pid += pid_slice[1] as u32;
    pid = pid << 8; pid += pid_slice[0] as u32;

    pid
   
}

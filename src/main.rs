extern crate libc;
extern crate rupervise;
extern crate rustbox;

#[macro_use]
extern crate lazy_static;

use std::env;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use std::os::unix::fs::OpenOptionsExt;

use rupervise::tai::*;
use rustbox::{RustBox, Color, Key};

mod interpreter;

#[derive(Debug, Copy, Clone)]
enum SvstatError {
    UnableToChDir,
    UnableToStatDown,
    SuperviseNotRunning,
    UnableToOpenSuperviseOk,
    UnableToOpenSuperviseStatus,
    StatusBadFormat,
    StatusOtherError,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum SvWants {
    WantsUp,
    WantsDown,
}

#[derive(Debug, Copy, Clone)]
enum SvstatType {
    SvError(SvstatError),
    SvOk {
        pid: Option<u32>,
        normally_up: bool,
        is_paused: bool,
        duration: u64,
        wants: Option<SvWants>,
    },
}

#[derive(Debug, Clone)]
struct Service {
    name: PathBuf,
    status: Option<SvstatType>,
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.status {
            Some(SvstatType::SvOk { pid: Some(p),
                                    normally_up: nu,
                                    is_paused: ip,
                                    duration: d,
                                    wants: w }) => {
                write!(f, "{}: up (pid {}) {} seconds", self.name.display(), p, d);
                if !nu {
                    write!(f, ", normally down");
                }
                if ip {
                    write!(f, ", paused");
                }
                if w == Some(SvWants::WantsDown) {
                    write!(f, ", want down");
                }
                write!(f, "")

            }
            Some(SvstatType::SvOk { pid: None,
                                    normally_up: nu,
                                    is_paused: ip,
                                    duration: d,
                                    wants: w }) => {
                write!(f, "{}: down {} seconds", self.name.display(), d);
                if nu {
                    write!(f, ", normally up");
                }
                if w == Some(SvWants::WantsUp) {
                    write!(f, ", want up");
                }
                write!(f, "")

            }
            Some(SvstatType::SvError(e)) => write!(f, "{}: {:?}", self.name.display(), e),
            _ => write!(f, "error with service"),
        }
    }
}

#[derive(Debug,Clone)]
enum Mode {
    Normal,
    Command,
}

#[derive(Clone, Debug)]
struct State {
    mode: Mode,
    command: char,
    highlighted: Option<u32>,
    services: Vec<Service>,
}

// impl<'a> State {
//    fn new<'a>(&'a Vec<Service>) -> State {


fn main() {

    let current_path = Path::new(".");
    let original_dir = match current_path.canonicalize() {
        Err(why) => panic!("couldn't open current directory: {}", why.description()),
        Ok(original_dir) => original_dir,
    };

    let args: Vec<String> = env::args().collect();
    let mut services: Vec<Service> = Vec::new();
    let dirs = args.iter().skip(1);
    for dir in dirs {

        let mut service = Service {
            name: PathBuf::from(dir),
            status: None,
        };
        update_supervise(&mut service);
        services.push(service);
        env::set_current_dir(&original_dir).unwrap();

    }

    let rustbox = RustBox::init(Default::default()).unwrap();
    let mut state = State {
        mode: Mode::Normal,
        command: '\0',
        highlighted: None,
        services: services,
    };

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let now = Instant::now();
        for sv in &mut state.services {
            update_supervise(sv);
            env::set_current_dir(&original_dir).unwrap();
        }

        rustbox.draw(&state);
        let timeout = Duration::new(0, 1000000);
        if let rustbox::Event::KeyEvent(mkey) = rustbox.peek_event(timeout, false)
            .ok()
            .expect("poll failed") {
            match mkey {
                Key::Ctrl('c') => break,
                _ => {
                    let new_state = handle_key(mkey, &state);
                    state = new_state;

                }
            }
        }

        // For timing purposes if needed
        // let elapsed = now.elapsed();
        // let seconds = elapsed.as_secs();
        // let nanos = elapsed.subsec_nanos();
    }
}

fn handle_key(key: rustbox::Key, state: &State) -> State {
    match state.mode {
        Mode::Normal => handle_normal_input(key, state),
        Mode::Command => handle_command_input(key, state),
    }
}

fn handle_normal_input(key: rustbox::Key, state: &State) -> State {
    let new_state = match key {

        Key::Char(num) if num.is_digit(10) => highlight_row(state, num.to_digit(10)), 
        Key::Char(c) if is_svc_command(c) => send_command(state, c),
        _ => state.clone(),
    };
    new_state
}

fn is_svc_command(c: char) -> bool {
    match c {
        'i' | 't' | 'd' | 'k' | 'p' | 'c' | 'o' | 'x' => true,
        _ => false,
    }
}

fn handle_command_input(key: rustbox::Key, state: &State) -> State {
    let mut new_state = state.clone();
    new_state
}

fn highlight_row(state: &State, row: Option<u32>) -> State {
    let mut new_state = state.clone();

    if let Some(row) = row {
        new_state.highlighted = Some(row);
    } else {
        new_state.highlighted = None;
    }

    new_state
}

fn send_command(state: &State, command: char) -> State {

    let mut new_state = state.clone();
    new_state.command = command;
    new_state
}
fn open_write<P: AsRef<Path>>(path: P) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
}

fn update_supervise(service: &mut Service) -> &mut Service {
    // let mut service = service;
    if let Err(_) = env::set_current_dir(&service.name) {
        service.status = Some(SvstatType::SvError(SvstatError::UnableToChDir));
        return service;
    }

    let mut normally_up = false;
    if let Err(e) = std::fs::metadata("down") {
        if e.kind() == io::ErrorKind::NotFound {
            normally_up = true;
        } else {
            service.status = Some(SvstatType::SvError(SvstatError::UnableToStatDown));
            return service;
        }
    }

    if let Err(e) = open_write("supervise/ok") {
        if e.kind() == io::ErrorKind::Other {
            service.status = Some(SvstatType::SvError(SvstatError::SuperviseNotRunning));
            return service;
        }
        service.status = Some(SvstatType::SvError(SvstatError::UnableToOpenSuperviseOk));
        return service;
    }

    let mut status_buf: [u8; 18] = [0; 18];
    {
        let mut status_file = match File::open("supervise/status") {
            Ok(status_file) => status_file,
            Err(_) => {
                service.status =
                    Some(SvstatType::SvError(SvstatError::UnableToOpenSuperviseStatus));
                return service;
            }

        };
        let read_bytes = status_file.read(&mut status_buf[..]);
        match read_bytes {
            Ok(n) if n == status_buf.len() => {}
            Ok(_) => {
                service.status = Some(SvstatType::SvError(SvstatError::StatusBadFormat));
                return service;
            } 
            Err(_) => {
                service.status = Some(SvstatType::SvError(SvstatError::StatusOtherError));
                return service;
            }
        };
    }

    let pid = get_pid(&status_buf[12..16]);

    let want = status_buf[17] as char;
    let paused = status_buf[16] as char;

    let mut when = rupervise::tai::unpack(&status_buf[0..8]);
    let now = rupervise::tai::now();

    if now < when {
        when = now;
    }


    service.status = Some(SvstatType::SvOk {
        pid: if pid != 0 { Some(pid) } else { None },
        normally_up: normally_up,
        is_paused: if paused as u8 != 0 { true } else { false },
        duration: now.as_secs() - when.as_secs(),
        wants: match want { 
            'u' => Some(SvWants::WantsUp),
            'd' => Some(SvWants::WantsDown),
            _ => None,
        },
    });

    service
}

fn get_pid(pid_slice: &[u8]) -> u32 {

    let mut pid: u32 = pid_slice[3] as u32;
    pid = pid << 8;
    pid += pid_slice[2] as u32;
    pid = pid << 8;
    pid += pid_slice[1] as u32;
    pid = pid << 8;
    pid += pid_slice[0] as u32;

    pid

}

// struct ServiceState {
//
// }

trait ScreenWriter {
    fn write(&self, x: usize, y: usize, text: &str);
    fn write_inverted(&self, x: usize, y: usize, text: &str);
    fn draw(&self, state: &State);
}

impl ScreenWriter for RustBox {
    fn write(&self, x: usize, y: usize, text: &str) {
        self.print(x, y, rustbox::RB_BOLD, Color::White, Color::Default, text);
    }

    fn write_inverted(&self, x: usize, y: usize, text: &str) {
        self.print(x, y, rustbox::RB_BOLD, Color::Default, Color::White, text);
    }

    fn draw(&self, state: &State) {
        self.clear();
        self.present();

        for (i, service) in state.services.iter().enumerate() {
            let y = i + 1;
            let s = format!("{:>3}: {}", i, service);

            if state.highlighted == Some(i as u32) {
                self.write_inverted(0, y, &s);
            } else {
                self.write(0, y, &s);
            }
        }
        self.write(0,
                   state.services.len() + 1,
                   &format!("Command: {}", state.command));
        self.present();
    }
}

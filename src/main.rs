use std::ffi::{CStr, CString};
use std::env;
use nix::unistd::{self, ForkResult, fork};
use nix::sys::wait::wait;

fn main() {
    let args:Vec<_> = env::args().collect();
    let cmd = &args[1];
    println!("Launching: {}", cmd);
    create_init(cmd);
}

fn create_init(c : &str) {
    unsharenamespaces();
    match unsafe { fork() } { 
        Err(_) => panic!("fork failed"),
        Ok(ForkResult::Child) => init(c),
        Ok(ForkResult::Parent { child, .. }) => {
            wait();
        }
    }
}


fn init(c: &str) {
    let pid = unistd::getpid();
    println!("Init's pid is {}", pid);
    launch_and_wait(c);
    println!("Init died!");
    std::process::exit(1);
}

fn launch_and_wait(c : &str) {
    match unsafe { fork() } {
        Err(_) => panic!("fork failed"),
        Ok(ForkResult::Child) => launch(c),
        Ok(ForkResult::Parent { child, .. }) => { 
           wait(); 
        }
    }
}

fn unsharenamespaces() {
    let flags = libc::CLONE_NEWUTS | libc::CLONE_NEWPID;
    let res = unsafe { libc::unshare(flags) };
    if (res != 0) { panic!("unshare failed!"); }
    let hostname = "container";
    let hostname_r = CString::new(hostname).unwrap();
    unsafe {
        libc::sethostname(hostname_r.as_ptr(), hostname.len());
    }
}

fn launch(c : &str) {
    unsharenamespaces();
    let cmd = CString::new(c).unwrap();
    let args = [cmd];
    unistd::execv(&args[0], &args);
}

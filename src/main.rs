use std::ffi::{CStr, CString};
use std::env;
use nix::unistd::{self, ForkResult, fork};
use nix::sys::wait::wait;

fn main() {
    let mut args:Vec<_> = env::args().collect();
    if let Some(cmd) = args.get(1) { 
        println!("Launching: {}", cmd);
        args.remove(0);
        create_init(args);
    } else { 
        println!("Usage: {} command", args[0]);
        std::process::exit(1);
    }
}

fn create_init(c : Vec<String>) {
    unsharenamespaces();
    match unsafe { fork() } { 
        Err(_) => panic!("fork failed"),
        Ok(ForkResult::Child) => init(c),
        Ok(ForkResult::Parent { child, .. }) => {
            wait();
        }
    }
}


fn init(c: Vec<String>) {
    unsafe { nix::env::clearenv(); }
    let pid = unistd::getpid();
    println!("Init's pid is {}", pid);
    launch_and_wait(c);
    println!("Init died!");
    std::process::exit(1);
}

fn launch_and_wait(c : Vec<String>) {
    match unsafe { fork() } {
        Err(_) => panic!("fork failed"),
        Ok(ForkResult::Child) => launch(c),
        Ok(ForkResult::Parent { child, .. }) => { 
           wait(); 
        }
    }
}

fn unsharenamespaces() {
    let flags = libc::CLONE_NEWUTS | libc::CLONE_NEWPID | libc::CLONE_NEWNS;
    let res = unsafe { libc::unshare(flags) };
    let errmsg = CString::new("unshare()").unwrap();
    if res != 0 {
        unsafe { libc::perror(errmsg.as_ptr()); }
    }
    if (res != 0) { panic!("unshare failed!"); }
    let hostname = "container";
    let hostname_r = CString::new(hostname).unwrap();
    unsafe {
        libc::sethostname(hostname_r.as_ptr(), hostname.len());
    }
}

fn launch(c : Vec<String>) {
    let args : Vec<CString> = c
        .into_iter()
        .map(|s| CString::new(s).unwrap())
        .collect();
    unistd::execv(&args[0], &args);
}

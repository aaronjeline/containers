use std::ffi::{CStr, CString};
use std::env;
use nix::unistd::{self, ForkResult, fork};
use nix::sys::wait::wait;
use nix::mount;

fn main() {
    checkamroot();
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

fn checkamroot() { 
    if !unistd::geteuid().is_root() { 
        panic!("Cannot work without effective root!");
    }
}


fn create_init(c : Vec<String>) {
    unsharenamespaces();
    match unsafe { fork() } { 
        Err(_) => panic!("fork failed"),
        Ok(ForkResult::Child) => init(c),
        Ok(ForkResult::Parent { child, .. }) => {
            wait();
            println!("Container is dead, unmounting fs");
            match cleanup() {
                Ok(()) => (),
                Err(e) => { println!("Cleanup Error!: {}", e);
                    std::process::exit(1);
                }
            };
        }
    }
}

fn cleanup() -> nix::Result<()> {
    nix::mount::umount("root/proc")?;
    nix::mount::umount("root")?;
    rmdir("rootfs/oldfs")?;
    Ok(())
}

// Safe wrapper for libc rmdir
fn rmdir(path : &str) -> nix::Result<()> { 
    let path_c = CString::new(path).unwrap();
    let response : libc::c_int;
    unsafe {
        response = libc::rmdir(path_c.as_ptr());
    }

    if response == 0 { 
        Ok(())
    } else { 
        let no = unsafe { *(libc::__errno_location()) };
        let errno = nix::errno::Errno::from_i32(no);
        Err(nix::Error::Sys(errno))
    }

}


fn init(c: Vec<String>) {
    let pid = unistd::getpid();
    match create_root() { 
        Ok(()) => {
            setup_env();
            println!("Init's pid is {}", pid);
            launch_and_wait(c);
            println!("Init died!");
            std::process::exit(0);
        }, 
        Err(e) => {
            println!("Failed to create root filesystem: {}", e);
            std::process::exit(1);
        }
    }
}

fn setup_env() {
    unsafe { nix::env::clearenv(); }
    env::set_var("PATH", "/sbin:/bin/:/usr/bin/:/usr/sbin");
    env::set_var("TERM", "xterm-256color");
}

fn create_root() -> nix::Result<()> { 
    // Path to our root directory
    let flags = mount::MsFlags::MS_BIND | mount::MsFlags::MS_PRIVATE;
    let none : Option<&str> = None;
    mount::mount(Some("rootfs"), "root", none, flags, none)?;
    // Mount proc 
    let empty_flags = mount::MsFlags::empty();
    mount::mount(Some("proc"), "root/proc", Some("proc"), empty_flags, none)?;
    let userall = nix::sys::stat::Mode::S_IRWXU;
    unistd::mkdir("rootfs/oldfs", userall);
    println!("Pivoting root");
    //unistd::pivot_root("./root", "./root/oldfs")?;
    //unistd::chdir("/")?;
    Ok(())
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
    // this is a hack and should be pivot root
    // as is it is probably easy to escape?
    unistd::chroot("root").unwrap();
    unistd::chdir("/");
    let args : Vec<CString> = c
        .into_iter()
        .map(|s| CString::new(s).unwrap())
        .collect();
    unistd::execv(&args[0], &args).unwrap();
}

use std::ffi::{CStr, CString};
use std::env;
use nix::unistd::{self, ForkResult, fork};
use nix::sys::wait::wait;
use nix::mount;
use nix::sys::stat::*;
use lazy_static::*;

type ProofResult = nix::Result<IsolationProof>;

fn main() {
    checkamroot();
    let mut args:Vec<_> = env::args().collect();
    if let Some(cmd) = args.get(1) { 
        if cmd == "run" {
            args.remove(0);
            args.remove(0);
            create_init(args);
        } else {
            printusage(&args[0]);
        }
    } else { 
        printusage(&args[0]);
    }
}

fn printusage(path : &str) {
    println!("Usage:");
    println!("{} run <cmd>", path);
    println!("{} exec <id> <cmd>", path);
    std::process::exit(1);
}

fn checkamroot() { 
    if !unistd::geteuid().is_root() { 
        panic!("Cannot work without effective root!");
    }
}


fn create_init(c : Vec<String>) -> nix::Result<()> {
    let proof = isolation::enter_namespace()?;
    match unsafe { fork() } { 
        Err(_) => panic!("fork failed"),
        Ok(ForkResult::Child) => { init(c, proof); Ok(()) },
        Ok(ForkResult::Parent { child, .. }) => {
            println!("Container id: {}", child);
            wait();
            println!("Container is dead, unmounting fs");
            match cleanup() {
                Ok(()) => (),
                Err(e) => { println!("Cleanup Error!: {}", e);
                    std::process::exit(1);
                }
            };
            Ok(())
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


fn init(c: Vec<String>, np : NamespaceProof) -> nix::Result<()> {
    let pid = unistd::getpid();
    let proof = isolate_fs(np)?;
    let proof = cleanup_dev(proof);
    setup_env();
    let proof = setup_dev(proof)?;
    println!("Init's pid is {}", pid);
    let proof = launch_and_wait(c, proof);
    println!("Init died!");
    let proof = cleanup_dev(proof);
    Ok(())
}

lazy_static! {
    static ref devs : Vec<(&'static str, dev_t)> = 
        vec![("/dev/null", makedev(1,3)),
             ("/dev/zero", makedev(1,5)),
             ("/dev/random", makedev(1,8)),
             ("/dev/urandom", makedev(1,9))];
}

//Create /dev/null and /dev/zero
fn setup_dev(p : IsolationProof) -> ProofResult {
    let access = Mode::from_bits(0o666).unwrap();

    for (name, dev) in devs.iter() {
        mknod(*name, SFlag::S_IFCHR, access, *dev)?;
    }
    
    Ok(p)
}

fn cleanup_dev(p : IsolationProof) -> IsolationProof {
    for (name, _) in devs.iter() { 
        unistd::unlink(*name);
    }
    p
}

fn setup_env() {
    unsafe { nix::env::clearenv(); }
    env::set_var("PATH", "/sbin:/bin/:/usr/bin/:/usr/sbin");
    env::set_var("TERM", "xterm-256color");
}

mod isolation { 
    use std::ffi::{CStr, CString};
    use nix::{unistd, mount};

    // This zero sized type is a proof that we entered isoloation
    pub struct IsolationProof {}
    pub struct NamespaceProof {}

    pub fn isolate_fs(p : NamespaceProof) -> nix::Result<IsolationProof> {
        IsolationProof::isolate_fs(p)
    }

    impl IsolationProof { 
        pub fn isolate_fs(_ : NamespaceProof) -> nix::Result<Self> { 
            // Path to our root directory
            let flags = mount::MsFlags::MS_BIND | mount::MsFlags::MS_PRIVATE;
            let none : Option<&str> = None;
            mount::mount(Some("rootfs"), "root", none, flags, none)?;
            // Mount proc 
            let empty_flags = mount::MsFlags::empty();
            mount::mount(Some("proc"), "root/proc", 
                         Some("proc"), empty_flags, none)?;
            let userall = nix::sys::stat::Mode::S_IRWXU;
            unistd::mkdir("rootfs/oldfs", userall);
            //unistd::pivot_root("./root", "./root/oldfs")?;
            //unistd::chdir("/")?;
            unistd::chroot("root").unwrap();
            unistd::chdir("/");
            Ok(IsolationProof {})
        }
    }

    impl NamespaceProof {
        pub fn enter_namespace() -> nix::Result<Self> {
            let flags = 
             libc::CLONE_NEWUTS | libc::CLONE_NEWPID | libc::CLONE_NEWNS;
            let res = unsafe { libc::unshare(flags) };
            let errmsg = CString::new("unshare()").unwrap();
            if res != 0 {
                return Err(nix::Error::Sys(nix::errno::Errno::last()));
            }
            let hostname = "container";
            let hostname_r = CString::new(hostname).unwrap();
            unsafe {
                libc::sethostname(hostname_r.as_ptr(), hostname.len());
            }
            Ok(Self {})
        }
    }

    pub fn enter_namespace() -> nix::Result<NamespaceProof> {
        NamespaceProof::enter_namespace()
    }

}
use isolation::*;


fn launch_and_wait(c : Vec<String>, p : IsolationProof) -> IsolationProof {
    match unsafe { fork() } {
        Err(_) => panic!("fork failed"),
        Ok(ForkResult::Child) => launch(c, p),
        Ok(ForkResult::Parent { child, .. }) => { 
           wait(); 
           p
        }
    }
}


fn launch(c : Vec<String>, _ : IsolationProof) -> ! {
    // this is a hack and should be using pivot root
    let args : Vec<CString> = c
        .into_iter()
        .map(|s| CString::new(s).unwrap())
        .collect();
    match unistd::execv(&args[0], &args) { 
        Ok(_) => { panic!("Impossible"); },
        Err(e) => { 
            println!("Failed to launch {:?}", args);
            println!("Errno: {}", e);
            std::process::exit(1);
        }
    }
}

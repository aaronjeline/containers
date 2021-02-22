# Containers
A simple container runtime written in rust! Just for fun

Status: [![forthebadge](https://forthebadge.com/images/badges/works-on-my-machine.svg)](https://forthebadge.com)

## How to use
Run the script `setuproot.sh`. This will donwload an alpine linux image, and setup the directorty structure needed.
The executable must be run as root.

### Running a contianer
The `run` subcommand will create a new container

Example: `./containers run /bin/bash`
Containers will output their ID (just the PID of the init process)

### Executing a command in an already running container
The `exec` subcommand executes the following command in an existing container. Specify the contaienr by PID.

Example: `./containers exec 213 /bin/bash`


## TODO 
- [ ] Properly isolate the filesystem
- [ ] Isolate the network
- [ ] Drop capabilities so that root in the container is powerless
- [ ] Add cgroups to restrict containers resource access

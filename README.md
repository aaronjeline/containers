# Containers
A simple container runtime written in rust! Just for fun

Status: [![forthebadge](https://forthebadge.com/images/badges/works-on-my-machine.svg)](https://forthebadge.com)

## How to use
Run the script `setuproot.sh`. This will donwload an alpine linux image, and setup the directorty structure needed.

Run the script `launch.sh` to spawn the container with `/bin/bash`.
Or run the executable directly with any command.

## TODO 
- [ ] Properly isolate the filesystem
- [ ] Isolate the network
- [ ] Drop capabilities so that root in the container is powerless
- [ ] Add cgroups to restrict containers resource access

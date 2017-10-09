/* External functions for calling sendmsg() and recvmsg().
 */
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/socket.h>

#define RECV_TIMEOUT 2

union cmsghdr_buf {
	struct cmsghdr align;
	char buf[CMSG_SPACE(sizeof(int))];
};

static int get_timeout(int socket) {
	struct timeval buf = { 0 };
	socklen_t buf_len = sizeof(buf);

	/* Get socket options */
	errno = 0;
	int ret = getsockopt(socket, SOL_SOCKET, SO_RCVTIMEO, &buf, &buf_len);
	if (ret < 0 || errno) {
		perror("getsockopt(): %s");
		return ret;
	}

	return buf.tv_sec;
}

static void set_timeout(int socket, int timeout) {
	/* Configure options */
	const struct timeval value = {
		.tv_sec = timeout,
		.tv_usec = 0,
	};

	/* Set socket options */
	errno = 0;
	int ret = setsockopt(socket, SOL_SOCKET, SO_RCVTIMEO, &value, sizeof(value));
	if (ret < 0 || errno) {
		perror("setsockopt(): %s");
	}
}

ssize_t open_sendmsg_err(int socket) {
	struct msghdr msg = { 0 };

	/* Allocate scatter-gather locations */
	struct iovec iov = { .iov_base = "\x01", .iov_len = 1 };
	msg.msg_iov = &iov;
	msg.msg_iovlen = 1;

	/* Allocate access control messages */
	union cmsghdr_buf u = { 0 };
	msg.msg_control = u.buf;
	msg.msg_controllen = sizeof(u.buf);
	struct cmsghdr *cmsgp = CMSG_FIRSTHDR(&msg);
	cmsgp->cmsg_len = CMSG_LEN(0);
	msg.msg_controllen = sizeof(u);

	/* Send error */
	errno = 0;
	return sendmsg(socket, &msg, 0);
}

int open_sendmsg(int socket, int fd) {
	struct msghdr msg = { 0 };

	/* Allocate scatter-gather locations */
	struct iovec iov = { .iov_base = "\x00", .iov_len = 1 };
	msg.msg_iov = &iov;
	msg.msg_iovlen = 1;

	/* Allocate access control messages */
	union cmsghdr_buf u = { 0 };
	msg.msg_control = u.buf;
	msg.msg_controllen = sizeof(u.buf);
	struct cmsghdr *cmsgp = CMSG_FIRSTHDR(&msg);

	/* Configure access control message to send fd */
	cmsgp->cmsg_len = CMSG_LEN(sizeof(fd));
	cmsgp->cmsg_level = SOL_SOCKET;
	cmsgp->cmsg_type = SCM_RIGHTS;
	*((int *)CMSG_DATA(cmsgp)) = fd;
	msg.msg_controllen = sizeof(u);

	/* Send fd */
	errno = 0;
	return sendmsg(socket, &msg, 0);
}

int open_recvmsg(int socket, int *fd) {
	struct msghdr msg = { 0 };

	/* Check arguments */
	if (fd == NULL) {
		errno = EINVAL;
		return -1;
	}

	/* Allocate scatter-gather locations */
	char iov_buf[1] = { -1 };
	struct iovec iov = { .iov_base = &iov_buf, .iov_len = sizeof(iov_buf) };
	msg.msg_iov = &iov;
	msg.msg_iovlen = 1;

	/* Allocate access control messages */
	char buf[sizeof(union cmsghdr_buf)];
	msg.msg_control = buf;
	msg.msg_controllen = sizeof(buf);

	/* Set a new socket timeout */
	int timeout = get_timeout(socket);
	if (timeout < 0) {
		/* Assume the socket didn't have a timeout */
		timeout = 0;
	}
	set_timeout(socket, RECV_TIMEOUT);

	/* Wait for fd and restore the old socket timeout */
	ssize_t ret = recvmsg(socket, &msg, 0);
	int saved_errno = errno;
	set_timeout(socket, timeout);
	errno = saved_errno;
	if (ret < 0 || errno) {
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
	*fd = *((int *)CMSG_DATA(cmsgp));

	return 0;
}

/* External functions for calling seccomp().
 */
#include <errno.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/prctl.h>
#include <linux/audit.h>
#include <linux/filter.h>
#include <linux/seccomp.h>

#define WHITELIST_SYSCALL(name) \
	BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, __NR_##name, 0, 1), \
	BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_ALLOW)

int32_t sandbox() {
	struct sock_filter filter[] = {
		/* Validate architecture */
		BPF_STMT(BPF_LD | BPF_W | BPF_ABS, (offsetof(struct seccomp_data, arch))),
		BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, AUDIT_ARCH_X86_64, 1, 0),
		BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_KILL),

		/* Load the syscall */
		BPF_STMT(BPF_LD | BPF_W | BPF_ABS, (offsetof(struct seccomp_data, nr))),

		/* Whitelist */
		/* NOTE: This whitelist is _way_ too permissive. Realistically it only
		 *       needs about 6 different syscalls given the expected
		 *       functionality. It's based on the default docker policy without
		 *       argument filtering, with a few added and a few missing)...but
		 *       maybe that's on purpose? :)
		 */
		WHITELIST_SYSCALL(read),
		WHITELIST_SYSCALL(write),
		WHITELIST_SYSCALL(open),
		WHITELIST_SYSCALL(close),
		WHITELIST_SYSCALL(stat),
		WHITELIST_SYSCALL(fstat),
		WHITELIST_SYSCALL(lstat),
		WHITELIST_SYSCALL(poll),
		WHITELIST_SYSCALL(lseek),
		WHITELIST_SYSCALL(mmap),
		WHITELIST_SYSCALL(mprotect),
		WHITELIST_SYSCALL(munmap),
		WHITELIST_SYSCALL(brk),
		WHITELIST_SYSCALL(rt_sigaction),
		WHITELIST_SYSCALL(rt_sigprocmask),
		WHITELIST_SYSCALL(rt_sigreturn),
		WHITELIST_SYSCALL(ioctl),
		WHITELIST_SYSCALL(pread64),
		WHITELIST_SYSCALL(pwrite64),
		WHITELIST_SYSCALL(readv),
		WHITELIST_SYSCALL(writev),
		WHITELIST_SYSCALL(access),
		WHITELIST_SYSCALL(pipe),
		WHITELIST_SYSCALL(select),
		WHITELIST_SYSCALL(sched_yield),
		WHITELIST_SYSCALL(mremap),
		WHITELIST_SYSCALL(msync),
		WHITELIST_SYSCALL(mincore),
		WHITELIST_SYSCALL(madvise),
		WHITELIST_SYSCALL(shmget),
		WHITELIST_SYSCALL(shmat),
		WHITELIST_SYSCALL(shmctl),
		WHITELIST_SYSCALL(dup),
		WHITELIST_SYSCALL(dup2),
		WHITELIST_SYSCALL(pause),
		WHITELIST_SYSCALL(nanosleep),
		WHITELIST_SYSCALL(getitimer),
		WHITELIST_SYSCALL(alarm),
		WHITELIST_SYSCALL(setitimer),
		WHITELIST_SYSCALL(getpid),
		WHITELIST_SYSCALL(sendfile),
		WHITELIST_SYSCALL(socket),
		WHITELIST_SYSCALL(connect),
		WHITELIST_SYSCALL(accept),
		WHITELIST_SYSCALL(sendto),
		WHITELIST_SYSCALL(recvfrom),
		WHITELIST_SYSCALL(sendmsg),
		WHITELIST_SYSCALL(recvmsg),
		WHITELIST_SYSCALL(shutdown),
		//WHITELIST_SYSCALL(bind),
		WHITELIST_SYSCALL(listen),
		WHITELIST_SYSCALL(getsockname),
		WHITELIST_SYSCALL(getpeername),
		WHITELIST_SYSCALL(socketpair),
		WHITELIST_SYSCALL(setsockopt),
		WHITELIST_SYSCALL(getsockopt),
		//WHITELIST_SYSCALL(clone),
		WHITELIST_SYSCALL(fork),
		WHITELIST_SYSCALL(vfork),
		WHITELIST_SYSCALL(execve),
		WHITELIST_SYSCALL(exit),
		WHITELIST_SYSCALL(wait4),
		WHITELIST_SYSCALL(kill),
		WHITELIST_SYSCALL(uname),
		WHITELIST_SYSCALL(semget),
		WHITELIST_SYSCALL(semop),
		WHITELIST_SYSCALL(semctl),
		WHITELIST_SYSCALL(shmdt),
		WHITELIST_SYSCALL(msgget),
		WHITELIST_SYSCALL(msgsnd),
		WHITELIST_SYSCALL(msgrcv),
		WHITELIST_SYSCALL(msgctl),
		WHITELIST_SYSCALL(fcntl),
		WHITELIST_SYSCALL(flock),
		WHITELIST_SYSCALL(fsync),
		WHITELIST_SYSCALL(fdatasync),
		WHITELIST_SYSCALL(truncate),
		WHITELIST_SYSCALL(ftruncate),
		WHITELIST_SYSCALL(getdents),
		WHITELIST_SYSCALL(getcwd),
		WHITELIST_SYSCALL(chdir),
		WHITELIST_SYSCALL(fchdir),
		WHITELIST_SYSCALL(rename),
		WHITELIST_SYSCALL(mkdir),
		WHITELIST_SYSCALL(rmdir),
		WHITELIST_SYSCALL(creat),
		WHITELIST_SYSCALL(link),
		WHITELIST_SYSCALL(unlink),
		WHITELIST_SYSCALL(symlink),
		WHITELIST_SYSCALL(readlink),
		WHITELIST_SYSCALL(chmod),
		WHITELIST_SYSCALL(fchmod),
		WHITELIST_SYSCALL(chown),
		WHITELIST_SYSCALL(fchown),
		WHITELIST_SYSCALL(lchown),
		WHITELIST_SYSCALL(umask),
		WHITELIST_SYSCALL(gettimeofday),
		WHITELIST_SYSCALL(getrlimit),
		WHITELIST_SYSCALL(getrusage),
		WHITELIST_SYSCALL(sysinfo),
		WHITELIST_SYSCALL(times),
		//WHITELIST_SYSCALL(ptrace),
		WHITELIST_SYSCALL(getuid),
		WHITELIST_SYSCALL(syslog),
		WHITELIST_SYSCALL(getgid),
		WHITELIST_SYSCALL(setuid),
		WHITELIST_SYSCALL(setgid),
		WHITELIST_SYSCALL(geteuid),
		WHITELIST_SYSCALL(getegid),
		WHITELIST_SYSCALL(setpgid),
		WHITELIST_SYSCALL(getppid),
		WHITELIST_SYSCALL(getpgrp),
		WHITELIST_SYSCALL(setsid),
		WHITELIST_SYSCALL(setreuid),
		WHITELIST_SYSCALL(setregid),
		WHITELIST_SYSCALL(getgroups),
		WHITELIST_SYSCALL(setgroups),
		WHITELIST_SYSCALL(setresuid),
		WHITELIST_SYSCALL(getresuid),
		WHITELIST_SYSCALL(setresgid),
		WHITELIST_SYSCALL(getresgid),
		WHITELIST_SYSCALL(getpgid),
		WHITELIST_SYSCALL(setfsuid),
		WHITELIST_SYSCALL(setfsgid),
		WHITELIST_SYSCALL(getsid),
		WHITELIST_SYSCALL(capget),
		WHITELIST_SYSCALL(capset),
		WHITELIST_SYSCALL(rt_sigpending),
		WHITELIST_SYSCALL(rt_sigtimedwait),
		WHITELIST_SYSCALL(rt_sigqueueinfo),
		WHITELIST_SYSCALL(rt_sigsuspend),
		WHITELIST_SYSCALL(sigaltstack),
		WHITELIST_SYSCALL(utime),
		WHITELIST_SYSCALL(mknod),
		//WHITELIST_SYSCALL(uselib),
		//WHITELIST_SYSCALL(personality),
		//WHITELIST_SYSCALL(ustat),
		WHITELIST_SYSCALL(statfs),
		WHITELIST_SYSCALL(fstatfs),
		//WHITELIST_SYSCALL(sysfs),
		WHITELIST_SYSCALL(getpriority),
		WHITELIST_SYSCALL(setpriority),
		WHITELIST_SYSCALL(sched_setparam),
		WHITELIST_SYSCALL(sched_getparam),
		WHITELIST_SYSCALL(sched_setscheduler),
		WHITELIST_SYSCALL(sched_getscheduler),
		WHITELIST_SYSCALL(sched_get_priority_max),
		WHITELIST_SYSCALL(sched_get_priority_min),
		WHITELIST_SYSCALL(sched_rr_get_interval),
		WHITELIST_SYSCALL(mlock),
		WHITELIST_SYSCALL(munlock),
		WHITELIST_SYSCALL(mlockall),
		WHITELIST_SYSCALL(munlockall),
		//WHITELIST_SYSCALL(vhangup),
		//WHITELIST_SYSCALL(modify_ldt),
		//WHITELIST_SYSCALL(pivot_root),
		//WHITELIST_SYSCALL(_sysctl),
		WHITELIST_SYSCALL(prctl),  // XXX: Should probably be blocked, but there's a chicken and egg with dropping capabilities
		//WHITELIST_SYSCALL(arch_prctl),
		WHITELIST_SYSCALL(adjtimex),
		WHITELIST_SYSCALL(setrlimit),
		//WHITELIST_SYSCALL(chroot),
		WHITELIST_SYSCALL(sync),
		//WHITELIST_SYSCALL(acct),
		//WHITELIST_SYSCALL(settimeofday),
		//WHITELIST_SYSCALL(mount),
		//WHITELIST_SYSCALL(umount2),
		//WHITELIST_SYSCALL(swapon),
		//WHITELIST_SYSCALL(swapoff),
		//WHITELIST_SYSCALL(reboot),
		//WHITELIST_SYSCALL(sethostname),
		//WHITELIST_SYSCALL(setdomainname),
		//WHITELIST_SYSCALL(iopl),
		//WHITELIST_SYSCALL(ioperm),
		//WHITELIST_SYSCALL(create_module),
		//WHITELIST_SYSCALL(init_module),
		//WHITELIST_SYSCALL(delete_module),
		//WHITELIST_SYSCALL(get_kernel_syms),
		//WHITELIST_SYSCALL(query_module),
		//WHITELIST_SYSCALL(quotactl),
		//WHITELIST_SYSCALL(nfsservctl),
		//WHITELIST_SYSCALL(getpmsg),
		//WHITELIST_SYSCALL(putpmsg),
		//WHITELIST_SYSCALL(afs_syscall),
		//WHITELIST_SYSCALL(tuxcall),
		//WHITELIST_SYSCALL(security),
		WHITELIST_SYSCALL(gettid),
		WHITELIST_SYSCALL(readahead),
		WHITELIST_SYSCALL(setxattr),
		WHITELIST_SYSCALL(lsetxattr),
		WHITELIST_SYSCALL(fsetxattr),
		WHITELIST_SYSCALL(getxattr),
		WHITELIST_SYSCALL(lgetxattr),
		WHITELIST_SYSCALL(fgetxattr),
		WHITELIST_SYSCALL(listxattr),
		WHITELIST_SYSCALL(llistxattr),
		WHITELIST_SYSCALL(flistxattr),
		WHITELIST_SYSCALL(removexattr),
		WHITELIST_SYSCALL(lremovexattr),
		WHITELIST_SYSCALL(fremovexattr),
		WHITELIST_SYSCALL(tkill),
		WHITELIST_SYSCALL(time),
		WHITELIST_SYSCALL(futex),
		WHITELIST_SYSCALL(sched_setaffinity),
		WHITELIST_SYSCALL(sched_getaffinity),
		WHITELIST_SYSCALL(set_thread_area),
		WHITELIST_SYSCALL(io_setup),
		WHITELIST_SYSCALL(io_destroy),
		WHITELIST_SYSCALL(io_getevents),
		WHITELIST_SYSCALL(io_submit),
		WHITELIST_SYSCALL(io_cancel),
		WHITELIST_SYSCALL(get_thread_area),
		//WHITELIST_SYSCALL(lookup_dcookie),
		WHITELIST_SYSCALL(epoll_create),
		WHITELIST_SYSCALL(epoll_ctl_old),
		WHITELIST_SYSCALL(epoll_wait_old),
		WHITELIST_SYSCALL(remap_file_pages),
		WHITELIST_SYSCALL(getdents64),
		WHITELIST_SYSCALL(set_tid_address),
		WHITELIST_SYSCALL(restart_syscall),
		WHITELIST_SYSCALL(semtimedop),
		WHITELIST_SYSCALL(fadvise64),
		WHITELIST_SYSCALL(timer_create),
		WHITELIST_SYSCALL(timer_settime),
		WHITELIST_SYSCALL(timer_gettime),
		WHITELIST_SYSCALL(timer_getoverrun),
		WHITELIST_SYSCALL(timer_delete),
		//WHITELIST_SYSCALL(clock_settime),
		WHITELIST_SYSCALL(clock_gettime),
		WHITELIST_SYSCALL(clock_getres),
		WHITELIST_SYSCALL(clock_nanosleep),
		WHITELIST_SYSCALL(exit_group),
		WHITELIST_SYSCALL(epoll_wait),
		WHITELIST_SYSCALL(epoll_ctl),
		WHITELIST_SYSCALL(tgkill),
		WHITELIST_SYSCALL(utimes),
		//WHITELIST_SYSCALL(vserver),
		//WHITELIST_SYSCALL(mbind),
		//WHITELIST_SYSCALL(set_mempolicy),
		//WHITELIST_SYSCALL(get_mempolicy),
		WHITELIST_SYSCALL(mq_open),
		WHITELIST_SYSCALL(mq_unlink),
		WHITELIST_SYSCALL(mq_timedsend),
		WHITELIST_SYSCALL(mq_timedreceive),
		WHITELIST_SYSCALL(mq_notify),
		WHITELIST_SYSCALL(mq_getsetattr),
		//WHITELIST_SYSCALL(kexec_load),
		WHITELIST_SYSCALL(waitid),
		//WHITELIST_SYSCALL(add_key),
		//WHITELIST_SYSCALL(request_key),
		//WHITELIST_SYSCALL(keyctl),
		WHITELIST_SYSCALL(ioprio_set),
		WHITELIST_SYSCALL(ioprio_get),
		WHITELIST_SYSCALL(inotify_init),
		WHITELIST_SYSCALL(inotify_add_watch),
		WHITELIST_SYSCALL(inotify_rm_watch),
		//WHITELIST_SYSCALL(migrate_pages),
		WHITELIST_SYSCALL(openat),
		WHITELIST_SYSCALL(mkdirat),
		WHITELIST_SYSCALL(mknodat),
		WHITELIST_SYSCALL(fchownat),
		WHITELIST_SYSCALL(futimesat),
		WHITELIST_SYSCALL(newfstatat),
		WHITELIST_SYSCALL(unlinkat),
		WHITELIST_SYSCALL(renameat),
		WHITELIST_SYSCALL(linkat),
		WHITELIST_SYSCALL(symlinkat),
		WHITELIST_SYSCALL(readlinkat),
		WHITELIST_SYSCALL(fchmodat),
		WHITELIST_SYSCALL(faccessat),
		WHITELIST_SYSCALL(pselect6),
		WHITELIST_SYSCALL(ppoll),
		//WHITELIST_SYSCALL(unshare),
		WHITELIST_SYSCALL(set_robust_list),
		WHITELIST_SYSCALL(get_robust_list),
		WHITELIST_SYSCALL(splice),
		WHITELIST_SYSCALL(tee),
		WHITELIST_SYSCALL(sync_file_range),
		WHITELIST_SYSCALL(vmsplice),
		//WHITELIST_SYSCALL(move_pages),
		WHITELIST_SYSCALL(utimensat),
		WHITELIST_SYSCALL(epoll_pwait),
		WHITELIST_SYSCALL(signalfd),
		WHITELIST_SYSCALL(timerfd_create),
		WHITELIST_SYSCALL(eventfd),
		WHITELIST_SYSCALL(fallocate),
		WHITELIST_SYSCALL(timerfd_settime),
		WHITELIST_SYSCALL(timerfd_gettime),
		WHITELIST_SYSCALL(accept4),
		WHITELIST_SYSCALL(signalfd4),
		WHITELIST_SYSCALL(eventfd2),
		WHITELIST_SYSCALL(epoll_create1),
		WHITELIST_SYSCALL(dup3),
		WHITELIST_SYSCALL(pipe2),
		WHITELIST_SYSCALL(inotify_init1),
		WHITELIST_SYSCALL(preadv),
		WHITELIST_SYSCALL(pwritev),
		WHITELIST_SYSCALL(rt_tgsigqueueinfo),
		//WHITELIST_SYSCALL(perf_event_open),
		WHITELIST_SYSCALL(recvmmsg),
		//WHITELIST_SYSCALL(fanotify_init),
		WHITELIST_SYSCALL(fanotify_mark),
		WHITELIST_SYSCALL(prlimit64),
		WHITELIST_SYSCALL(name_to_handle_at),
		WHITELIST_SYSCALL(open_by_handle_at),
		//WHITELIST_SYSCALL(clock_adjtime),
		WHITELIST_SYSCALL(syncfs),
		WHITELIST_SYSCALL(sendmmsg),
		//WHITELIST_SYSCALL(setns),
		WHITELIST_SYSCALL(getcpu),
		//WHITELIST_SYSCALL(process_vm_readv),
		//WHITELIST_SYSCALL(process_vm_writev),
		//WHITELIST_SYSCALL(kcmp),
		//WHITELIST_SYSCALL(finit_module),
		WHITELIST_SYSCALL(sched_setattr),
		WHITELIST_SYSCALL(sched_getattr),
		WHITELIST_SYSCALL(renameat2),
		//WHITELIST_SYSCALL(seccomp),
		WHITELIST_SYSCALL(getrandom),
		WHITELIST_SYSCALL(memfd_create),
		//WHITELIST_SYSCALL(kexec_file_load),
		//WHITELIST_SYSCALL(bpf),
		WHITELIST_SYSCALL(execveat),
		//WHITELIST_SYSCALL(userfaultfd),
		//WHITELIST_SYSCALL(membarrier),
		WHITELIST_SYSCALL(mlock2),
		WHITELIST_SYSCALL(copy_file_range),
		WHITELIST_SYSCALL(preadv2),
		WHITELIST_SYSCALL(pwritev2),
		//WHITELIST_SYSCALL(pkey_mprotect),
		//WHITELIST_SYSCALL(pkey_alloc),
		//WHITELIST_SYSCALL(pkey_free),
		//WHITELIST_SYSCALL(statx),

		/* Otherwise deny */
		BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_KILL)
	};

	struct sock_fprog prog = {
		.len = (unsigned short) (sizeof(filter) / sizeof(filter[0])),
		.filter = filter,
	};

	return prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog);
}

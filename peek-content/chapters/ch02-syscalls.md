# Chapter 2: syscalls

A syscall is a controlled doorway between an unprivileged program and the
kernel. The CPU treats this doorway specially: when the program executes the
syscall instruction, the hardware switches to privileged mode and jumps to a
fixed kernel entry point. The kernel then reads which service was asked for,
checks the arguments, does the work, and returns control to the program at the
instruction after the syscall.

## the registers, on Linux x86_64

By convention on x86_64 Linux, the program puts the syscall number in `rax`,
the first six arguments in `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9`, and then
executes the `syscall` instruction. When control returns, `rax` holds the
return value (or a negative errno on failure). On AArch64 the registers are
different (`x8` for the syscall number, `x0..x5` for arguments, `svc #0` for
the trap) but the shape is the same.

## the syscall numbers you see most

You will rarely write `syscall` by hand; you will call libc wrappers. But
knowing which numbers correspond to what helps when you read traces.

- `read(fd, buf, count)`: copy up to `count` bytes from the file descriptor's
  current offset into `buf`. Returns the number of bytes read, 0 at end of
  file, or -1 on error.
- `write(fd, buf, count)`: copy up to `count` bytes from `buf` into the file
  descriptor. Returns the number of bytes written.
- `open(path, flags, mode)`: ask the kernel to look up `path`, check the
  permissions against the calling process, and hand back a file descriptor on
  success. The earliest place that permission checks can fail.
- `close(fd)`: tell the kernel "I am done with this file descriptor." After
  this call, the number can be reused by the next `open`.
- `mmap(addr, len, prot, flags, fd, offset)`: ask for a region of virtual
  memory. We talk about this in chapter 3.
- `fork()`, `execve()`, `wait4()`: the process-management trio. fork makes a
  copy, execve replaces the current process image, wait4 reaps a child.

## errors and errno

A syscall returning -1 (or a negative value) is a contract: the actual error
sits in the per-thread `errno` integer. When you read kernel source you will
see `return -EACCES;`; when you read the libc side you will see `errno = EACCES;
return -1;`. Same content, different sign convention.

## the cost of a syscall

A syscall is not free. The CPU has to flush some pipeline state, change
privilege level, save user-mode registers, and (on modern CPUs with mitigation
for cross-privilege side-channel attacks) potentially flush some caches. The
cost is small (hundreds of nanoseconds on a current x86_64 box), but it is not
zero. This is why programs that need to write a lot of small bits sometimes
buffer them up in user space and only call `write` when the buffer is full.

## strace, the friendly observer

You can watch any program's syscall stream with `strace ./yourprogram`. This
is one of the most valuable habits you can build. When something is mysterious,
the syscalls almost always tell the truth.

## key terms

- syscall instruction
- syscall number
- errno
- read, write, open, close
- mmap
- fork, execve
- privilege level
- strace

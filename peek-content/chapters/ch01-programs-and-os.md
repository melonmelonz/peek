# Chapter 1: programs and the operating system

A program is a list of instructions a CPU is willing to follow. By itself, a
program is greedy: it would happily decide what runs on which core, where its
memory lives, and which devices it owns. If two programs both insist on owning
the same speaker at the same time, the speaker has no way to arbitrate.

The operating system is the program whose entire job is arbitration. It
multiplexes hardware (CPU time, RAM, the disk, the network card, the audio
device) across many other programs that do not know about each other. When you
type into one window and your music keeps playing in another, an OS is the
reason that works.

## what the kernel actually owns

We use the word "kernel" for the part of the OS that runs in a privileged CPU
mode and has direct access to the hardware. Everything else (your shell, your
browser, the program you are writing) runs in an unprivileged mode and asks the
kernel to do hardware-touching things on its behalf. The kernel owns:

- the page tables that decide which addresses your program can read or write
- the file descriptor table that tracks which files and sockets your program
  has open
- the scheduler queue that decides which thread runs on which CPU core next
- the interrupt vectors that decide what happens when a device says "I have new
  data for you"

Your program owns its own memory inside the boundaries the kernel set, its own
registers while it is running, and a small chunk of state inside the kernel
that says "this is the program with pid 1532, running as uid 1000, with these
files open."

## the boundary, in one sentence

A program asks the kernel for things by performing a *system call*. The CPU has
a special instruction (`syscall` on x86_64, `svc` on AArch64) that traps from
unprivileged to privileged mode in a controlled way. The kernel reads which
service was requested, validates the arguments, does the thing, and returns.

Everything you see a program do that touches anything outside its own address
space is, in the end, a system call. Reading a file is a syscall. Sending a
packet is a syscall. Asking what time it is is, on most systems, a syscall (or
a careful imitation of one through `vDSO`, which is a different chapter).

## why this matters for the rest of PEEK

The rest of this curriculum traces, layer by layer, what happens between
"the program made a request" and "the hardware finished serving it." Chapter 2
covers the syscall boundary itself. Chapter 3 covers memory: how the kernel
hands a contiguous-looking address space to a program even when the underlying
RAM is scattered across the machine. Later chapters cover device drivers,
interrupts, and finally a small bare-metal program of our own.

You can think of the creature beside this chapter as a small embodiment of the
material you have read. It needs to be fed correct answers; the more of the
chapter you understand, the longer it stays tethered.

## key terms

- program
- operating system
- kernel
- privileged mode
- unprivileged mode
- system call
- file descriptor
- page table
- scheduler
- pid

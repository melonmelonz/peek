# Chapter 3: memory and mmap

A modern program sees a contiguous address space that goes from low addresses
to very high addresses, even though the actual physical RAM in the machine is
much smaller and in fact shared with every other running program. This trick is
called *virtual memory*, and the data structure that makes it possible is the
*page table*.

## what a page is

The CPU and the kernel agree on a fixed unit, the *page*. On most x86_64 and
AArch64 systems the page size is 4096 bytes. Memory is allocated, mapped, and
protected one page at a time. When your program accesses an address, the CPU
splits the address into "which page is this on?" and "where inside that page?",
and looks up the first half in a per-process table that the kernel maintains.
The answer to the lookup is either "this virtual page is currently backed by
physical page P" or "this virtual page has no mapping; raise a page fault."

## the program break and the heap

The classic way to grow a process's heap is the `brk` syscall: the kernel
maintains, per process, a "program break" address that marks the end of the
heap. Calling `brk(new_break)` extends or shrinks the heap. Old `malloc`
implementations used `brk` directly. Modern allocators (glibc's, jemalloc,
mimalloc) prefer `mmap` for large allocations because `mmap` lets you ask for
a fresh region anywhere, with arbitrary protection bits, without having to
walk the heap end forward.

## mmap, the swiss army knife

`mmap(addr, len, prot, flags, fd, offset)` asks the kernel for a region of
virtual memory, optionally backed by a file. The classic shapes are:

- *anonymous*, `MAP_ANONYMOUS | MAP_PRIVATE`: pure RAM, zero-filled, no file
  backing. This is the modern way to get a fresh chunk of memory.
- *file-backed read-only*, with `PROT_READ | MAP_PRIVATE`: read a file by
  treating it as memory. The kernel pages bytes in lazily as you touch them.
- *file-backed shared*, `PROT_READ | PROT_WRITE | MAP_SHARED`: writes go back
  to the file (eventually); other processes that map the same file see the
  changes. This is how IPC by shared memory works on Linux.

`munmap(addr, len)` removes the mapping. Accessing the address after `munmap`
gives you `SIGSEGV` because there is no longer any backing VMA at that
virtual address.

## protections

Each mapped region carries protection bits: `PROT_READ`, `PROT_WRITE`,
`PROT_EXEC`. The CPU enforces these on every access. A page that is `PROT_READ`
only and you try to write to it: page fault, kernel sees the protection
violation, kernel delivers `SIGSEGV` to your process. This is also how
just-in-time compilers protect their code pages: they map the page writable
while they are filling it, then `mprotect` it to read+exec only before they
ever jump into it.

## why we care for the creature

Several PEEK questions use mmap traces. The right intuition is "ask for the
mapping, check the protections, touch the address, and watch what the CPU does
when you violate them." Memorize the shape; the details fall out.

## key terms

- virtual memory
- page table
- page (4096 bytes)
- page fault
- mmap, munmap
- mprotect
- MAP_ANONYMOUS, MAP_PRIVATE, MAP_SHARED
- PROT_READ, PROT_WRITE, PROT_EXEC
- SIGSEGV

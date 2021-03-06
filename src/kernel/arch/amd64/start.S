/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang)
 *
 * arcm/amd64/start.S
 * - AMD64 Entrypoint
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

/* The kernel is linked to run at -2GB. This allows efficient addressing */
KERNEL_BASE = 0xFFFFFFFF80000000

/* === Multiboot Header === */
MULTIBOOT_PAGE_ALIGN  =  (1<<0)
MULTIBOOT_MEMORY_INFO =  (1<<1)
MULTIBOOT_REQVIDMODE  =  (1<<2)
MULTIBOOT_HEADER_MAGIC =  0x1BADB002
MULTIBOOT_HEADER_FLAGS = (MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO | MULTIBOOT_REQVIDMODE)
MULTIBOOT_CHECKSUM     = -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)
.section .multiboot, "a"
.globl mboot
mboot:
    .long MULTIBOOT_HEADER_MAGIC
    .long MULTIBOOT_HEADER_FLAGS
    .long MULTIBOOT_CHECKSUM
    .long mboot
    /* a.out kludge (not used, the kernel is elf) */
    .long 0, 0, 0, 0	/* load_addr, load_end_addr, bss_end_addr, entry_addr */
    /* Video mode */
    .long 0 	/* Mode type (0: LFB) */
    .long 0 	/* Width (no preference) */
    .long 0 	/* Height (no preference) */
    .long 32	/* Depth (32-bit preferred) */

#define DEBUG(c)	mov $0x3f8, %dx ; mov $c, %al ; outb %al, %dx

/* === Code === */
.section .inittext, "ax"
.globl start
.code32
start:
    /* The kernel starts in protected mode (32-bit mode, we want to switch to long mode) */

    /* 1. Save multiboot state */
    mov %eax, mboot_sig - KERNEL_BASE
    mov %ebx, mboot_ptr - KERNEL_BASE

    /* 2. Ensure that the CPU support long mode */
    mov $0x80000000, %eax
    cpuid
    /* - Check if CPUID supports the field we want to query */
    cmp $0x80000001, %eax
    jbe not64bitCapable
    /* - Test the IA-32e bit */
    mov $0x80000001, %eax
    cpuid
    test $0x20000000, %edx /* bit 29 = */
    jz not64bitCapable

    /* 3. Set up state for long mode */
    /* Enable:
        PGE (Page Global Enable)
      + PAE (Physical Address Extension)
      + PSE (Page Size Extensions)
    */
    mov %cr4, %eax
    or $(0x80|0x20|0x10), %eax
    mov %eax, %cr4

    /* Load PDP4 */
    mov $(init_pml4 - KERNEL_BASE), %eax
    mov %eax, %cr3

    /* Enable IA-32e mode (Also enables SYSCALL and NX) */
    mov $0xC0000080, %ecx
    rdmsr
    or $(1 << 11)|(1 << 8)|(1 << 0), %eax     /* NXE, LME, SCE */
    wrmsr

    /* Enable paging and enter long mode */
    mov %cr0, %eax
    or $0x80010000, %eax      /* PG & WP */
    mov %eax, %cr0
    lgdt GDTPtr_low - KERNEL_BASE
    ljmp $0x08, $start64


not64bitCapable:
    /* If the CPU isn't 64-bit capable, print a message to serial/b8000 then busy wait */
    mov $0x3f8, %dx
    mov $'N', %al ; outb %al, %dx
    movw $0x100|'N', 0xb8000
    mov $'o', %al ; outb %al, %dx
    movw $0x100|'o', 0xb8002
    mov $'t', %al ; outb %al, %dx
    movw $0x100|'t', 0xb8004
    mov $'6', %al ; outb %al, %dx
    movw $0x100|'6', 0xb8006
    mov $'4', %al ; outb %al, %dx
    movw $0x100|'4', 0xb8008

not64bitCapable.loop:
    hlt
    jmp not64bitCapable.loop
.code64
.globl start64
start64:
    /* Running in 64-bit mode, jump to high memory */
    lgdt GDTPtr
    mov $start64_high, %rax
    jmp *%rax

.section .text
.extern kmain
.globl start64_high
start64_high:
    /* and clear low-memory mapping */
    mov $0, %rax
    mov %rax, init_pml4 - KERNEL_BASE + 0

    /* Set up segment registers */
    mov $0x10, %ax
    mov %ax, %ss
    mov %ax, %ds
    mov %ax, %es
    xor %ax, %ax
    mov %ax, %fs
    mov %ax, %gs

    /* Set up stack pointer */
    mov $init_stack, %rsp

    /* Enable SSE */
    mov %cr0, %rax
    and $0xFFFB, %ax    // clear coprocessor emulation CR0.EM
    or $2, %ax          // set coprocessor monitoring  CR0.MP
    mov %rax, %cr0
    mov %cr4, %rax
    or $(3 << 9), %rax  // set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
    mov %rax, %cr4

    /* call the rust code */
    call kmain

    /* and if that returns (it shouldn't) loop forever */
start64.loop:
    hlt
    jmp start64.loop
    .align 16


.globl thread_entry_asm
thread_entry_asm:
    mov %rbx, %rdi
    jmp thread_entry
    .align 16


#define PUSH_REGS \
    push %rbx ;\
    push %rcx ;\
    push %rdx ;\
    push %rbp ;\
    push %rdi ;\
    push %rsi ;\
    push %r8 ;\
    push %r9 ;\
    push %r10 ;\
    push %r11 ;\
    push %r12 ;\
    push %r13 ;\
    push %r14 ;\
    push %r15 ;\
    sub $512, %rsp ;\
    fxsave (%rsp);

#define POP_REGS \
    fxrstor (%rsp); \
    add $512, %rsp; \
    pop %r15 ;\
    pop %r14 ;\
    pop %r13 ;\
    pop %r12 ;\
    pop %r11 ;\
    pop %r10 ;\
    pop %r9 ;\
    pop %r8 ;\
    pop %rsi ;\
    pop %rdi ;\
    pop %rbp ;\
    pop %rdx ;\
    pop %rcx ;\
    pop %rbx


.globl syscall_entry_asm
syscall_entry_asm:
    mov %rsp, %r11
    movq TSS+4, %rsp
    sti
    push %r11
    push %rax
    PUSH_REGS
    mov %rsp, %rdi
    call syscall_entry
    POP_REGS
    add $8, %rsp    	// skip rax
    pop %r11        	// preserved rsp
    cli             	// about to switch stacks
    mov %r11, %rsp
    movq $(1<<9), %r11  // r11=user flags: IF=1
    sysretq         	// rcx=user rip (preserved above)


#define ISR_ASM(num, rust_handler) \
    sti ;\
    pushq $0 ; \
    push %rax ;\
    PUSH_REGS ;\
    mov $num, %rdi ;\
    mov %rsp, %rsi ;\
    call rust_handler ;\
    POP_REGS ;\
    pop %rax ;\
    cli ;\
    add $16, %rsp


#undef exception
#define exception(n) \
    .globl	exception_##n ;\
    \
exception_##n: \
    ISR_ASM(0x##n, exception) ;\
    iretq ;\
    .align 16

#undef irq
#define irq(n) \
    .globl	irq_##n ;\
    \
irq_##n: \
    pushq	$-1 ;\
    ISR_ASM(0x##n, irq) ;\
    iretq ;\
    .align 16

#undef interrupt
#define interrupt(n) \
    .globl	interrupt_##n ;\
    \
interrupt_##n: \
    pushq	$-2 ;\
    ISR_ASM(0x##n, interrupt) ;\
    iretq ;\
    .align 16

#include "interrupts.h"

.globl lidt
lidt:
    lidtq (%rdi)
    ret

/* === Page-aligned data === */
.section .padata
/* Initial paging structures, four levels */
/* The +3 for sub-pages indicates "present (1) + writable (2)" */
.globl init_pml4
init_pml4:
    .quad low_pdpt - KERNEL_BASE + 3	/* low map for startup, will be cleared before rust code runs */
    .rept 512 - 3
    	.quad 0
    .endr
    .quad init_pml4 - KERNEL_BASE + 3 	/* recursive page table mapping */
    .quad init_pdpt - KERNEL_BASE + 3	/* identity mapping */
low_pdpt:
    .quad init_pd - KERNEL_BASE + 3	/* early init identity map */
    .rept 512 - 1
    	.quad 0
    .endr
init_pdpt:	/* covers the top 512GB, 1GB each entry */
    .rept 512 - 2
    	.quad 0
    .endr
    .quad init_pd - KERNEL_BASE + 3	/* at -2GB, identity map the kernel image */
    .quad 0
init_pd:
    /* 0x80 = Page size extension */
    .quad 0x000000 + 0x80 + 3
    .quad 0x200000 + 0x80 + 3
    .quad 0x400000 + 0x80 + 3
    .quad 0x600000 + 0x80 + 3
    .rept 512 - 4
    	.quad 0
    .endr
init_stack_base:
    .rept 0x1000 * 2
    	.byte 0
    .endr
init_stack:

/* === General Data === */
.section .data
.globl mboot_sig
.globl mboot_ptr
mboot_sig:	.long 0
mboot_ptr:	.long 0

/* Global Descriptor Table */
GDTPtr_low:
    .word GDT - GDTEnd
    .long GDT - KERNEL_BASE
GDTPtr:
    .word GDT - GDTEnd
    .quad GDT
.globl GDT
GDT:
/*
 * Offset   Size    Description
 * 0        2       Limit low bits (0..15)
 * 2        2       Base low bits (0..15)
 * 4        1       Base middle bits (16..23)
 * 5        1       Access
 * 6        1       Limit high bits (16..19), flags
 * 7        1       Base high bits (24..31)
 * 8        4       Base extra-high bits (32..63)
 *
 * Access when S=0:
 * 	0x09: (x64) Available 64-bit TSS
 * 	0x80: Present
 * -
 * 	0x89: Available 64-bit TSS, present
 *
 * Access when S=1:
 *  0x01: Accessed
 *  0x02: Readable code/writable data
 *  0x04: Direction/conforming
 *  0x08: Executable
 *  0x10: (S bit - always 1)
 *  0x60: Ring 3
 *  0x80: Present
 *  -
 *  0x9A = Readable code, executable, present
 *  0x92 = Writable data, present
 *  0xF2 = Ring 3, writable data, present
 *  0xFA = Ring 3, readable code, executable, present
 *  0x82 = Writable data, present, descriptor is 16 bytes
 *
 * Flags:
 *  0x2x = (x64) 64-bit
 *  0x4x = 32-bit
 *  0x8x = Page granularity
 *  -
 */
    .long 0, 0
    .long 0x00000000, 0x00209A00    // 0x08: 64-bit Code
    .long 0x00000000, 0x00009200    // 0x10: 64-bit Data
    .long 0x00000000, 0x00000000	// 0x18: (not used)
    .long 0x00000000, 0x0040FA00    // 0x20: 32-bit User Code
    .long 0x00000000, 0x0040F200    // 0x28: User Data
    .long 0x00000000, 0x0020FA00    // 0x30: 64-bit User Code

.globl GDT_TSS
GDT_TSS:
    .long 0x00000000, 0x00000000	// 0x38: TSS (adjusted in Rust code before ltr instruction)
    .long 0x00000000, 0x00000000
GDTEnd:

.globl TSS
TSS:
    .rept 104
    	.byte 0
    .endr
.globl TSSEnd
TSSEnd:

.globl IDT
IDT:
#undef exception
#define exception(n) .long 0, 0, 0, 0

#undef irq
#define irq(n) .long 0, 0, 0, 0

#undef interrupt
#define interrupt(n) .long 0, 0, 0, 0

#include "interrupts.h"
.globl IDTEnd
IDTEnd:

/* Interrupt handlers */
.globl interrupt_handlers
interrupt_handlers:
#undef exception
#define exception(n) .quad exception_##n

#undef irq
#define irq(n) .quad irq_##n

#undef interrupt
#define interrupt(n) .quad interrupt_##n

#include "interrupts.h"
.globl interrupt_handlers_end
interrupt_handlers_end:

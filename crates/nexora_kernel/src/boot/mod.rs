use core::arch::global_asm;

global_asm!(
    r#"
.intel_syntax noprefix

/* 1. Multiboot2 Header Section */
.section .multiboot_header, "a"
.balign 8
header_start:
    .long 0xe85250d6                                /* Magic number (Multiboot2) */
    .long 0                                         /* Architecture 0 (i386 protected mode) */
    .long header_end - header_start                 /* Header length */
    .long -(0xe85250d6 + 0 + (header_end - header_start)) /* Checksum */

    /* End Tag */
    .short 0
    .short 0
    .long 8
header_end:

/* 2. Kernel Entry Point (32-bit Protected Mode) */
.section .text
.code32
.global _start
_start:
    cli                                             /* Disable interrupts */

    /* Validate Multiboot2 Magic Number */
    cmp eax, 0x36d76289
    jne .Lno_multiboot

    /* Save Multiboot2 Information Structure Address in EDI */
    mov edi, ebx

    /* Setup Boot Stack */
    mov esp, offset boot_stack_top

    /* Setup Page Tables (Identity Map first 2MiB) */
    mov eax, offset pdpt
    or eax, 0x3                                     /* Present + Writable */
    mov dword ptr [pml4], eax

    mov eax, offset pd
    or eax, 0x3                                     /* Present + Writable */
    mov dword ptr [pdpt], eax

    mov eax, 0x0
    or eax, 0x83                                    /* Present + Writable + HugePage (2MiB) */
    mov dword ptr [pd], eax

    /* Load PML4 into CR3 */
    mov eax, offset pml4
    mov cr3, eax

    /* Enable PAE in CR4 */
    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    /* Enable Long Mode in EFER MSR (0xC0000080) */
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    /* Enable Paging in CR0 */
    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax

    /* Load 64-bit Global Descriptor Table (GDT) */
    lgdt [gdt64_pointer]

    /* Far reload into 64-bit Code Segment */
    push 0x08
    mov eax, offset _start64
    push eax
    retf

.Lno_multiboot:
    hlt
    jmp .Lno_multiboot

/* 3. 64-bit Long Mode Entry */
.code64
_start64:
    /* Reload Segment Registers */
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /* RDI already holds Multiboot2 info address from EDI (System V AMD64 ABI) */
    call kmain

.Lhang64:
    cli
    hlt
    jmp .Lhang64

/* 4. Boot Page Tables & Stack Allocation */
.section .bss
.balign 4096
pml4:
    .skip 4096
pdpt:
    .skip 4096
pd:
    .skip 4096
boot_stack:
    .skip 16384
boot_stack_top:

/* 5. GDT Descriptor */
.section .rodata
.balign 8
gdt64:
    .quad 0                                         /* Null Descriptor */
    .quad 0x00209a0000000000                        /* 64-bit Code Segment (Kernel) */
    .quad 0x0000920000000000                        /* 64-bit Data Segment (Kernel) */
gdt64_pointer:
    .word . - gdt64 - 1
    .quad gdt64
"#
);

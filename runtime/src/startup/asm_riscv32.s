/* rt_header is defined by the general linker script (libtock_layout.ld). It has
 * the following layout:
 *
 *     Field                       | Offset
 *     ------------------------------------
 *     Address of the start symbol |      0
 *     Initial process break       |      4
 *     Top of the stack            |      8
 *     Size of .data               |     12
 *     Start of .data in flash     |     16
 *     Start of .data in ram       |     20
 *     Size of .bss                |     24
 *     Start of .bss in ram        |     28
 */

/* start is the entry point -- the first code executed by the kernel. The kernel
 * passes arguments through 4 registers:
 *
 *     a0  Pointer to beginning of the process binary's code. The linker script
 *         locates rt_header at this address.
 *
 *     a1  Address of the beginning of the process's usable memory region.
 *     a2  Size of the process' allocated memory region (including grant region)
 *     a3  Process break provided by the kernel.
 *
 * We currently only use the value in a0. It is copied into a5 early on because
 * a0-a4 are needed to invoke system calls.
 */
.section .start, "ax"
.globl start
start:
    /* Compute the stack top.
     *
     * struct hdr* myhdr = (struct hdr*) app_start;
     * uint32_t stacktop = (((uint32_t) mem_start + myhdr->stack_size + 7) & 0xfffffff8);
	 */
    lw t0, 36(a0)
    addi t0, t0, 7
    add t0, t0, a1
    li t1, 7
    not t1, t1
    and t0, t0, t1
    
    /* Compute the app data size and where initial app brk should go.
     * This includes the GOT, data, and BSS sections. However, we can't be sure
     * the linker puts them back-to-back, but we do assume that BSS is last
     * (i.e. myhdr->got_start < myhdr->bss_start && myhdr->data_start <
     * myhdr->bss_start). With all of that true, then the size is equivalent
     * to the end of the BSS section.
	 * 
	 * uint32_t app_brk = mem_start + myhdr->bss_start + myhdr->bss_size;
     */
    lw t1, 24(a0)
    lw t2, 28(a0)
    add t1, t1, t2
    add t1, t1, a1
    
    /* Move arguments we need to keep over to callee-saved locations. */
    mv   s0, a0
    mv   s1, t0
    mv   s2, a1
    
    /* Now we may want to move the stack pointer. If the kernel set the
     * `app_heap_break` larger than we need (and we are going to call `brk()`
     * to reduce it) then our stack pointer will fit and we can move it now.
     * Otherwise after the first syscall (the memop to set the brk), the return
     * will use a stack that is outside of the process accessible memory.
	 * Compare `app_heap_break` with new brk.
	 * If our current `app_heap_break` is larger
	 * then we need to move the stack pointer
	 * before we call the `brk` syscall.
     */
    bgt t1, a3, skip_set_sp
	/* Update the stack pointer */
    mv  sp, t0

	/* Back to regularly scheduled programming. */
    skip_set_sp:

    /* Call `brk` to set to requested memory
     * memop(0, stacktop + appdata_size);
	 */
    li  a4, 5
    li  a0, 0
    mv  a1, t1
    ecall
    
    /* Setup initial stack pointer for normal execution
	 */
    mv   sp, s1

    /* Debug support, tell the kernel the stack location
     *
     * memop(10, stacktop);
	 */
    li  a4, 5
    li  a0, 10
    mv  a1, s1
    ecall
    
    /* Debug support, tell the kernel the heap location
     *
     * memop(11, app_brk);
	 */
    li  a4, 5
    li  a0, 11
    mv  a1, t1
    ecall

    /* Set gp, the ePIC base register. The ePIC code uses this as a reference
     * point to enable the RAM section of the app to be at any address.
	 */
    mv   gp, s2

    /* Call into the rest of startup. This should never return.
	 */
    mv   a0, s0
    mv   s0, sp
    mv   a1, s2
    jal  rust_start

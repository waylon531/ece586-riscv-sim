.global pjpeg_need_bytes_callback
pjpeg_need_bytes_callback:
    la t0, image_end
    # Load cursor
    la t4, image_cursor
    lw t1, (t4)
    sub t2, t0, t1
    # Check if there's less bytes to read than requested
    bge t2, a1, skip
    add a1, t2, zero
skip:
    sb a1, (a2) # Write the bytes_actually_read field
loop:
    lb t3, (t1) # load a byte from the image
    sb t3, (a0) # store that byte back in the buffer
    addi a0, a0, 1  # increment the buffer pointer and cursor
    addi t1, t1, 1
    addi a1, a1, -1 # decrement the number of bytes to read
    bnez a1, loop

    # Save the value of the cursor
    sw t1, (t4)
    
    li a0, 0

    ret

    # a0 char * pbuf (buffer to fill)
    # a1 buf_size (max amount to read)
    # a2 char * bytes_actually_read (how many bytes we read)
    # a3 * callback data (ignore)
.section .data
image_cursor:
    .word image
.section .rodata
image:
.space 2998
image_end:
    .word 2998

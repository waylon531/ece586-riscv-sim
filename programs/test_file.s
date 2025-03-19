.global main
.data
filename:
  .space 256
pre_prompt:
  .string "Please enter filename: "
prompt:
  .string "Please write: "
msg:
  .string "You wrote: "
errmsg:
  .string "Invalid file.\n"
user:
  .space 256
output:
  .space 256
.text
main:
# Ask user for filename
li a7, 64
li a0, 1
la a1, pre_prompt
li a2, 23
ecall
# Read filename
li a7, 63
li a0, 0
la a1, filename
li a2, 256
ecall
# Print the prompt
li a7, 64
li a0, 1
la a1, prompt
li a2, 14
ecall
# Read input from user
li a7, 63
li a0, 0
la a1, user
li a2, 256
ecall
# Save length of user input
addi a4, a0, 0
# Open file, creating if it does not exist
li a7, 56
la a0, filename
li a1, 0x40
ecall
bltz a0, error
# save fd
addi a3, a0, 0
# Write the user's input to the file
li a7, 64
la a1, user
addi a2, a4, 0
ecall
# Close the file for writing
li a7, 57
addi a0, a3, 0
ecall
# Print message
li a7, 64
li a0, 1
la a1, msg
li a2, 11
ecall
# Reopen the file
li a7, 56
la a0, filename
li a1, 0
ecall
# Now, read the file to the output buffer
li a7, 63
add a0, a3, 0
la a1, output
# Save again
ecall
add a3, a0, 0
# Finally, output the buffer to stdout
li a0, 1
la a1, output
li a7, 64
ecall
# And close the file again
li a7, 57
add a0, a3, 0
ecall
jr ra
error:
  # Print message
  li a7, 64
  li a0, 2
  la a1, errmsg
  li a2, 14
  ecall
  # exit
  li a7, 94
  li a0, 1
  ecall

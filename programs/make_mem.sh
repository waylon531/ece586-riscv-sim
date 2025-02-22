#!/bin/bash


file="$1"
stripped="${1%.*}"
if ! [[ -e $file ]] ; then
    echo "File $file not found"
    exit 1
fi
# I dunno if these are all possible sections
riscv32-unknown-elf-objdump -j .text -j .data -j .bss -j .got -D "$file" | grep -o '^[[:blank:]]*[[:xdigit:]]*:[[:blank:]][[:xdigit:]]*' > "${stripped}.mem"

echo "Done!"

#!/bin/bash


file="$1"
stripped="${1%.*}"
if ! [[ -e $file ]] ; then
    echo "File $file not found"
    exit 1
fi
# I dunno if these are all possible sections
riscv64-unknown-elf-objdump -D "$file" | grep -oE '^[[:blank:]]*[[:xdigit:]]*:[[:blank:]][[:xdigit:]]*( [[:xdigit:]]+)*' > "${stripped}.mem"

echo "Done!"

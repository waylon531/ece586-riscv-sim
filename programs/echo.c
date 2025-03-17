char * SERIAL_ADDR=(char*)0xF00003F8;
char * SERIAL_STATUS_ADDR=(char*)(0xF00003F8+5);
char inb() {
    // Block until data is ready
    while ((*SERIAL_STATUS_ADDR & 1) == 0) {}
    
    // Return the data
    return *SERIAL_ADDR;
}
void outb(char data) {
    *SERIAL_ADDR = data;
}
int main() {
    char b;
    outb(' ');
    outb('h');
    outb('e');
    outb('l');
    outb('l');
    outb('o');
    outb('\r');
    outb('\n');
    while(1) {
        b = inb();
        outb(b);
    }
}

char * SERIAL_ADDR=(char*)0xF00003F8;
void outb(char data) {
    *SERIAL_ADDR = data;
}
int main() {
    while(1) {
    outb('h');
    outb('e');
    outb('l');
    outb('l');
    outb('o');
    outb('\r');
    outb('\n');
    }
}

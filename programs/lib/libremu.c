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

void outs(char * s) {
    if (s == 0) {
        return;
    }
    while(*s != '\0') {
        outb(*s);
        s++;
    }
    return;
}

int min(int a, int b) {

    if (a < b)
        return a;
    else
        return b;
}

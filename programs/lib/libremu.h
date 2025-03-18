char inb();
void outb(char data);

void outs(char * s);

int min(int a, int b);

int * const FRAMEBUFFER = (int *)0xFF000000;
char * const SERIAL_PORT = (char *)0xF00003F8;

const int HEIGHT = 144;
const int WIDTH = 160;


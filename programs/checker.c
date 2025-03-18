int* FB = (int*) 0xFF000000;

int black = 0x00FFFFFF;
int white = 0x00000000;

int main() {
    char invert;
    // 20 px checker pattern
    for(int i = 0; i < 400; i++) {
        for(int j = 0; j < 640; j++) {
            invert = (i/20) % 2;
            if (((j/20) % 2 ) ^ invert) {
                FB[i*640+j] = black;
            } else {
                FB[i*640+j] = white;
            }
        }

    }
    while(1) {}
}

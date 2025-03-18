int* FB = (int*) 0xFF000000;

int black = 0x00FFFFFF;
int white = 0x00000000;

const int WIDTH = 160;
const int HEIGHT = 144;
int main() {
    char invert;
    // 20 px checker pattern
    for(int i = 0; i < HEIGHT; i++) {
        for(int j = 0; j < WIDTH; j++) {
            invert = (i/20) % 2;
            if (((j/20) % 2 ) ^ invert) {
                FB[i*WIDTH+j] = black;
            } else {
                FB[i*WIDTH+j] = white;
            }
        }

    }
    while(1) {}
}

char multiplicands[] = {3,4,5};
short nums[] = {512,600,452};
int result[3];

int main() {
    int i=0;
    int j=0;
    int acc=0;
    int multiplied=0;
    for (i=0; i<3; i++) {
        j = multiplicands[i];
        multiplied=0;
        for (;j>0; j--) {
            multiplied += nums[i];
        }
        result[i] = multiplied;
        acc += multiplied;

    }
    return acc;
    
}

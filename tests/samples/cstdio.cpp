#include <cstdio>

int main() {
    FILE* file = stderr;
    return fprintf(file, "value = %d\n", 42) < 0;
}

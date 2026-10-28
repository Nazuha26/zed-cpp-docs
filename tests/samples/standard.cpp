#include <stdio.h>
#include <sstream>
#include <string>
#include <vector>

namespace user {
int printf(int value) {
    return value;
}
}

int main() {
    std::istringstream input("first\nsecond");
    std::string line;
    std::getline(input, line);
    std::vector<int> values;
    values.push_back(42);
    printf("%s %d\n", line.c_str(), values[0]);
    return user::printf(0);
}

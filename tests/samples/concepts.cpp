#include <concepts>

int main() {
    return std::same_as<int, int> ? 0 : 1;
}

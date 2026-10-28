#include <errno.h>
#include <stdio.h>
#include <exception>
#include <ios>
#include <iostream>
#include <limits>
#include <map>
#include <sstream>
#include <string>
#include <system_error>
#include <type_traits>
#include <utility>
#include <vector>

namespace user {
struct string {
    int value;
};
}

typedef std::string LocalString;

int main() {
    std::string text("value");
    std::istringstream input(text);
    std::vector<int> values;
    std::map<int, std::string> labels;
    std::pair<int, int> pair(1, 2);
    std::basic_string<char> other("other");
    std::size_t count = text.size();
    std::string::size_type length = text.length();
    std::string::value_type character = text.operator[](0);
    std::string::size_type missing = std::string::npos;
    std::ios_base::iostate state = std::ios_base::goodbit;
    std::exception exception;
    int endOfFile = EOF;
    int error = errno;
    int first = pair.first;
    std::ostream* output = &std::cout;
    std::float_round_style rounding = std::round_to_nearest;
    std::errc condition = std::errc::invalid_argument;
    int precision = std::numeric_limits<double>::max_digits10;
    bool constant = std::is_const_v<const int>;
    std::remove_const_t<const int> mutableValue = 0;
    user::string local;
    LocalString ownAlias;
    local.value = 0;
    return local.value;
}

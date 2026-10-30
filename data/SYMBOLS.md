# Compact symbol catalog

Every entry below has an original description and a runnable example in [`compact-docs.json`](compact-docs.json). Reference links identify the imported cppreference page; standard references are recorded in [`compact-docs-sources.json`](compact-docs-sources.json).

Coverage is extended by practical API groups; there is no fixed entry limit.

## C++ (215)

| Symbol | Description |
| --- | --- |
| [`std::getline`](https://en.cppreference.com/w/cpp/string/basic_string/getline) | Read characters from a stream into a string up to a delimiter. |
| [`std::vector`](https://en.cppreference.com/w/cpp/container/vector) | A resizable array with contiguous storage for elements other than bool. |
| [`std::vector::push_back`](https://en.cppreference.com/w/cpp/container/vector/push_back) | Append one element to the end of a vector. |
| [`std::vector::emplace_back`](https://en.cppreference.com/w/cpp/container/vector/emplace_back) | Construct an element directly at the end from the supplied arguments. |
| [`std::vector::pop_back`](https://en.cppreference.com/w/cpp/container/vector/pop_back) | Destroy and remove the last element. |
| [`std::vector::size`](https://en.cppreference.com/w/cpp/container/vector/size) | Return the number of stored elements, not the amount of reserved storage. |
| [`std::vector::empty`](https://en.cppreference.com/w/cpp/container/vector/empty) | Test whether the vector contains no elements. |
| [`std::vector::reserve`](https://en.cppreference.com/w/cpp/container/vector/reserve) | Request storage for at least the given number of elements without changing size. |
| [`std::vector::resize`](https://en.cppreference.com/w/cpp/container/vector/resize) | Change the number of elements. |
| [`std::vector::capacity`](https://en.cppreference.com/w/cpp/container/vector/capacity) | Return how many elements fit in the current allocation before growth requires reallocation. |
| [`std::vector::clear`](https://en.cppreference.com/w/cpp/container/vector/clear) | Destroy all elements and set size to zero while retaining the allocation. |
| [`std::vector::erase`](https://en.cppreference.com/w/cpp/container/vector/erase) | Remove an element or a half-open range and shift later elements toward the beginning. |
| [`std::vector::insert`](https://en.cppreference.com/w/cpp/container/vector/insert) | Insert before a position and shift following elements. |
| [`std::vector::at`](https://en.cppreference.com/w/cpp/container/vector/at) | Access an element with bounds checking. |
| [`std::vector::operator[]`](https://en.cppreference.com/w/cpp/container/vector/operator_at) | Access an element without bounds checking. |
| [`std::vector::front`](https://en.cppreference.com/w/cpp/container/vector/front) | Access the first element without removing it. |
| [`std::vector::back`](https://en.cppreference.com/w/cpp/container/vector/back) | Access the last element without removing it. |
| [`std::vector::data`](https://en.cppreference.com/w/cpp/container/vector/data) | Obtain a pointer to contiguous element storage. |
| [`std::string`](https://en.cppreference.com/w/cpp/string/basic_string) | An owning, resizable contiguous character sequence. |
| [`std::basic_string`](https://en.cppreference.com/w/cpp/string/basic_string) | An owning, resizable contiguous character sequence. |
| [`std::basic_string::size`](https://en.cppreference.com/w/cpp/string/basic_string/size) | Return the number of stored character code units, excluding the trailing null terminator. |
| [`std::basic_string::empty`](https://en.cppreference.com/w/cpp/string/basic_string/empty) | Test whether the string contains zero stored characters. |
| [`std::basic_string::c_str`](https://en.cppreference.com/w/cpp/string/basic_string/c_str) | Return a null-terminated, read-only view of the character storage. |
| [`std::basic_string::data`](https://en.cppreference.com/w/cpp/string/basic_string/data) | Access contiguous character storage followed by a null terminator. |
| [`std::basic_string::substr`](https://en.cppreference.com/w/cpp/string/basic_string/substr) | Create an owning copy of a substring. |
| [`std::basic_string::find`](https://en.cppreference.com/w/cpp/string/basic_string/find) | Search for the first occurrence at or after a position. |
| [`std::basic_string::append`](https://en.cppreference.com/w/cpp/string/basic_string/append) | Add characters to the end of a string. |
| [`std::basic_string::erase`](https://en.cppreference.com/w/cpp/string/basic_string/erase) | Remove characters starting at a position. |
| [`std::basic_string::npos`](https://en.cppreference.com/w/cpp/string/basic_string/npos) | The largest value representable by size_type. |
| [`std::basic_string::operator[]`](https://en.cppreference.com/w/cpp/string/basic_string/operator_at) | Access a character without bounds checking. |
| [`std::map`](https://en.cppreference.com/w/cpp/container/map) | An ordered collection of unique keys and associated values. |
| [`std::map::operator[]`](https://en.cppreference.com/w/cpp/container/map/operator_at) | Return the mapped value for a key, inserting a new element if the key is absent. |
| [`std::map::at`](https://en.cppreference.com/w/cpp/container/map/at) | Access the mapped value of an existing key without inserting. |
| [`std::map::find`](https://en.cppreference.com/w/cpp/container/map/find) | Look up an element by key without inserting. |
| [`std::map::count`](https://en.cppreference.com/w/cpp/container/map/count) | Count elements with a key equivalent to the supplied key. |
| [`std::map::insert`](https://en.cppreference.com/w/cpp/container/map/insert) | Insert a key/value pair only if no equivalent key exists. |
| [`std::map::erase`](https://en.cppreference.com/w/cpp/container/map/erase) | Remove the element matching a key. |
| [`std::unordered_map`](https://en.cppreference.com/w/cpp/container/unordered_map) | A hash table of unique keys and associated values. |
| [`std::set`](https://en.cppreference.com/w/cpp/container/set) | An ordered collection of unique keys. |
| [`std::unordered_set`](https://en.cppreference.com/w/cpp/container/unordered_set) | A hash table of unique keys with unspecified iteration order. |
| [`std::array`](https://en.cppreference.com/w/cpp/container/array) | An aggregate containing a fixed number of contiguous elements. |
| [`std::deque`](https://en.cppreference.com/w/cpp/container/deque) | A resizable double-ended sequence with constant-time indexed access and constant-time insertion and erasure of single elements at either end. |
| [`std::list`](https://en.cppreference.com/w/cpp/container/list) | A bidirectional linked sequence. |
| [`std::queue`](https://en.cppreference.com/w/cpp/container/queue) | A first-in, first-out container adaptor. |
| [`std::stack`](https://en.cppreference.com/w/cpp/container/stack) | A last-in, first-out container adaptor. |
| [`std::pair`](https://en.cppreference.com/w/cpp/utility/pair) | A structure storing two values, which may have different types. |
| [`std::pair::first`](https://en.cppreference.com/w/cpp/utility/pair) | The first public data member of a pair. |
| [`std::pair::second`](https://en.cppreference.com/w/cpp/utility/pair) | The second public data member of a pair. |
| [`std::make_pair`](https://en.cppreference.com/w/cpp/utility/pair/make_pair) | Construct a pair with element types deduced from the supplied values. |
| [`std::sort`](https://en.cppreference.com/w/cpp/algorithm/sort) | Reorder a half-open range into ascending order by default. |
| [`std::find`](https://en.cppreference.com/w/cpp/algorithm/find) | Search a half-open range from the beginning for the first element equal to a value. |
| [`std::count`](https://en.cppreference.com/w/cpp/algorithm/count) | Count elements equal to a value in a half-open range. |
| [`std::binary_search`](https://en.cppreference.com/w/cpp/algorithm/binary_search) | Test whether an equivalent value exists in a range partitioned appropriately for the comparison. |
| [`std::lower_bound`](https://en.cppreference.com/w/cpp/algorithm/lower_bound) | Find the first element that is not less than a value. |
| [`std::min`](https://en.cppreference.com/w/cpp/algorithm/min) | Select the smaller of two values using operator< by default. |
| [`std::max`](https://en.cppreference.com/w/cpp/algorithm/max) | Select the larger of two values using operator< by default. |
| [`std::reverse`](https://en.cppreference.com/w/cpp/algorithm/reverse) | Reverse element order in a writable half-open range. |
| [`std::fill`](https://en.cppreference.com/w/cpp/algorithm/fill) | Assign the same value to every element in a writable half-open range. |
| [`std::accumulate`](https://en.cppreference.com/w/cpp/algorithm/accumulate) | Fold a range from left to right, starting with an initial value and adding each element by default. |
| [`std::istringstream`](https://en.cppreference.com/w/cpp/io/basic_istringstream) | An input stream backed by an owned string buffer. |
| [`std::basic_string::at`](https://en.cppreference.com/w/cpp/string/basic_string/at) | Access a stored character with bounds checking. |
| [`std::basic_string::clear`](https://en.cppreference.com/w/cpp/string/basic_string/clear) | Remove every stored character and make the string empty. |
| [`std::basic_string::insert`](https://en.cppreference.com/w/cpp/string/basic_string/insert) | Insert characters before an index and move the following characters. |
| [`std::basic_string::replace`](https://en.cppreference.com/w/cpp/string/basic_string/replace) | Replace a substring with new characters. |
| [`std::basic_string::compare`](https://en.cppreference.com/w/cpp/string/basic_string/compare) | Compare character sequences lexicographically using the character traits. |
| [`std::basic_string::rfind`](https://en.cppreference.com/w/cpp/string/basic_string/rfind) | Search backward for the last occurrence starting at or before a position. |
| [`std::basic_string::find_first_of`](https://en.cppreference.com/w/cpp/string/basic_string/find_first_of) | Find the first stored character belonging to a set, starting at a given position. |
| [`std::basic_string::find_last_of`](https://en.cppreference.com/w/cpp/string/basic_string/find_last_of) | Find the last stored character belonging to a set, searching backward from a position. |
| [`std::basic_string::find_first_not_of`](https://en.cppreference.com/w/cpp/string/basic_string/find_first_not_of) | Find the first character outside a supplied set, starting at a position. |
| [`std::basic_string::find_last_not_of`](https://en.cppreference.com/w/cpp/string/basic_string/find_last_not_of) | Find the last character outside a supplied set, searching backward. |
| [`std::basic_string::resize`](https://en.cppreference.com/w/cpp/string/basic_string/resize) | Change the stored character count. |
| [`std::basic_string::reserve`](https://en.cppreference.com/w/cpp/string/basic_string/reserve) | Request character storage without changing size(). |
| [`std::basic_string::capacity`](https://en.cppreference.com/w/cpp/string/basic_string/capacity) | Return how many characters fit in the current allocation without requiring a new allocation. |
| [`std::basic_string::push_back`](https://en.cppreference.com/w/cpp/string/basic_string/push_back) | Append one character to the end of a string. |
| [`std::basic_string::pop_back`](https://en.cppreference.com/w/cpp/string/basic_string/pop_back) | Remove the last stored character. |
| [`std::basic_string::front`](https://en.cppreference.com/w/cpp/string/basic_string/front) | Access the first stored character. |
| [`std::basic_string::back`](https://en.cppreference.com/w/cpp/string/basic_string/back) | Access the last stored character, excluding the trailing terminator. |
| [`std::stoi`](https://en.cppreference.com/w/cpp/string/basic_string/stol) | Parse a numeric prefix from a string after optional whitespace and sign. |
| [`std::stol`](https://en.cppreference.com/w/cpp/string/basic_string/stol) | Parse a numeric prefix from a string after optional whitespace and sign. |
| [`std::stoll`](https://en.cppreference.com/w/cpp/string/basic_string/stol) | Parse a numeric prefix from a string after optional whitespace and sign. |
| [`std::stoul`](https://en.cppreference.com/w/cpp/string/basic_string/stoul) | Parse a numeric prefix from a string after optional whitespace and sign. |
| [`std::stoull`](https://en.cppreference.com/w/cpp/string/basic_string/stoul) | Parse a numeric prefix from a string after optional whitespace and sign. |
| [`std::stof`](https://en.cppreference.com/w/cpp/string/basic_string/stof) | Parse a numeric prefix from a string after optional whitespace and sign. |
| [`std::stod`](https://en.cppreference.com/w/cpp/string/basic_string/stof) | Parse a numeric prefix from a string after optional whitespace and sign. |
| [`std::stold`](https://en.cppreference.com/w/cpp/string/basic_string/stof) | Parse a numeric prefix from a string after optional whitespace and sign. |
| [`std::to_string`](https://en.cppreference.com/w/cpp/string/basic_string/to_string) | Create a string representation of a numeric value. |
| [`std::string_view`](https://en.cppreference.com/w/cpp/string/basic_string_view) | A non-owning, read-only view of a contiguous character sequence. |
| [`std::find_if`](https://en.cppreference.com/w/cpp/algorithm/find) | Find the first element for which a predicate is true, without changing the range. |
| [`std::count_if`](https://en.cppreference.com/w/cpp/algorithm/count) | Count elements satisfying a predicate. |
| [`std::all_of`](https://en.cppreference.com/w/cpp/algorithm/all_any_none_of) | Test whether every element satisfies a predicate. |
| [`std::any_of`](https://en.cppreference.com/w/cpp/algorithm/all_any_none_of) | Test whether at least one element satisfies a predicate. |
| [`std::none_of`](https://en.cppreference.com/w/cpp/algorithm/all_any_none_of) | Test whether no element satisfies a predicate. |
| [`std::copy`](https://en.cppreference.com/w/cpp/algorithm/copy) | Copy a range forward into already available destination elements. |
| [`std::copy_if`](https://en.cppreference.com/w/cpp/algorithm/copy) | Copy elements satisfying a predicate into available output storage, preserving their relative order. |
| [`std::copy_n`](https://en.cppreference.com/w/cpp/algorithm/copy_n) | Copy count elements into existing output storage. |
| [`std::transform`](https://en.cppreference.com/w/cpp/algorithm/transform) | Apply an operation and assign its result into each destination element. |
| [`std::remove`](https://en.cppreference.com/w/cpp/algorithm/remove) | Move elements not equal to value toward the beginning, preserving their order. |
| [`std::remove_if`](https://en.cppreference.com/w/cpp/algorithm/remove) | Move elements failing a predicate toward the beginning, preserving their relative order. |
| [`std::unique`](https://en.cppreference.com/w/cpp/algorithm/unique) | Remove consecutive equivalent values from the logical range, preserving the retained order. |
| [`std::replace`](https://en.cppreference.com/w/cpp/algorithm/replace) | Assign newValue to every element equal to oldValue. |
| [`std::replace_if`](https://en.cppreference.com/w/cpp/algorithm/replace) | Assign a replacement value to elements satisfying a predicate. |
| [`std::fill_n`](https://en.cppreference.com/w/cpp/algorithm/fill_n) | Assign the same value to count existing output elements. |
| [`std::stable_sort`](https://en.cppreference.com/w/cpp/algorithm/stable_sort) | Sort a writable random-access range while preserving the relative order of equivalent elements. |
| [`std::partial_sort`](https://en.cppreference.com/w/cpp/algorithm/partial_sort) | Put the smallest middle-first elements into a sorted prefix. |
| [`std::nth_element`](https://en.cppreference.com/w/cpp/algorithm/nth_element) | Place the element that would occupy nth in a sorted range at that position and partition around it. |
| [`std::upper_bound`](https://en.cppreference.com/w/cpp/algorithm/upper_bound) | Find the first element greater than value according to the comparison. |
| [`std::equal_range`](https://en.cppreference.com/w/cpp/algorithm/equal_range) | Find the half-open subrange of elements equivalent to value. |
| [`std::clamp`](https://en.cppreference.com/w/cpp/algorithm/clamp) | Select value when it is within [low, high], otherwise select the nearest bound. |
| [`std::min_element`](https://en.cppreference.com/w/cpp/algorithm/min_element) | Find the first smallest element according to operator< or a supplied strict weak ordering. |
| [`std::max_element`](https://en.cppreference.com/w/cpp/algorithm/max_element) | Find the first largest element according to operator< or a supplied strict weak ordering. |
| [`std::rotate`](https://en.cppreference.com/w/cpp/algorithm/rotate) | Move the range prefix behind the suffix while preserving order within both parts. |
| [`std::is_sorted`](https://en.cppreference.com/w/cpp/algorithm/is_sorted) | Test whether a range is in nondecreasing order according to operator< or a comparator. |
| [`std::deque::push_front`](https://en.cppreference.com/w/cpp/container/deque/push_front) | Add one element at the front of the sequence. |
| [`std::deque::push_back`](https://en.cppreference.com/w/cpp/container/deque/push_back) | Add one element at the back of the sequence. |
| [`std::deque::pop_front`](https://en.cppreference.com/w/cpp/container/deque/pop_front) | Destroy and remove the front element. |
| [`std::deque::pop_back`](https://en.cppreference.com/w/cpp/container/deque/pop_back) | Destroy and remove the back element. |
| [`std::deque::front`](https://en.cppreference.com/w/cpp/container/deque/front) | Access the first element without removing it. |
| [`std::deque::back`](https://en.cppreference.com/w/cpp/container/deque/back) | Access the last element without removing it. |
| [`std::deque::size`](https://en.cppreference.com/w/cpp/container/deque/size) | Return the number of stored elements. |
| [`std::deque::empty`](https://en.cppreference.com/w/cpp/container/deque/empty) | Test whether the sequence contains zero elements. |
| [`std::list::push_front`](https://en.cppreference.com/w/cpp/container/list/push_front) | Add one element at the front of the sequence. |
| [`std::list::push_back`](https://en.cppreference.com/w/cpp/container/list/push_back) | Add one element at the back of the sequence. |
| [`std::list::pop_front`](https://en.cppreference.com/w/cpp/container/list/pop_front) | Destroy and remove the front element. |
| [`std::list::pop_back`](https://en.cppreference.com/w/cpp/container/list/pop_back) | Destroy and remove the back element. |
| [`std::list::front`](https://en.cppreference.com/w/cpp/container/list/front) | Access the first element without removing it. |
| [`std::list::back`](https://en.cppreference.com/w/cpp/container/list/back) | Access the last element without removing it. |
| [`std::list::size`](https://en.cppreference.com/w/cpp/container/list/size) | Return the number of stored elements. |
| [`std::list::empty`](https://en.cppreference.com/w/cpp/container/list/empty) | Test whether the sequence contains zero elements. |
| [`std::list::sort`](https://en.cppreference.com/w/cpp/container/list/sort) | Sort a list in place while preserving the order of equivalent values. |
| [`std::list::remove`](https://en.cppreference.com/w/cpp/container/list/remove) | Erase every list element equal to a value. |
| [`std::list::unique`](https://en.cppreference.com/w/cpp/container/list/unique) | Erase adjacent equivalent elements while retaining the first element of each group. |
| [`std::queue::push`](https://en.cppreference.com/w/cpp/container/queue/push) | Add one element to the underlying container. |
| [`std::queue::pop`](https://en.cppreference.com/w/cpp/container/queue/pop) | Remove the next element without returning it. |
| [`std::queue::front`](https://en.cppreference.com/w/cpp/container/queue/front) | Access the oldest stored element without removing it. |
| [`std::queue::back`](https://en.cppreference.com/w/cpp/container/queue/back) | Access the newest stored element without removing it. |
| [`std::queue::size`](https://en.cppreference.com/w/cpp/container/queue/size) | Return the number of stored elements. |
| [`std::queue::empty`](https://en.cppreference.com/w/cpp/container/queue/empty) | Test whether no elements are stored. |
| [`std::stack::push`](https://en.cppreference.com/w/cpp/container/stack/push) | Add one element to the underlying container. |
| [`std::stack::pop`](https://en.cppreference.com/w/cpp/container/stack/pop) | Remove the next element without returning it. |
| [`std::stack::top`](https://en.cppreference.com/w/cpp/container/stack/top) | Access the newest stored element without removing it. |
| [`std::stack::size`](https://en.cppreference.com/w/cpp/container/stack/size) | Return the number of stored elements. |
| [`std::stack::empty`](https://en.cppreference.com/w/cpp/container/stack/empty) | Test whether no elements are stored. |
| [`std::priority_queue`](https://en.cppreference.com/w/cpp/container/priority_queue) | A heap-backed adaptor exposing the highest-priority value at top(). |
| [`std::priority_queue::push`](https://en.cppreference.com/w/cpp/container/priority_queue/push) | Insert a value and restore the heap order. |
| [`std::priority_queue::pop`](https://en.cppreference.com/w/cpp/container/priority_queue/pop) | Remove the highest-priority element and restore heap order. |
| [`std::priority_queue::top`](https://en.cppreference.com/w/cpp/container/priority_queue/top) | Access the highest-priority element without removing it. |
| [`std::priority_queue::size`](https://en.cppreference.com/w/cpp/container/priority_queue/size) | Return the number of stored elements. |
| [`std::priority_queue::empty`](https://en.cppreference.com/w/cpp/container/priority_queue/empty) | Test whether the priority queue contains zero elements. |
| [`std::set::find`](https://en.cppreference.com/w/cpp/container/set/find) | Find an element with an equivalent key without inserting. |
| [`std::set::count`](https://en.cppreference.com/w/cpp/container/set/count) | Count elements whose keys are equivalent. |
| [`std::set::erase`](https://en.cppreference.com/w/cpp/container/set/erase) | Remove an element by its key. |
| [`std::set::insert`](https://en.cppreference.com/w/cpp/container/set/insert) | Insert a value only if no equivalent key is present. |
| [`std::unordered_map::find`](https://en.cppreference.com/w/cpp/container/unordered_map/find) | Find an element with an equivalent key without inserting. |
| [`std::unordered_map::count`](https://en.cppreference.com/w/cpp/container/unordered_map/count) | Count elements whose keys are equivalent. |
| [`std::unordered_map::erase`](https://en.cppreference.com/w/cpp/container/unordered_map/erase) | Remove an element by its key. |
| [`std::unordered_map::insert`](https://en.cppreference.com/w/cpp/container/unordered_map/insert) | Insert a value only if no equivalent key is present. |
| [`std::unordered_map::at`](https://en.cppreference.com/w/cpp/container/unordered_map/at) | Access the mapped value for an existing key without inserting. |
| [`std::unordered_map::operator[]`](https://en.cppreference.com/w/cpp/container/unordered_map/operator_at) | Find the mapped value or insert a new value-initialized one for an absent key. |
| [`std::vector::begin`](https://en.cppreference.com/w/cpp/container/vector/begin) | Return an iterator to the first element. |
| [`std::vector::end`](https://en.cppreference.com/w/cpp/container/vector/end) | Return the past-the-end iterator. |
| [`std::vector::assign`](https://en.cppreference.com/w/cpp/container/vector/assign) | Replace all elements with a new sequence. |
| [`std::vector::emplace`](https://en.cppreference.com/w/cpp/container/vector/emplace) | Construct an element before a position from forwarded constructor arguments. |
| [`std::vector::swap`](https://en.cppreference.com/w/cpp/container/vector/swap) | Exchange contents and storage with another vector without moving individual elements. |
| [`std::ifstream`](https://en.cppreference.com/w/cpp/io/basic_ifstream) | An owning input file stream for char text or bytes. |
| [`std::ofstream`](https://en.cppreference.com/w/cpp/io/basic_ofstream) | An owning output file stream for char data. |
| [`std::fstream`](https://en.cppreference.com/w/cpp/io/basic_fstream) | A file stream supporting both input and output. |
| [`std::ostringstream`](https://en.cppreference.com/w/cpp/io/basic_ostringstream) | An output stream backed by an owned string buffer. |
| [`std::stringstream`](https://en.cppreference.com/w/cpp/io/basic_stringstream) | A string-backed stream supporting both input and output. |
| [`std::istream`](https://en.cppreference.com/w/cpp/io/basic_istream) | The char input-stream type. |
| [`std::ostream`](https://en.cppreference.com/w/cpp/io/basic_ostream) | The char output-stream type. |
| [`std::basic_ifstream::open`](https://en.cppreference.com/w/cpp/io/basic_ifstream/open) | Associate the stream with a file. |
| [`std::basic_ifstream::close`](https://en.cppreference.com/w/cpp/io/basic_ifstream/close) | Close the associated file. |
| [`std::basic_ifstream::is_open`](https://en.cppreference.com/w/cpp/io/basic_ifstream/is_open) | Test whether the file buffer is currently associated with an open file. |
| [`std::basic_ofstream::open`](https://en.cppreference.com/w/cpp/io/basic_ofstream/open) | Associate the stream with a file. |
| [`std::basic_ofstream::close`](https://en.cppreference.com/w/cpp/io/basic_ofstream/close) | Close the associated file. |
| [`std::basic_ofstream::is_open`](https://en.cppreference.com/w/cpp/io/basic_ofstream/is_open) | Test whether the file buffer is currently associated with an open file. |
| [`std::basic_istream::get`](https://en.cppreference.com/w/cpp/io/basic_istream/get) | Extract one character without skipping whitespace. |
| [`std::basic_istream::peek`](https://en.cppreference.com/w/cpp/io/basic_istream/peek) | Inspect the next character without extracting it. |
| [`std::basic_istream::read`](https://en.cppreference.com/w/cpp/io/basic_istream/read) | Extract up to count characters into caller-owned storage, without appending a terminator. |
| [`std::basic_istream::ignore`](https://en.cppreference.com/w/cpp/io/basic_istream/ignore) | Discard characters until a count, delimiter or EOF is reached. |
| [`std::basic_istream::gcount`](https://en.cppreference.com/w/cpp/io/basic_istream/gcount) | Report the number of characters extracted by the most recent unformatted input operation. |
| [`std::basic_ostream::put`](https://en.cppreference.com/w/cpp/io/basic_ostream/put) | Write one character without formatting it. |
| [`std::basic_ostream::write`](https://en.cppreference.com/w/cpp/io/basic_ostream/write) | Write exactly count characters from a buffer without treating null bytes as terminators. |
| [`std::basic_ostream::flush`](https://en.cppreference.com/w/cpp/io/basic_ostream/flush) | Synchronize the output buffer with its associated external sequence. |
| [`std::endl`](https://en.cppreference.com/w/cpp/io/manip/endl) | Insert a widened newline character and flush the output stream. |
| [`std::flush`](https://en.cppreference.com/w/cpp/io/manip/flush) | Flush an output stream without inserting any character. |
| [`std::cin`](https://en.cppreference.com/w/cpp/io/cin) | The standard char input stream, associated with C stdin. |
| [`std::cout`](https://en.cppreference.com/w/cpp/io/cout) | The standard char output stream, associated with C stdout. |
| [`std::cerr`](https://en.cppreference.com/w/cpp/io/cerr) | The standard char error stream, associated with C stderr. |
| [`std::begin`](https://en.cppreference.com/w/cpp/iterator/begin) | Return the beginning of a container or built-in array. |
| [`std::end`](https://en.cppreference.com/w/cpp/iterator/end) | Return a past-the-end iterator for a container or built-in array. |
| [`std::cbegin`](https://en.cppreference.com/w/cpp/iterator/begin) | Return a read-only beginning iterator, treating the supplied range as const. |
| [`std::cend`](https://en.cppreference.com/w/cpp/iterator/end) | Return a read-only past-the-end iterator, treating the supplied range as const. |
| [`std::distance`](https://en.cppreference.com/w/cpp/iterator/distance) | Count the iterator increments from first to last. |
| [`std::advance`](https://en.cppreference.com/w/cpp/iterator/advance) | Move an iterator in place by a signed distance. |
| [`std::next`](https://en.cppreference.com/w/cpp/iterator/next) | Return a copy of an iterator moved forward by a count, leaving the original unchanged. |
| [`std::prev`](https://en.cppreference.com/w/cpp/iterator/prev) | Return a copy of an iterator moved backward by a count. |
| [`std::filesystem::path`](https://en.cppreference.com/w/cpp/filesystem/path) | Represent a filesystem path without requiring that it exist. |
| [`std::filesystem::exists`](https://en.cppreference.com/w/cpp/filesystem/exists) | Test whether a path resolves to an existing filesystem object, following symlinks. |
| [`std::filesystem::file_size`](https://en.cppreference.com/w/cpp/filesystem/file_size) | Obtain the size of a regular file in bytes, following symlinks. |
| [`std::filesystem::is_regular_file`](https://en.cppreference.com/w/cpp/filesystem/is_regular_file) | Test whether a path refers to a regular file, following symlinks. |
| [`std::filesystem::is_directory`](https://en.cppreference.com/w/cpp/filesystem/is_directory) | Test whether a path refers to a directory, following symlinks. |
| [`std::filesystem::create_directory`](https://en.cppreference.com/w/cpp/filesystem/create_directory) | Create one directory. |
| [`std::filesystem::create_directories`](https://en.cppreference.com/w/cpp/filesystem/create_directory) | Create missing intermediate directories and the final directory. |
| [`std::filesystem::remove`](https://en.cppreference.com/w/cpp/filesystem/remove) | Remove a single file, symlink or empty directory. |
| [`std::filesystem::rename`](https://en.cppreference.com/w/cpp/filesystem/rename) | Rename or move an object as if by the corresponding POSIX rename rules, including supported replacement cases. |
| [`std::filesystem::current_path`](https://en.cppreference.com/w/cpp/filesystem/current_path) | Get or change the process-wide working directory. |
| [`std::filesystem::directory_iterator`](https://en.cppreference.com/w/cpp/filesystem/directory_iterator) | A single-pass input iterator over entries in one directory, excluding the special dot and dot-dot entries. |
| [`std::filesystem::path::filename`](https://en.cppreference.com/w/cpp/filesystem/path/filename) | Return the final filename component without accessing the filesystem. |
| [`std::filesystem::path::stem`](https://en.cppreference.com/w/cpp/filesystem/path/stem) | Return the filename with its last extension removed. |
| [`std::filesystem::path::extension`](https://en.cppreference.com/w/cpp/filesystem/path/extension) | Return the last filename extension including its leading dot. |
| [`std::filesystem::path::parent_path`](https://en.cppreference.com/w/cpp/filesystem/path/parent_path) | Return the path with its final filename component removed, without looking up the filesystem. |
| [`std::filesystem::path::empty`](https://en.cppreference.com/w/cpp/filesystem/path/empty) | Test whether the path stores no characters. |
| [`std::filesystem::path::is_absolute`](https://en.cppreference.com/w/cpp/filesystem/path/is_absrel) | Test whether the stored path is absolute according to the platform path rules. |
| [`std::filesystem::path::string`](https://en.cppreference.com/w/cpp/filesystem/path/string) | Obtain a narrow-character representation of a path. |

## C library (126)

| Symbol | Description |
| --- | --- |
| [`printf`](https://en.cppreference.com/w/c/io/fprintf) | Write formatted output to standard output. |
| [`fprintf`](https://en.cppreference.com/w/c/io/fprintf) | Write formatted output to a specified stream. |
| [`snprintf`](https://en.cppreference.com/w/c/io/fprintf) | Format output into a bounded character buffer. |
| [`scanf`](https://en.cppreference.com/w/c/io/fscanf) | Read formatted input from standard input. |
| [`fopen`](https://en.cppreference.com/w/c/io/fopen) | Open a file and associate a stream with it. |
| [`fclose`](https://en.cppreference.com/w/c/io/fclose) | Flush buffered output and close a stream. |
| [`fread`](https://en.cppreference.com/w/c/io/fread) | Read up to count objects of size bytes into a buffer. |
| [`fwrite`](https://en.cppreference.com/w/c/io/fwrite) | Write count objects of size bytes from a buffer to a stream. |
| [`fgets`](https://en.cppreference.com/w/c/io/fgets) | Read at most capacity - 1 characters and append a null terminator on success. |
| [`fputs`](https://en.cppreference.com/w/c/io/fputs) | Write a null-terminated string to a stream without writing its terminating null character. |
| [`fgetc`](https://en.cppreference.com/w/c/io/fgetc) | Read one byte from a stream and advance its position. |
| [`fputc`](https://en.cppreference.com/w/c/io/fputc) | Convert a value to unsigned char and write that byte to a stream. |
| [`fflush`](https://en.cppreference.com/w/c/io/fflush) | Write buffered output to the underlying file for an output stream, or an update stream whose last operation was not input. |
| [`fseek`](https://en.cppreference.com/w/c/io/fseek) | Set a stream position relative to an origin and clear its end-of-file indicator on success. |
| [`ftell`](https://en.cppreference.com/w/c/io/ftell) | Obtain the current stream position. |
| [`feof`](https://en.cppreference.com/w/c/io/feof) | Test the end-of-file indicator already stored in a stream. |
| [`ferror`](https://en.cppreference.com/w/c/io/ferror) | Test the error indicator stored in a stream. |
| [`perror`](https://en.cppreference.com/w/c/io/perror) | Write a message for the current errno value to standard error. |
| [`malloc`](https://en.cppreference.com/w/c/memory/malloc) | Allocate uninitialized storage of the requested byte size, suitably aligned for types with fundamental alignment requirements. |
| [`calloc`](https://en.cppreference.com/w/c/memory/calloc) | Allocate storage for count objects and set every byte to zero. |
| [`realloc`](https://en.cppreference.com/w/c/memory/realloc) | Change the size of a malloc-family allocation. |
| [`free`](https://en.cppreference.com/w/c/memory/free) | Release storage obtained from malloc, calloc or realloc. |
| [`memcpy`](https://en.cppreference.com/w/c/string/byte/memcpy) | Copy exactly count bytes between non-overlapping memory regions. |
| [`memmove`](https://en.cppreference.com/w/c/string/byte/memmove) | Copy exactly count bytes between memory regions, correctly handling overlap as if the source bytes were first copied to a temporary buffer. |
| [`memset`](https://en.cppreference.com/w/c/string/byte/memset) | Set each of count bytes to value converted to unsigned char. |
| [`memcmp`](https://en.cppreference.com/w/c/string/byte/memcmp) | Compare exactly count bytes as unsigned char values. |
| [`strlen`](https://en.cppreference.com/w/c/string/byte/strlen) | Count bytes before the first null character in a string. |
| [`strcmp`](https://en.cppreference.com/w/c/string/byte/strcmp) | Compare two null-terminated strings lexicographically as unsigned char sequences. |
| [`strncmp`](https://en.cppreference.com/w/c/string/byte/strncmp) | Compare at most count characters of two strings, stopping earlier at a null terminator. |
| [`strcpy`](https://en.cppreference.com/w/c/string/byte/strcpy) | Copy a null-terminated string, including its terminator, into a separate buffer. |
| [`strncpy`](https://en.cppreference.com/w/c/string/byte/strncpy) | Copy at most count source characters, padding with null bytes until count bytes have been written if the source ends earlier. |
| [`strchr`](https://en.cppreference.com/w/c/string/byte/strchr) | Find the first occurrence of a character, converted to char, in a null-terminated string. |
| [`strstr`](https://en.cppreference.com/w/c/string/byte/strstr) | Find the first occurrence of a null-terminated substring in a null-terminated string. |
| [`strtol`](https://en.cppreference.com/w/c/string/byte/strtol) | Parse an integer prefix after optional whitespace and sign. |
| [`strtod`](https://en.cppreference.com/w/c/string/byte/strtof) | Parse a floating-point prefix after optional whitespace and sign. |
| [`qsort`](https://en.cppreference.com/w/c/algorithm/qsort) | Sort an array in place using a comparison callback. |
| [`FILE`](https://en.cppreference.com/w/c/io/FILE) | The opaque object type used by C streams. |
| [`tm`](https://en.cppreference.com/w/c/chrono/tm) | A structure holding a broken-down calendar date and time. |
| [`EOF`](https://en.cppreference.com/w/c/io) | A negative int constant used by byte input functions to report end of file or a read error, and by some output functions to report failure. |
| [`assert`](https://en.cppreference.com/w/c/error/assert) | Check an invariant in a debug-enabled build. |
| [`sprintf`](https://en.cppreference.com/w/c/io/fprintf) | Format output into a character array, including a trailing null character. |
| [`sscanf`](https://en.cppreference.com/w/c/io/fscanf) | Read formatted values from a null-terminated string. |
| [`fscanf`](https://en.cppreference.com/w/c/io/fscanf) | Read formatted values from a stream. |
| [`getchar`](https://en.cppreference.com/w/c/io/getchar) | Read one character from standard input. |
| [`putchar`](https://en.cppreference.com/w/c/io/putchar) | Write one character to standard output after conversion to unsigned char. |
| [`puts`](https://en.cppreference.com/w/c/io/puts) | Write a null-terminated string to standard output and then append a newline. |
| [`rewind`](https://en.cppreference.com/w/c/io/rewind) | Move a stream to its beginning and clear its error and end-of-file indicators. |
| [`clearerr`](https://en.cppreference.com/w/c/io/clearerr) | Clear the error and end-of-file indicators of a stream. |
| [`remove`](https://en.cppreference.com/w/c/io/remove) | Delete a file named by a path. |
| [`rename`](https://en.cppreference.com/w/c/io/rename) | Change the name of a file. |
| [`tmpfile`](https://en.cppreference.com/w/c/io/tmpfile) | Create a temporary binary update stream, opened as if in wb+ mode. |
| [`setvbuf`](https://en.cppreference.com/w/c/io/setvbuf) | Choose buffering for a stream before any other operation on that stream, apart from an unsuccessful setvbuf call. |
| [`ungetc`](https://en.cppreference.com/w/c/io/ungetc) | Push a character back onto an input stream so it is returned by the next read. |
| [`fgetpos`](https://en.cppreference.com/w/c/io/fgetpos) | Save a stream position and its parsing state in an fpos_t object. |
| [`fsetpos`](https://en.cppreference.com/w/c/io/fsetpos) | Restore a position previously saved by fgetpos on the same stream. |
| [`fpos_t`](https://en.cppreference.com/w/c/io/fpos_t) | An opaque type holding a stream position and any associated multibyte conversion state. |
| [`stdin`](https://en.cppreference.com/w/c/io/std_streams) | A macro yielding a FILE pointer for standard input. |
| [`stdout`](https://en.cppreference.com/w/c/io/std_streams) | A macro yielding a FILE pointer for standard output. |
| [`stderr`](https://en.cppreference.com/w/c/io/std_streams) | A macro yielding a FILE pointer for standard error. |
| [`atoi`](https://en.cppreference.com/w/c/string/byte/atoi) | Convert an initial decimal integer after optional whitespace and sign. |
| [`atol`](https://en.cppreference.com/w/c/string/byte/atoi) | Convert an initial decimal integer after optional whitespace and sign. |
| [`atoll`](https://en.cppreference.com/w/c/string/byte/atoi) | Convert an initial decimal integer after optional whitespace and sign. |
| [`strtoll`](https://en.cppreference.com/w/c/string/byte/strtol) | Parse an integer prefix after optional whitespace and sign, with an end pointer and range checking. |
| [`strtoul`](https://en.cppreference.com/w/c/string/byte/strtoul) | Parse an integer prefix after optional whitespace and sign, with an end pointer and range checking. |
| [`strtoull`](https://en.cppreference.com/w/c/string/byte/strtoul) | Parse an integer prefix after optional whitespace and sign, with an end pointer and range checking. |
| [`abs`](https://en.cppreference.com/w/c/numeric/math/abs) | Compute the absolute value of a signed integer. |
| [`labs`](https://en.cppreference.com/w/c/numeric/math/abs) | Compute the absolute value of a signed integer. |
| [`div`](https://en.cppreference.com/w/c/numeric/math/div) | Compute an integer quotient and remainder together. |
| [`ldiv`](https://en.cppreference.com/w/c/numeric/math/div) | Compute an integer quotient and remainder together. |
| [`bsearch`](https://en.cppreference.com/w/c/algorithm/bsearch) | Find an element in an array sorted according to the same comparison relation. |
| [`rand`](https://en.cppreference.com/w/c/numeric/random/rand) | Return the next value from the implementation-defined pseudo-random sequence. |
| [`srand`](https://en.cppreference.com/w/c/numeric/random/srand) | Seed the pseudo-random sequence used by rand. |
| [`getenv`](https://en.cppreference.com/w/c/program/getenv) | Look up an environment variable. |
| [`exit`](https://en.cppreference.com/w/c/program/exit) | Terminate the program normally, call registered atexit handlers, flush and close C streams and remove temporary files. |
| [`abort`](https://en.cppreference.com/w/c/program/abort) | Terminate the program abnormally by raising SIGABRT. |
| [`EXIT_SUCCESS`](https://en.cppreference.com/w/c/program/EXIT_status) | A macro expanding to an integer constant expression representing successful program termination. |
| [`EXIT_FAILURE`](https://en.cppreference.com/w/c/program/EXIT_status) | A macro expanding to an integer constant expression representing unsuccessful program termination. |
| [`RAND_MAX`](https://en.cppreference.com/w/c/numeric/random/RAND_MAX) | The largest value rand can return. |
| [`strcat`](https://en.cppreference.com/w/c/string/byte/strcat) | Append a null-terminated string, including its terminator, to another string. |
| [`strncat`](https://en.cppreference.com/w/c/string/byte/strncat) | Append at most count source characters and always add a terminating null character. |
| [`strrchr`](https://en.cppreference.com/w/c/string/byte/strrchr) | Find the last occurrence of a character in a null-terminated string, including the terminating null character in the search. |
| [`strcspn`](https://en.cppreference.com/w/c/string/byte/strcspn) | Count the initial characters of a string that contain none of the characters in a reject set. |
| [`strspn`](https://en.cppreference.com/w/c/string/byte/strspn) | Count the initial characters of a string that all belong to an accept set. |
| [`strpbrk`](https://en.cppreference.com/w/c/string/byte/strpbrk) | Find the first character of a string that belongs to a supplied character set. |
| [`strtok`](https://en.cppreference.com/w/c/string/byte/strtok) | Split a writable string into nonempty tokens by replacing delimiters with null characters. |
| [`memchr`](https://en.cppreference.com/w/c/string/byte/memchr) | Search exactly count bytes for value converted to unsigned char. |
| [`strerror`](https://en.cppreference.com/w/c/string/byte/strerror) | Return implementation-provided text describing an error number. |
| [`isalpha`](https://en.cppreference.com/w/c/string/byte/isalpha) | Test whether a value is an alphabetic character in the current C locale. |
| [`isdigit`](https://en.cppreference.com/w/c/string/byte/isdigit) | Test whether a value is one of the decimal digits 0 through 9. |
| [`isspace`](https://en.cppreference.com/w/c/string/byte/isspace) | Test whether a value is a whitespace character in the current C locale. |
| [`tolower`](https://en.cppreference.com/w/c/string/byte/tolower) | Convert a character using the current C locale. |
| [`toupper`](https://en.cppreference.com/w/c/string/byte/toupper) | Convert a character using the current C locale. |
| [`sqrt`](https://en.cppreference.com/w/c/numeric/math/sqrt) | Compute the nonnegative square root. |
| [`pow`](https://en.cppreference.com/w/c/numeric/math/pow) | Raise base to exponent. |
| [`fabs`](https://en.cppreference.com/w/c/numeric/math/fabs) | Compute the absolute value of a floating-point number. |
| [`floor`](https://en.cppreference.com/w/c/numeric/math/floor) | Round toward negative infinity. |
| [`ceil`](https://en.cppreference.com/w/c/numeric/math/ceil) | Round toward positive infinity. |
| [`round`](https://en.cppreference.com/w/c/numeric/math/round) | Round to the nearest integer value, with halfway cases away from zero, independently of the current rounding mode. |
| [`sin`](https://en.cppreference.com/w/c/numeric/math/sin) | Compute the sine of an angle expressed in radians, not degrees. |
| [`cos`](https://en.cppreference.com/w/c/numeric/math/cos) | Compute the cosine of an angle expressed in radians, not degrees. |
| [`tan`](https://en.cppreference.com/w/c/numeric/math/tan) | Compute the tangent of an angle in radians. |
| [`asin`](https://en.cppreference.com/w/c/numeric/math/asin) | Compute the principal inverse sine in radians. |
| [`acos`](https://en.cppreference.com/w/c/numeric/math/acos) | Compute the principal inverse cosine in radians. |
| [`atan`](https://en.cppreference.com/w/c/numeric/math/atan) | Compute the principal inverse tangent in radians, with a result in [-pi/2, pi/2]. |
| [`atan2`](https://en.cppreference.com/w/c/numeric/math/atan2) | Compute the polar angle of the vector (x, y) in radians, using both signs to select a quadrant. |
| [`exp`](https://en.cppreference.com/w/c/numeric/math/exp) | Compute e raised to a floating-point power. |
| [`log`](https://en.cppreference.com/w/c/numeric/math/log) | Compute the natural logarithm. |
| [`log10`](https://en.cppreference.com/w/c/numeric/math/log10) | Compute the base-10 logarithm. |
| [`fmod`](https://en.cppreference.com/w/c/numeric/math/fmod) | Compute the remainder of x/y with the quotient truncated toward zero. |
| [`time`](https://en.cppreference.com/w/c/chrono/time) | Obtain the current calendar time in the implementation-defined time_t representation. |
| [`difftime`](https://en.cppreference.com/w/c/chrono/difftime) | Compute elapsed calendar seconds between two time_t values. |
| [`clock`](https://en.cppreference.com/w/c/chrono/clock) | ISO C measures processor time used by the program. |
| [`localtime`](https://en.cppreference.com/w/c/chrono/localtime) | Convert a calendar time into local calendar components using the implementation time-zone rules. |
| [`gmtime`](https://en.cppreference.com/w/c/chrono/gmtime) | Convert a calendar time into UTC calendar components. |
| [`mktime`](https://en.cppreference.com/w/c/chrono/mktime) | Interpret calendar components as local time, normalize out-of-range fields and compute the corresponding calendar time. |
| [`strftime`](https://en.cppreference.com/w/c/chrono/strftime) | Format calendar components into a bounded character buffer using a format string and the current locale. |
| [`time_t`](https://en.cppreference.com/w/c/chrono/time_t) | An arithmetic type representing calendar time. |
| [`clock_t`](https://en.cppreference.com/w/c/chrono/clock_t) | A real arithmetic type representing the time reported by clock (CPU time in ISO C; elapsed time in Microsoft CRT). |
| [`CLOCKS_PER_SEC`](https://en.cppreference.com/w/c/chrono/CLOCKS_PER_SEC) | A macro giving the number of clock ticks per second for converting clock results. |
| [`errno`](https://en.cppreference.com/w/c/error/errno) | A macro yielding a modifiable int lvalue used by library error-reporting interfaces. |
| [`ERANGE`](https://en.cppreference.com/w/c/error/errno_macros) | A positive integer error-code macro indicating that a result is outside the representable range. |
| [`NULL`](https://en.cppreference.com/w/c/types/NULL) | An implementation-defined null pointer constant. |
| [`offsetof`](https://en.cppreference.com/w/c/types/offsetof) | Compute the byte offset of a member within a structure as a size_t constant expression. |
| [`SIZE_MAX`](https://en.cppreference.com/w/c/types/limits) | The largest value representable by size_t. |
| [`INT_MAX`](https://en.cppreference.com/w/c/types/limits) | The largest value representable by int. |
| [`INT_MIN`](https://en.cppreference.com/w/c/types/limits) | The smallest value representable by int. |

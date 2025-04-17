

# on 17.04.2025
thought it's too complex and bad design decisions in data parsing and endianness transmission.
Designed a new todo list, and a simplified library design.
Big mess was there with traits and from_bytes, etc - now try_into is better, as well as the hesitation in using the full type vs `&[u8]`.
There might be another rewrite for Maturin python compatibility.
also each kind of parsing as a feature, for compile time inclusion.


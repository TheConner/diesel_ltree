# diesel_ltree

Adds support for the `ltree` PostgreSQL extension type to Diesel.

Currently adding all functions and operators supported by Postrges.

Unfortunately it's not possible to add support for representing the actual ltree data natively, as the extension doesn't support transmitting ltree data in binary format, and libpq doesn't support per-column result formats ([#1](https://github.com/kivikakk/diesel_ltree/issue/1)).

# diesel_ltree [![Build Status](https://travis-ci.org/kivikakk/diesel_ltree.svg?branch=master)](https://travis-ci.org/kivikakk/diesel_ltree) [![crates.io version](https://img.shields.io/crates/v/diesel_ltree.svg)](https://crates.io/crates/diesel_ltree)

Adds support for the `ltree` PostgreSQL extension type to Diesel, supporting all functions and operators thereon.

## A note about `Insertable` and Postgres Ltree binary protocol support

Binary protocol support for Ltrees (has been committed)[https://commitfest.postgresql.org/24/2242/], but not released as of 2020.04.29. As such, the only option for Diesel, which uses the Postgres binary protocol (likewise for rust_postgres), is to treat the transmittion of Ltrees as Postgres `text`, until this commit goes live. This crate currently transmits Ltree paths as `text`, then casts them to `ltree`.


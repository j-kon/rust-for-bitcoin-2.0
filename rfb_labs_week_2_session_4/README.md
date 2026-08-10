# Rust for Bitcoin 2.0 — Week 2, Session 4

Build a small lending library while practising structs, enums, traits,
ownership, borrowing, collections, and `Result`-based error handling. No
Bitcoin and no external crates — just Rust.

The crate is intentionally incomplete. Search for `TODO` and implement each
part; do not change the public type names or function signatures.

## Recommended workflow

1. Read [ASSIGNMENT.md](ASSIGNMENT.md).
2. Complete Part 2 in `error.rs`, then Part 3 in `library.rs`.
3. Remove `#[ignore]` from the relevant test and run it.
4. Complete the traits in Part 4 and the two operations in Parts 5–6.
5. Run the ownership experiments and record the errors.
6. Build the demo in `main.rs`.
7. Add the remaining required tests yourself.

```bash
cargo test
cargo test -- --ignored
cargo run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

`cargo test` checks the starter project. Ignored tests intentionally exercise
unfinished code; enable them progressively rather than leaving them ignored in
the submission.

## Ownership and Borrowing Experiments

### Experiment A

```text
error[E0382]: borrow of moved value: `item`
  --> src/main.rs:14:20
   |
 7 |     let item = rfb_labs_week_2_session_4::Item::new(
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
...
13 |     library.add_item(item)?;
   |                      ---- value moved here
14 |     println!("{}", item.title);
   |                    ^^^^^^^^^^ value borrowed here after move
```

#### Explanation
- **What value was moved:** The variable `item` of type `Item` was passed by value into `library.add_item(item)`.
- **Why ownership transferred:** `add_item` takes ownership (`item: Item`) to store the item inside `Library`'s internal `items: Vec<Item>` collection. Because `Item` contains `String` fields and does not implement `Copy`, ownership is transferred into the library.
- **Why later use is rejected:** Once moved, `item` in the caller's stack frame becomes uninitialized and invalid. Attempting to read `item.title` violates Rust's move semantics.
- **What would change if borrowed:** If `add_item` took `&Item`, `Library` would either need to clone the entire item or store borrowed references requiring explicit lifetimes across the struct. Taking ownership by value is the cleanest approach.

### Experiment B

```text
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:18:5
   |
17 |     let held = library.find_item(1);
   |                ------- immutable borrow occurs here
18 |     library.checkout(1, 100, 5)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
19 |     if let Some(i) = held {
   |                      ---- immutable borrow later used here
```

#### Explanation
- **Borrow conflict:** `let held = library.find_item(1);` creates an immutable reference (`&Item`) tied to the lifetime of `library`. Calling `library.checkout(...)` on line 18 requires an exclusive mutable reference (`&mut library`).
- **Rust Aliasing Rule:** Rust's borrow checker prohibits simultaneous active immutable and mutable borrows to the same data structure to guarantee memory safety and prevent data races.
- **Resolution:** Narrowing the scope of `held` so that the immutable reference drops before calling `checkout` (or performing lookups sequentially) satisfies the borrow checker.

## Written answers

Answer in your own words. Add both ownership compiler errors from Part 7 as
fenced text blocks, then explain what caused each.

1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?
2. What does `match` force you to do when a fourth `MediaKind` is added later?
3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?
4. Why does `add_item` take `self` by `&mut` but `item` by value?
5. When `add_item` returns `Err`, what happened to the `Item` the caller passed
   in? Was that a good design choice, and what is the alternative?
6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?
7. What is the lifetime `'a` in `items_by_author` actually saying?
8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same
   `Library` at once, and how did you structure the method around that?
9. Why are `Library`'s fields private?
10. What duplication does the provided `late_fee_cents` remove, and what would
    you lose by making it a free function instead?
11. Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.
12. Which derive did you deliberately leave off a type, and why?

## Design notes

Describe any choices you made, including how you kept an item's status and its
borrower's list from drifting apart, and (if attempted) the optional generic
search.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.

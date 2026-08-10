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

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?**  
   If `LoanStatus` were modeled using `is_on_loan: bool`, `member_id: Option<u32>`, and `day_borrowed: Option<u32>`, it would allow invalid state combinations such as `is_on_loan = false` with `member_id = Some(100)` or `is_on_loan = true` with `day_borrowed = None`. Modeling state with Rust enums makes invalid states unrepresentable in the type system. Each variant (`Available`, `OnLoan { member_id, day_borrowed }`, `Lost`) carries exactly the payload required for that state.

2. **What does `match` force you to do when a fourth `MediaKind` is added later?**  
   Rust's `match` expressions are exhaustive. If a fourth `MediaKind` variant (such as `Magazine { issue: u32 }`) is added to the enum, the Rust compiler will flag every unhandled `match` across the codebase as a compile-time error. This guarantees that new variants are explicitly handled across all loan duration, fee calculation, and formatting logic.

3. **`Item::new` takes `String` rather than `&str`. Who owns the title afterwards?**  
   When `Item::new` accepts `title: String`, ownership of the allocated string buffer is transferred to `Item::new`, which moves it into the newly constructed `Item` struct. The `Item` instance becomes the sole owner of the title string afterwards.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**  
   `add_item` takes `&mut self` because it mutates the library's internal state by pushing a new element to `self.items`. It takes `item` by value (`Item`) so that ownership of the item is transferred from the caller directly into the library's internal storage (`Vec<Item>`) without requiring allocation clones.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller passed in? Was that a good design choice, and what is the alternative?**  
   When `add_item` returns an `Err(LibraryError)`, the `item` passed by value was already moved into `add_item`'s scope and gets dropped at the end of the function call when returning the error. In standard Rust APIs, this is acceptable for simple values, but an alternative signature `pub fn add_item(&mut self, item: Item) -> Result<(), (Item, LibraryError)>` could return the un-added `Item` back to the caller so they retain ownership if addition fails.

6. **Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**  
   `find_item` returns `Option<&Item>` to grant callers read-only access to the item stored inside the library via borrowing. Returning an owned `Option<Item>` would require removing the item from the library or cloning the entire `Item` struct (including heap-allocated strings), incurring unnecessary overhead.

7. **What is the lifetime `'a` in `items_by_author` actually saying?**  
   The signature `pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item>` states that the returned vector of `&Item` references is bound to the lifetime `'a` of the `&'a self` borrow. It guarantees to the compiler that the returned item references remain valid as long as the borrowing view of `Library` (`'a`) remains active.

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same `Library` at once, and how did you structure the method around that?**  
   Borrowing `&mut self.items` and `&mut self.members` simultaneously as separate mutable references to fields of `self` within a single method call can trigger borrow checker errors if done through method calls taking `&mut self`. We structured `checkout` by performing all validation checks first via immutable references (`&self`), and then performing narrow, non-overlapping mutable lookups (`items.iter_mut()` and `members.iter_mut()`) strictly in the mutation phase after validation succeeds.

9. **Why are `Library`'s fields private?**  
   `Library`'s fields (`items` and `members`) are private to enforce encapsulation and domain invariant consistency. The library must maintain a strict bidirectional invariant: whenever an item's status is `OnLoan { member_id, .. }`, that item's ID must appear in the corresponding member's `borrowed_item_ids` list. Keeping fields private prevents external code from mutating items or members directly and causing state drift.

10. **What duplication does the provided `late_fee_cents` remove, and what would you lose by making it a free function instead?**  
    `late_fee_cents` provides a default trait method on `LoanTerms` that calculates `(days_held - loan_days) * daily_late_fee_cents`. This prevents every implementor (`MediaKind` and `Item`) from repeating identical overdue threshold and fee multiplication math. Making it a free function would lose trait polymorphism, preventing callers from invoking `.late_fee_cents(...)` dynamically on any type implementing `LoanTerms`.

11. **Why is `Result` preferable to `panic!` for validation failures? Name a place in this crate where a panic would be defensible.**  
    Validation failures (e.g. unknown item ID, member borrow limit reached) represent expected runtime outcomes caused by invalid caller inputs. Returning `Result` forces callers to explicitly handle errors without crashing the application thread. A `panic!` or `.unwrap()` is defensible during internal state mutation steps (such as `self.items.iter_mut().find(...).unwrap()`) after read-only validation has already guaranteed that the item exists.

12. **Which derive did you deliberately leave off a type, and why?**  
    `Copy` was deliberately omitted from `Item` and `Member`. Both types own heap-allocated `String` buffers (`title`, `author`, `name`) and collections (`borrowed_item_ids`). Types containing non-`Copy` owned data cannot implement `Copy` because bitwise copying would create duplicate pointers to the same heap allocations, leading to double-free undefined behavior.

## Design notes

### State Synchronization

To prevent state drift where an item's `LoanStatus` disagrees with a member's `borrowed_item_ids`:
- In `checkout`: validation verifies that both item and member exist, item is available, and member is within limits. State mutation then updates `item.status = LoanStatus::OnLoan { member_id, day_borrowed: day }` and appends `item_id` to `member.borrowed_item_ids` in a single atomic transaction.
- In `return_item`: item status is reset to `LoanStatus::Available` and `item_id` is removed from `member.borrowed_item_ids` using `.retain()`.

### Generic Item Filtering (Part 9)

`filter_items` is implemented as a higher-order function taking a closure `predicate: F` where `F: Fn(&Item) -> bool`. Both `items_by_author` and `available_items` are re-expressed using `filter_items` to eliminate duplicate iteration logic.

## Example output

```text
=== Initial Stock ===
[1] "The Rust Programming Language" by Steve Klabnik (Book (560 pages)) - Available
[2] "Project Hail Mary" by Andy Weir (Audiobook (540 mins)) - Available
[3] "Programming Bitcoin" by Jimmy Song (Ebook (4500 KB)) - Available

=== Checking out Item #1 on Day 5 ===
Checked out: [1] "The Rust Programming Language" by Steve Klabnik (Book (560 pages)) - On Loan to member 100 since day 5

=== Returning Item #1 on Day 35 ===
Item returned successfully. Late fee owed: 225 cents ($2.25)

=== Triggering Handled Error ===
Handled expected error: item 999 not found
```

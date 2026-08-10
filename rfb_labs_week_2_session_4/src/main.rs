//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    // 1. Stock multiple items
    library.add_item(Item::new(
        1,
        "The Rust Programming Language".into(),
        "Steve Klabnik".into(),
        MediaKind::Book { pages: 560 },
    ))?;

    library.add_item(Item::new(
        2,
        "Project Hail Mary".into(),
        "Andy Weir".into(),
        MediaKind::Audiobook { minutes: 540 },
    ))?;

    library.add_item(Item::new(
        3,
        "Programming Bitcoin".into(),
        "Jimmy Song".into(),
        MediaKind::Ebook { size_kb: 4_500 },
    ))?;

    // 2. Register a member
    library.register_member(Member::new(100, "Satoshi".into()))?;

    println!("=== Initial Stock ===");
    for item in library.available_items() {
        println!("{item}");
    }

    // 3. Checkout an item on day 5
    println!("\n=== Checking out Item #1 on Day 5 ===");
    library.checkout(1, 100, 5)?;

    if let Some(item) = library.find_item(1) {
        println!("Checked out: {item}");
    }

    // 4. Return the item late on day 35
    println!("\n=== Returning Item #1 on Day 35 ===");
    let late_fee_cents = library.return_item(1, 35)?;
    println!(
        "Item returned successfully. Late fee owed: {late_fee_cents} cents (${:.2})",
        late_fee_cents as f64 / 100.0
    );

    // 5. Intentionally trigger one handled error
    println!("\n=== Triggering Handled Error ===");
    let invalid_checkout = library.checkout(999, 100, 40);
    if let Err(err) = invalid_checkout {
        println!("Handled expected error: {err}");
    }

    Ok(())
}

use rfb_labs_week_2_session_4::{
    Item, Library, LibraryError, LoanStatus, LoanTerms, MediaKind, Member,
};

fn library_with_items() -> Library {
    let mut library = Library::new();

    for (id, title, author, kind) in [
        (1, "Dune", "Frank Herbert", MediaKind::Book { pages: 320 }),
        (
            2,
            "Children of Dune",
            "Frank Herbert",
            MediaKind::Book { pages: 180 },
        ),
        (
            3,
            "Project Hail Mary",
            "Andy Weir",
            MediaKind::Audiobook { minutes: 540 },
        ),
        (
            4,
            "The Rust Programming Language",
            "Steve Klabnik",
            MediaKind::Ebook { size_kb: 1_200 },
        ),
    ] {
        library
            .add_item(Item::new(id, title.into(), author.into(), kind))
            .unwrap();
    }

    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();

    library
}

#[test]
fn checkout_updates_both_the_item_and_the_member() {
    let mut library = library_with_items();

    library.checkout(1, 100, 5).unwrap();

    assert_eq!(
        library.find_item(1).unwrap().status,
        LoanStatus::OnLoan {
            member_id: 100,
            day_borrowed: 5,
        }
    );
    assert_eq!(library.find_member(100).unwrap().borrowed_item_ids, vec![1]);
}

#[test]
fn a_member_cannot_exceed_the_borrow_limit() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();
    library.checkout(2, 100, 0).unwrap();
    library.checkout(3, 100, 0).unwrap();

    assert_eq!(
        library.checkout(4, 100, 0),
        Err(LibraryError::BorrowLimitReached {
            member_id: 100,
            limit: 3,
        })
    );
}

#[test]
fn returning_a_book_late_charges_a_daily_fee() {
    let mut library = library_with_items();

    // A book may be kept 21 days. Held for 30, so 9 days are overdue.
    library.checkout(1, 100, 10).unwrap();

    assert_eq!(library.return_item(1, 40), Ok(9 * 25));
    assert_eq!(library.find_item(1).unwrap().status, LoanStatus::Available);
    assert!(
        library
            .find_member(100)
            .unwrap()
            .borrowed_item_ids
            .is_empty()
    );
}

#[test]
fn searching_by_author_borrows_rather_than_clones() {
    let library = library_with_items();

    let found = library.items_by_author("Frank Herbert");

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].title, "Dune");
    // `found` holds references into `library`, so these are the same item.
    assert!(std::ptr::eq(found[0], library.find_item(1).unwrap()));
}

#[test]
fn cannot_lend_an_item_twice() {
    let mut library = library_with_items();
    library.checkout(1, 100, 5).unwrap();

    assert_eq!(
        library.checkout(1, 100, 10),
        Err(LibraryError::ItemAlreadyOnLoan {
            id: 1,
            member_id: 100,
        })
    );
}

#[test]
fn on_time_return_owes_zero_fee() {
    let mut library = library_with_items();
    // Book allows 21 days. Borrowed day 5, returned day 25 (20 days held).
    library.checkout(1, 100, 5).unwrap();

    assert_eq!(library.return_item(1, 25), Ok(0));
    assert_eq!(library.find_item(1).unwrap().status, LoanStatus::Available);
}

#[test]
fn ebook_returned_late_still_owes_zero_fee() {
    let mut library = library_with_items();
    // Item #4 is an Ebook (allows 7 days). Borrowed day 0, returned day 100 (100 days held).
    library.checkout(4, 100, 0).unwrap();

    assert_eq!(library.return_item(4, 100), Ok(0));
}

#[test]
fn author_search_includes_borrowed_items() {
    let mut library = library_with_items();
    // Borrow Item #1 ("Dune" by Frank Herbert)
    library.checkout(1, 100, 5).unwrap();

    let herbert_items = library.items_by_author("Frank Herbert");
    assert_eq!(herbert_items.len(), 2);
    assert_eq!(herbert_items[0].id, 1);
    assert_eq!(herbert_items[1].id, 2);
}

#[test]
fn error_empty_title() {
    let mut library = Library::new();
    let err = library.add_item(Item::new(
        1,
        "   ".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    ));
    assert_eq!(err, Err(LibraryError::EmptyTitle));
}

#[test]
fn error_duplicate_item_id() {
    let mut library = library_with_items();
    let err = library.add_item(Item::new(
        1,
        "New Title".into(),
        "New Author".into(),
        MediaKind::Book { pages: 100 },
    ));
    assert_eq!(err, Err(LibraryError::DuplicateItemId { id: 1 }));
}

#[test]
fn error_duplicate_member_id() {
    let mut library = library_with_items();
    let err = library.register_member(Member::new(100, "Duplicate Ada".into()));
    assert_eq!(err, Err(LibraryError::DuplicateMemberId { id: 100 }));
}

#[test]
fn error_checkout_unknown_item() {
    let mut library = library_with_items();
    assert_eq!(
        library.checkout(999, 100, 5),
        Err(LibraryError::ItemNotFound { id: 999 })
    );
}

#[test]
fn error_checkout_unknown_member() {
    let mut library = library_with_items();
    assert_eq!(
        library.checkout(1, 999, 5),
        Err(LibraryError::MemberNotFound { id: 999 })
    );
}

#[test]
fn error_checkout_lost_item() {
    let mut library = Library::new();
    let mut item = Item::new(
        1,
        "Lost Book".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    item.status = LoanStatus::Lost;
    library.add_item(item).unwrap();
    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();

    assert_eq!(
        library.checkout(1, 100, 5),
        Err(LibraryError::ItemIsLost { id: 1 })
    );
}

#[test]
fn error_return_unknown_item() {
    let mut library = library_with_items();
    assert_eq!(
        library.return_item(999, 10),
        Err(LibraryError::ItemNotFound { id: 999 })
    );
}

#[test]
fn error_return_item_not_on_loan() {
    let mut library = library_with_items();
    assert_eq!(
        library.return_item(1, 10),
        Err(LibraryError::ItemNotOnLoan { id: 1 })
    );
}

#[test]
fn error_return_lost_item() {
    let mut library = Library::new();
    let mut item = Item::new(
        1,
        "Lost Book".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    item.status = LoanStatus::Lost;
    library.add_item(item).unwrap();

    assert_eq!(
        library.return_item(1, 10),
        Err(LibraryError::ItemIsLost { id: 1 })
    );
}

#[test]
fn error_return_day_earlier_than_borrow_day() {
    let mut library = library_with_items();
    library.checkout(1, 100, 20).unwrap();

    assert_eq!(
        library.return_item(1, 10),
        Err(LibraryError::InvalidReturnDay {
            day_borrowed: 20,
            day_returned: 10,
        })
    );
}

#[test]
fn longest_loan_item_query() {
    let library = library_with_items();
    let longest = library.longest_loan_item().unwrap();
    // Book has 21 days (highest)
    assert_eq!(longest.kind.loan_days(), 21);
}

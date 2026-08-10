use std::fmt;

/// Every expected failure in the lending library.
///
/// This is the only file whose types are written for you. Nothing here should
/// ever be produced by a `panic!`, an `unwrap`, or an `expect` — these are
/// ordinary outcomes a caller is expected to handle.
#[derive(Debug, PartialEq, Eq)]
pub enum LibraryError {
    EmptyTitle,
    DuplicateItemId {
        id: u32,
    },
    DuplicateMemberId {
        id: u32,
    },
    ItemNotFound {
        id: u32,
    },
    MemberNotFound {
        id: u32,
    },
    ItemAlreadyOnLoan {
        id: u32,
        member_id: u32,
    },
    ItemNotOnLoan {
        id: u32,
    },
    ItemIsLost {
        id: u32,
    },
    BorrowLimitReached {
        member_id: u32,
        limit: usize,
    },
    InvalidReturnDay {
        day_borrowed: u32,
        day_returned: u32,
    },
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::EmptyTitle => write!(f, "title cannot be empty"),
            LibraryError::DuplicateItemId { id } => write!(f, "duplicate item id: {id}"),
            LibraryError::DuplicateMemberId { id } => write!(f, "duplicate member id: {id}"),
            LibraryError::ItemNotFound { id } => write!(f, "item {id} not found"),
            LibraryError::MemberNotFound { id } => write!(f, "member {id} not found"),
            LibraryError::ItemAlreadyOnLoan { id, member_id } => {
                write!(f, "item {id} is already on loan to member {member_id}")
            }
            LibraryError::ItemNotOnLoan { id } => write!(f, "item {id} is not on loan"),
            LibraryError::ItemIsLost { id } => write!(f, "item {id} is lost"),
            LibraryError::BorrowLimitReached { member_id, limit } => {
                write!(
                    f,
                    "member {member_id} reached maximum borrow limit of {limit} items"
                )
            }
            LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => {
                write!(
                    f,
                    "invalid return day {day_returned} (borrowed on day {day_borrowed})"
                )
            }
        }
    }
}

impl std::error::Error for LibraryError {}

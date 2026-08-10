use crate::catalogue::{Item, LoanStatus, LoanTerms};
use crate::error::LibraryError;
use crate::member::Member;

pub const MAX_ITEMS_PER_MEMBER: usize = 3;

/// Owns every item and every member.
///
/// The fields are private because the library is responsible for keeping an
/// item's `LoanStatus` and a member's borrowed-id list in agreement. Callers
/// reach the data through the borrowing lookups below.
#[derive(Debug, Default)]
pub struct Library {
    items: Vec<Item>,
    members: Vec<Member>,
}

impl Library {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_item(&mut self, item: Item) -> Result<(), LibraryError> {
        if item.title.trim().is_empty() {
            return Err(LibraryError::EmptyTitle);
        }
        if self.items.iter().any(|i| i.id == item.id) {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        }
        self.items.push(item);
        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        if self.members.iter().any(|m| m.id == member.id) {
            return Err(LibraryError::DuplicateMemberId { id: member.id });
        }
        self.members.push(member);
        Ok(())
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        self.members.iter().find(|m| m.id == id)
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        self.items.iter().filter(|i| i.author == author).collect()
    }

    pub fn available_items(&self) -> Vec<&Item> {
        self.items
            .iter()
            .filter(|i| matches!(i.status, crate::catalogue::LoanStatus::Available))
            .collect()
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        self.items.iter().max_by_key(|i| i.loan_days())
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        let item = self
            .items
            .iter()
            .find(|i| i.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        let member = self
            .members
            .iter()
            .find(|m| m.id == member_id)
            .ok_or(LibraryError::MemberNotFound { id: member_id })?;

        match item.status {
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::OnLoan {
                member_id: borrower_id,
                ..
            } => {
                return Err(LibraryError::ItemAlreadyOnLoan {
                    id: item_id,
                    member_id: borrower_id,
                });
            }
            LoanStatus::Available => {}
        }

        if member.borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }

        let item_mut = self.items.iter_mut().find(|i| i.id == item_id).unwrap();
        item_mut.status = LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };

        let member_mut = self.members.iter_mut().find(|m| m.id == member_id).unwrap();
        member_mut.borrowed_item_ids.push(item_id);

        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        let item = self
            .items
            .iter()
            .find(|i| i.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        let (borrower_id, day_borrowed) = match item.status {
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::Available => return Err(LibraryError::ItemNotOnLoan { id: item_id }),
            LoanStatus::OnLoan {
                member_id,
                day_borrowed,
            } => (member_id, day_borrowed),
        };

        if day < day_borrowed {
            return Err(LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned: day,
            });
        }

        let days_held = day - day_borrowed;
        let fee_cents = item.late_fee_cents(days_held);

        let item_mut = self.items.iter_mut().find(|i| i.id == item_id).unwrap();
        item_mut.status = LoanStatus::Available;

        if let Some(member_mut) = self.members.iter_mut().find(|m| m.id == borrower_id) {
            member_mut.borrowed_item_ids.retain(|&id| id != item_id);
        }

        Ok(fee_cents)
    }
}

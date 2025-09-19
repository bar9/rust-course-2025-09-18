// Chapter 12, Exercise 1: Library Management System

pub mod books {
    #[derive(Debug, Clone)]
    pub struct Book {
        pub title: String,
        pub author: String,
        isbn: String,  // Private field
        pub available: bool,
    }
    
    impl Book {
        pub fn new(title: String, author: String, isbn: String) -> Self {
            Book {
                title,
                author,
                isbn,
                available: true,
            }
        }
        
        pub fn isbn(&self) -> &str {
            &self.isbn
        }
        
        pub(super) fn mark_unavailable(&mut self) {
            self.available = false;
        }
        
        pub(super) fn mark_available(&mut self) {
            self.available = true;
        }
    }
}

pub mod members {
    #[derive(Debug)]
    pub struct Member {
        pub id: u32,
        pub name: String,
        email: String,  // Private field
        pub active: bool,
    }
    
    impl Member {
        pub fn new(id: u32, name: String, email: String) -> Self {
            Member {
                id,
                name,
                email,
                active: true,
            }
        }
        
        pub fn email(&self) -> &str {
            &self.email
        }
        
        pub fn deactivate(&mut self) {
            self.active = false;
        }
    }
}

pub mod loans {
    use super::books::Book;
    use super::members::Member;
    use std::collections::HashMap;
    
    pub struct Loan {
        pub member_id: u32,
        pub book_isbn: String,
        pub due_date: String,
    }
    
    pub struct LoanManager {
        pub(super) loans: HashMap<String, Loan>,  // ISBN -> Loan
    }
    
    impl LoanManager {
        pub fn new() -> Self {
            LoanManager {
                loans: HashMap::new(),
            }
        }
        
        pub fn checkout_book(
            &mut self, 
            book: &mut Book, 
            member: &Member, 
            due_date: String
        ) -> Result<(), String> {
            if !book.available {
                return Err("Book is not available".to_string());
            }
            
            if !member.active {
                return Err("Member is not active".to_string());
            }
            
            let loan = Loan {
                member_id: member.id,
                book_isbn: book.isbn().to_string(),
                due_date,
            };
            
            self.loans.insert(book.isbn().to_string(), loan);
            book.mark_unavailable();
            
            Ok(())
        }
        
        pub fn return_book(&mut self, book: &mut Book) -> Result<(), String> {
            if self.loans.remove(book.isbn()).is_some() {
                book.mark_available();
                Ok(())
            } else {
                Err("Book was not loaned out".to_string())
            }
        }
        
        pub fn get_member_loans(&self, member_id: u32) -> Vec<&Loan> {
            self.loans
                .values()
                .filter(|loan| loan.member_id == member_id)
                .collect()
        }
    }
}

// Public API module that re-exports necessary types
pub mod library {
    pub use super::books::Book;
    pub use super::members::Member;
    pub use super::loans::{LoanManager, Loan};
    
    pub struct Library {
        pub books: Vec<Book>,
        pub members: Vec<Member>,
        pub loan_manager: LoanManager,
    }
    
    impl Library {
        pub fn new() -> Self {
            Library {
                books: Vec::new(),
                members: Vec::new(),
                loan_manager: LoanManager::new(),
            }
        }
        
        pub fn add_book(&mut self, book: Book) {
            self.books.push(book);
        }
        
        pub fn add_member(&mut self, member: Member) {
            self.members.push(member);
        }
        
        pub fn find_book_mut(&mut self, isbn: &str) -> Option<&mut Book> {
            self.books.iter_mut().find(|b| b.isbn() == isbn)
        }
        
        pub fn find_member(&self, id: u32) -> Option<&Member> {
            self.members.iter().find(|m| m.id == id)
        }
        
        // Combined method to checkout a book, avoiding multiple mutable borrows
        pub fn checkout_book_to_member(&mut self, isbn: &str, member_id: u32, due_date: String) -> Result<(), String> {
            // Find member (immutable borrow)
            let member_exists = self.members.iter().any(|m| m.id == member_id && m.active);
            if !member_exists {
                return Err("Member not found or inactive".to_string());
            }
            
            // Find book and perform checkout
            if let Some(book) = self.books.iter_mut().find(|b| b.isbn() == isbn) {
                if !book.available {
                    return Err("Book is not available".to_string());
                }
                
                let loan = Loan {
                    member_id,
                    book_isbn: isbn.to_string(),
                    due_date,
                };
                
                self.loan_manager.loans.insert(isbn.to_string(), loan);
                book.mark_unavailable();
                Ok(())
            } else {
                Err("Book not found".to_string())
            }
        }
    }
}
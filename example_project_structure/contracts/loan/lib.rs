#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

/// This contract will represent the loan of a user
#[openbrush::contract]
pub mod loan {
    use ink_storage::traits::SpreadAllocate;
    use lending_project::traits::loan::*;
    use openbrush::{
        contracts::{
            ownable::*,
            psp34::extensions::metadata::*,
        },
        modifiers,
        storage::Mapping,
        traits::{
            Storage,
            String,
        },
    };

    /// Define the storage for PSP34 data, Metadata data and Ownable data
    #[ink(storage)]
    #[derive(SpreadAllocate, Storage)]
    pub struct LoanContract {
        #[storage_field]
        psp34: psp34::Data,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        metadata: metadata::Data,

        // Fields of current contract
        /// mapping from token id to `LoanInfo`
        loan_info: Mapping<Id, LoanInfo>,
        /// the id of last loan
        last_loan_id: Id,
    }

    // Implement PSP34 Trait for our NFT
    impl PSP34 for LoanContract {}

    // Implement Ownable Trait for our NFT
    impl Ownable for LoanContract {}

    // Implement PSP34Metadata Trait for our NFT
    impl PSP34Metadata for LoanContract {}

    impl Loan for LoanContract {
        #[modifiers(only_owner)]
        #[ink(message)]
        fn create_loan(&mut self, mut loan_info: LoanInfo) -> Result<(), PSP34Error> {
            let loan_id = self._get_next_loan_id_and_increase()?;
            if self.loan_info.get(&loan_id).is_some() {
                return Err(PSP34Error::Custom(String::from("This loan id already exists!")))
            }
            loan_info.liquidated = false;
            self.loan_info.insert(&loan_id, &loan_info);
            self._mint_to(loan_info.borrower, loan_id)
        }

        #[modifiers(only_owner)]
        #[ink(message)]
        fn delete_loan(&mut self, initiator: AccountId, loan_id: Id) -> Result<(), PSP34Error> {
            self.loan_info.remove(&loan_id);
            self._burn_from(initiator, loan_id)
        }

        #[modifiers(only_owner)]
        #[ink(message)]
        fn update_loan(
            &mut self,
            loan_id: Id,
            new_borrow_amount: Balance,
            new_timestamp: Timestamp,
            new_collateral_amount: Balance,
        ) -> Result<(), PSP34Error> {
            self._update_loan(loan_id, new_borrow_amount, new_timestamp, new_collateral_amount)
        }

        #[modifiers(only_owner)]
        #[ink(message)]
        fn liquidate_loan(&mut self, loan_id: Id) -> Result<(), PSP34Error> {
            self._liquidate_loan(loan_id)
        }

        #[ink(message)]
        fn get_loan_info(&self, loan_id: Id) -> Result<LoanInfo, PSP34Error> {
            let loan_info = self.loan_info.get(&loan_id);
            if loan_info.is_none() {
                return Err(PSP34Error::Custom(String::from("Loan does not exist")))
            }
            Ok(loan_info.unwrap())
        }
    }

    impl LoanContract {
        /// constructor with name and symbol
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut LoanContract| {
                instance.last_loan_id = Id::U128(1);
                instance._set_attribute(
                    Id::U8(1u8),
                    String::from("LoanContract NFT").into_bytes(),
                    String::from("L-NFT").into_bytes(),
                );
                instance._init_with_owner(instance.env().caller());
            })
        }

        /// Internal function to update data of a loan
        fn _update_loan(
            &mut self,
            loan_id: Id,
            new_borrow_amount: Balance,
            new_timestamp: Timestamp,
            new_collateral_amount: Balance,
        ) -> Result<(), PSP34Error> {
            let loan_info = self.loan_info.get(&loan_id);

            if loan_info.is_none() {
                return Err(PSP34Error::Custom(String::from("This loan does not exist!")))
            }

            let mut loan_info = loan_info.unwrap();
            loan_info.collateral_amount = new_collateral_amount;
            loan_info.borrow_amount = new_borrow_amount;
            loan_info.timestamp = new_timestamp;

            self.loan_info.insert(&loan_id, &loan_info);

            Ok(())
        }

        /// Internal function to set loan to liquidated
        fn _liquidate_loan(&mut self, loan_id: Id) -> Result<(), PSP34Error> {
            let loan_info = self.loan_info.get(&loan_id);

            if loan_info.is_none() {
                return Err(PSP34Error::Custom(String::from("This loan does not exist!")))
            }

            let mut loan_info = loan_info.unwrap();
            loan_info.liquidated = true;

            self.loan_info.insert(&loan_id, &loan_info);

            Ok(())
        }

        /// Internal function to return the id of a new loan and to increase it in the storage
        fn _get_next_loan_id_and_increase(&mut self) -> Result<Id, PSP34Error> {
            match &mut self.last_loan_id {
                Id::U128(id) => {
                    let result = Id::U128(id.clone());
                    *id += 1;
                    Ok(result)
                }
                _ => Err(PSP34Error::Custom(String::from("Not expected Id!"))),
            }
        }
    }
}

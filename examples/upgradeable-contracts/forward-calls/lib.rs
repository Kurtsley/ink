//! This example demonstrates how the Proxy/Forward pattern can be
//! implemented in ink!.
//!
//! What the contract does is:
//!
//!   * Any call to this contract that does not match a selector
//!     of it is forwarded to a specified address.
//!   * The instantiator of the contract can modify this specified
//!     `forward_to` address at any point.
//!
//! Using this pattern it is possible to implement upgradeable contracts.
//!
//! Note though that the contract to which calls are forwarded still
//! contains it's own state.

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
pub mod proxy {
    use ink::env::call::Call;

    /// A simple proxy contract.
    #[ink(storage)]
    pub struct Proxy {
        /// The `AccountId` of a contract where any call that does not match a
        /// selector of this contract is forwarded to.
        forward_to: AccountId,
        /// The `AccountId` of a privileged account that can update the
        /// forwarding address. This address is set to the account that
        /// instantiated this contract.
        admin: AccountId,
    }

    impl Proxy {
        /// Instantiate this contract with an address of the `logic` contract.
        ///
        /// Sets the privileged account to the caller. Only this account may
        /// later changed the `forward_to` address.
        #[ink(constructor)]
        pub fn new(forward_to: AccountId) -> Self {
            Self {
                admin: Self::env().caller(),
                forward_to,
            }
        }

        /// Changes the `AccountId` of the contract where any call that does
        /// not match a selector of this contract is forwarded to.
        #[ink(message)]
        pub fn change_forward_address(&mut self, new_address: AccountId) {
            assert_eq!(
                self.env().caller(),
                self.admin,
                "caller {:?} does not have sufficient permissions, only {:?} does",
                self.env().caller(),
                self.admin,
            );
            self.forward_to = new_address;
        }

        /// Fallback message for a contract call that doesn't match any
        /// of the other message selectors.
        ///
        /// # Note:
        ///
        /// - We allow payable messages here and would forward any optionally supplied
        ///   value as well.
        /// - If the self receiver were `forward(&mut self)` here, this would not
        ///   have any effect whatsoever on the contract we forward to.
        #[ink(message, payable, selector = _)]
        pub fn forward(&self) -> u32 {
            ink::env::call::build_call::<ink::env::DefaultEnvironment>()
                .call_type(
                    Call::new()
                        .callee(self.forward_to)
                        .transferred_value(self.env().transferred_value())
                        .gas_limit(0),
                )
                .call_flags(
                    ink::env::CallFlags::default()
                        .set_forward_input(true)
                        .set_tail_call(true),
                )
                .try_invoke()
                .unwrap_or_else(|env_err| {
                    panic!(
                        "cross-contract call to {:?} failed due to {:?}",
                        self.forward_to, env_err
                    )
                })
                .unwrap_or_else(|lang_err| {
                    panic!(
                        "cross-contract call to {:?} failed due to {:?}",
                        self.forward_to, lang_err
                    )
                });
            unreachable!(
                "the forwarded call will never return since `tail_call` was set"
            );
        }
    }
}

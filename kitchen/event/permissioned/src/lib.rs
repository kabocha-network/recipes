#![cfg_attr(not(feature = "std"), no_std)]

/// Permissioned Function with Generic Event
/// a permissioned funtion which can only be called by the "owner". An event is emitted 
/// when the function is successfully executed.

use support::{ensure, decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};
use system::ensure_signed;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as PGeneric {
		Owner get(owner): T::AccountId;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		OwnershipTransferred(AccountId, AccountId),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		fn init_ownership(origin) -> Result {
            ensure!(!<Owner<T>>::exists(), "Owner already exists");
            let sender = ensure_signed(origin)?;
            <Owner<T>>::put(&sender);
            Self::deposit_event(RawEvent::OwnershipTransferred(sender.clone(), sender));
            Ok(())
        }

        fn transfer_ownership(origin, new_owner: T::AccountId) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(sender == Self::owner(), "This function can only be called by the owner");
            <Owner<T>>::put(&new_owner);
            Self::deposit_event(RawEvent::OwnershipTransferred(sender, new_owner));
            Ok(())
        }
	}
}
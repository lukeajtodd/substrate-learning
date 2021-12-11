#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use frame_support::pallet_prelude::*;
  use frame_system::pallet_prelude::*;
  use sp_std::vec::Vec; // Step 3.1 will include this in `Cargo.toml`

  #[pallet::config]
  pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
  }

  // Handles successful calls to the pallet
  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    ClaimCreated(T::AccountId, Vec<u8>),
    ClaimRevoked(T::AccountId, Vec<u8>),
  }

  #[pallet::error]
  pub enum Error<T> {
    ProofAlreadyClaimed,
    // The proof doesn't exist so it cannot be claimed
    NoSuchProof,
    // Can't revoke the proof as it belongs to someone else
    NotProofOwner
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::storage]
  // Mapping of one rto proof when the block is recorded on the chain
  pub(super) type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId, T::BlockNumber), ValueQuery>;

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    #[pallet::weight(1_000)]
    pub fn create_claim(
      origin: OriginFor<T>,
      proof: Vec<u8>
    ) -> DispatchResult {
      // Check that the extrinsic was signed and get the signer. Aan error will return if it is not signed.
      let sender = ensure_signed(origin)?;

      // Using our errors enum, verify that the proof hasn't already been claimed
      ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);

      // Get the block number from the FRAME system pallet
      let current_block = <frame_system::Pallet<T>>::block_number();

      // Store the proof with the sender info and block number (as detailed in storage)
      Proofs::<T>::insert(&proof, (&sender, current_block));

      // Emit an event that the claim has been created
      Self::deposit_event(Event::ClaimCreated(sender, proof));

      Ok(())
    }

    #[pallet::weight(10_000)]
    pub fn revoke_claim(
        origin: OriginFor<T>,
        proof: Vec<u8>,
    ) -> DispatchResult {
        // Check that the extrinsic was signed and get the signer.
        // This function will return an error if the extrinsic is not signed.
        // https://docs.substrate.io/v3/runtime/origins
        let sender = ensure_signed(origin)?;

        // Verify that the specified proof has been claimed.
        ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

        // Get owner of the claim.
        let (owner, _) = Proofs::<T>::get(&proof);

        // Verify that sender of the current call is the claim owner.
        ensure!(sender == owner, Error::<T>::NotProofOwner);

        // Remove claim from storage.
        Proofs::<T>::remove(&proof);

        // Emit an event that the claim was erased.
        Self::deposit_event(Event::ClaimRevoked(sender, proof));

        Ok(())
    }
  }
}
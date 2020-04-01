#![cfg_attr(not(feature = "std"), no_std)]

mod functions;
mod tests;

use sp_std::prelude::*;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, traits::Get};
use sp_runtime::{RuntimeDebug, traits::Zero};
use system::ensure_signed;
use pallet_timestamp;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct WhoAndWhen<T: Trait> {
  account: T::AccountId,
  block: T::BlockNumber,
  time: T::Moment,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceOwners<T: Trait> {
  pub created: WhoAndWhen<T>,
  pub space_id: SpaceId,
  pub owners: Vec<T::AccountId>,
  pub threshold: u16,

  pub changes_count: u64,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Transaction<T: Trait> {
  pub created: WhoAndWhen<T>,
  pub id: TransactionId,
  pub space_id: SpaceId,
  pub add_owners: Vec<T::AccountId>,
  pub remove_owners: Vec<T::AccountId>,
  pub new_threshold: Option<u16>,
  pub notes: Vec<u8>,
  pub confirmed_by: Vec<T::AccountId>,
  pub expires_at: T::BlockNumber,
}

type SpaceId = u64;
type TransactionId = u64;

pub trait SpaceOwnersShared<AccountId> {
  fn is_account_a_space_owner(possible_owner: &AccountId, space_id: SpaceId) -> bool;
}

impl<T: Trait> SpaceOwnersShared<T::AccountId> for Module<T> {
  fn is_account_a_space_owner(possible_owner: &T::AccountId, space_id: SpaceId) -> bool { 
    Self::space_ids_owned_by_account_id(possible_owner)
      .contains(&space_id)
  }
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_timestamp::Trait {
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

  /// Minimum space owners allowed.
  type MinSpaceOwners: Get<u16>;

  /// Maximum space owners allowed.
  type MaxSpaceOwners: Get<u16>;

  /// Maximum transaction notes length.
  type MaxTxNotesLength: Get<u16>;

  /// Expiration time for change proposal.
  type BlocksToLive: Get<Self::BlockNumber>;

  /// Period in blocks to initialize cleaning of pending txs that are outdated.
  type CleanExpiredTxsPeriod: Get<Self::BlockNumber>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// Space owners was not found by id
    SpaceOwnersNotFound,
    /// Transaction was not found in a space owners
    TxNotFound,
    /// Space owners already exist on this space
    SpaceOwnersAlreadyExist,

    /// There can not be less owners than allowed
    NotEnoughOwners,
    /// There can not be more owners than allowed
    TooManyOwners,
    /// Account is not a space owner
    NotASpaceOwner,

    /// The threshold can not be less than 1
    ZeroThershold,
    /// The required confirmation count can not be greater than owners count"
    TooBigThreshold,
    /// Transaction notes are too long
    TxNotesOversize,
    /// No space owners will left in result of tx
    NoSpaceOwnersLeft,
    /// No updates proposed in change proposal
    NoUpdatesProposed,
    /// No fields update in result of change proposal
    NoFieldsUpdatedOnProposal,

    /// Account has already confirmed this transaction
    TxAlreadyConfirmed,
    /// There are not enough confirmations on a transaction
    NotEnoughConfirms,
    /// Transaction is already executed
    TxAlreadyExecuted,
    /// Transaction is not tied to an owed wallet
    TxNotRelatedToSpace,
    /// Pending tx already exists
    PendingTxAlreadyExists,
    /// Pending tx doesn't exist
    PendingTxDoesNotExist,

    /// Account is not a proposal creator
    NotAProposalCreator,

    /// Overflow in Wallet executed tx counter when executing tx
    OverflowExecutingTx,
    /// Underflow in Wallet pending tx counter when executing tx
    UnderflowExecutingTx,
  }
}

// This pallet's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as TemplateModule {
    SpaceOwnersBySpaceById get(space_owners_by_space_id): map SpaceId => Option<SpaceOwners<T>>;
    SpaceIdsOwnedByAccountId get(space_ids_owned_by_account_id): map T::AccountId => BTreeSet<SpaceId> = BTreeSet::new();

    NextTxId get(next_tx_id): TransactionId = 1;
    TxById get(tx_by_id): map TransactionId => Option<Transaction<T>>;
    PendingTxIdBySpaceId get(pending_tx_id_by_space_id): map SpaceId => Option<TransactionId>;
    PendingTxIds get(pending_tx_ids): BTreeSet<TransactionId> = BTreeSet::new();
    ExecutedTxIdsBySpaceId get(executed_tx_ids_by_space_id): map SpaceId => Vec<TransactionId>;
  }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    /// Minimum space owners allowed.
    const MinSpaceOwners: u16 = T::MinSpaceOwners::get();

    /// Maximum space owners allowed.
    const MaxSpaceOwners: u16 = T::MaxSpaceOwners::get();

    /// Maximum transaction notes length.
    const MaxTxNotesLength: u16 = T::MaxTxNotesLength::get();

    /// Period for which change proposal is active.
    const BlocksToLive: T::BlockNumber = T::BlocksToLive::get();

    /// Period in blocks to initialize cleaning of pending txs that are outdated.
    const CleanExpiredTxsPeriod: T::BlockNumber = T::CleanExpiredTxsPeriod::get();

    // Initializing events
    fn deposit_event() = default;

    fn on_finalize(n: T::BlockNumber) {
      Self::clean_pending_txs(n);
    }

    pub fn create_space_owners(
      origin,
      space_id: SpaceId,
      owners: Vec<T::AccountId>,
      threshold: u16
    ) {
      let who = ensure_signed(origin)?;

      ensure!(Self::space_owners_by_space_id(space_id).is_none(), Error::<T>::SpaceOwnersAlreadyExist);

      let mut owners_map: BTreeMap<T::AccountId, bool> = BTreeMap::new();
      let mut unique_owners: Vec<T::AccountId> = Vec::new();

      for owner in owners.iter() {
        if !owners_map.contains_key(&owner) {
          owners_map.insert(owner.clone(), true);
          unique_owners.push(owner.clone());
        }
      }

      let owners_count = unique_owners.len() as u16;
      ensure!(owners_count >= T::MinSpaceOwners::get(), Error::<T>::NotEnoughOwners);
      ensure!(owners_count <= T::MaxSpaceOwners::get(), Error::<T>::TooManyOwners);

      ensure!(threshold <= owners_count, Error::<T>::TooBigThreshold);
      ensure!(threshold > 0, Error::<T>::ZeroThershold);

      let new_space_owners = SpaceOwners {
        created: Self::new_whoandwhen(who.clone()),
        space_id: space_id.clone(),
        owners: unique_owners.clone(),
        threshold,
        changes_count: 0
      };

      <SpaceOwnersBySpaceById<T>>::insert(space_id, new_space_owners);

      for owner in unique_owners.iter() {
        <SpaceIdsOwnedByAccountId<T>>::mutate(owner.clone(), |ids| ids.insert(space_id));
      }

      Self::deposit_event(RawEvent::SpaceOwnersCreated(who, space_id));
    }

    pub fn propose_change(
      origin,
      space_id: SpaceId,
      add_owners: Vec<T::AccountId>,
      remove_owners: Vec<T::AccountId>,
      new_threshold: Option<u16>,
      notes: Vec<u8>
    ) {
      let who = ensure_signed(origin)?;

      let has_updates =
        !add_owners.is_empty() ||
        !remove_owners.is_empty() ||
        new_threshold.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesProposed);
      ensure!(notes.len() <= T::MaxTxNotesLength::get() as usize, Error::<T>::TxNotesOversize);

      let space_owners = Self::space_owners_by_space_id(space_id.clone()).ok_or(Error::<T>::SpaceOwnersNotFound)?;
      ensure!(Self::pending_tx_id_by_space_id(space_id).is_none(), Error::<T>::PendingTxAlreadyExists);

      let is_space_owner = space_owners.owners.iter().any(|owner| *owner == who.clone());
      ensure!(is_space_owner, Error::<T>::NotASpaceOwner);

      let mut fields_updated : u16 = 0;

      let result_owners = Self::transform_new_owners_to_vec(space_owners.owners.clone(), add_owners.clone(), remove_owners.clone());
      ensure!(!result_owners.is_empty(), Error::<T>::NoSpaceOwnersLeft);
      if result_owners != space_owners.owners {
        fields_updated += 1;
      }

      if let Some(threshold) = new_threshold {
        if space_owners.threshold != threshold {
          ensure!(threshold as usize <= result_owners.len(), Error::<T>::TooBigThreshold);
          ensure!(threshold > 0, Error::<T>::ZeroThershold);
          fields_updated += 1;
        }
      }

      let tx_id = Self::next_tx_id();
      let mut new_tx = Transaction {
        created: Self::new_whoandwhen(who.clone()),
        id: tx_id,
        space_id,
        add_owners: add_owners,
        remove_owners: remove_owners,
        new_threshold: new_threshold,
        notes,
        confirmed_by: Vec::new(),
        expires_at: <system::Module<T>>::block_number() + T::BlocksToLive::get()
      };

      if fields_updated > 0 {
        new_tx.confirmed_by.push(who.clone());
        <TxById<T>>::insert(tx_id, new_tx);
        PendingTxIdBySpaceId::insert(space_id.clone(), tx_id);
        PendingTxIds::mutate(|set| set.insert(tx_id));
        NextTxId::mutate(|n| { *n += 1; });

        Self::deposit_event(RawEvent::ChangeProposed(who, space_id, tx_id));
      } else {
        Err(Error::<T>::NoFieldsUpdatedOnProposal)?
      }
    }

    pub fn confirm_change(
      origin,
      space_id: SpaceId,
      tx_id: TransactionId
    ) {
      let who = ensure_signed(origin)?;

      let space_owners = Self::space_owners_by_space_id(space_id.clone()).ok_or(Error::<T>::SpaceOwnersNotFound)?;

      let is_space_owner = space_owners.owners.iter().any(|owner| *owner == who.clone());
      ensure!(is_space_owner, Error::<T>::NotASpaceOwner);

      let mut tx = Self::tx_by_id(tx_id).ok_or(Error::<T>::TxNotFound)?;

      let pending_tx_id = Self::pending_tx_id_by_space_id(space_id.clone()).ok_or(Error::<T>::PendingTxDoesNotExist)?;
      ensure!(pending_tx_id == tx_id, Error::<T>::TxNotRelatedToSpace);

      // Check whether sender confirmed tx or not
      ensure!(!tx.confirmed_by.iter().any(|account| *account == who.clone()), Error::<T>::TxAlreadyConfirmed);

      tx.confirmed_by.push(who.clone());

      if tx.confirmed_by.len() == space_owners.threshold as usize {
        Self::update_space_owners(who.clone(), space_owners.clone(), tx.clone())?;
      } else {
        <TxById<T>>::insert(tx_id, tx);
      }

      Self::deposit_event(RawEvent::ChangeConfirmed(who, space_id, tx_id));
    }

    pub fn cancel_proposal(
      origin,
      space_id: SpaceId,
      tx_id: TransactionId
    ) {
      let who = ensure_signed(origin)?;

      let space_owners = Self::space_owners_by_space_id(space_id.clone()).ok_or(Error::<T>::SpaceOwnersNotFound)?;

      let is_space_owner = space_owners.owners.iter().any(|owner| *owner == who.clone());
      ensure!(is_space_owner, Error::<T>::NotASpaceOwner);

      let pending_tx_id = Self::pending_tx_id_by_space_id(space_id.clone()).ok_or(Error::<T>::PendingTxDoesNotExist)?;
      ensure!(pending_tx_id == tx_id, Error::<T>::TxNotRelatedToSpace);

      let tx = Self::tx_by_id(tx_id).ok_or(Error::<T>::TxNotFound)?;
      ensure!(tx.created.account == who, Error::<T>::NotAProposalCreator);

      <TxById<T>>::remove(tx_id);
      PendingTxIdBySpaceId::remove(space_id);
      PendingTxIds::mutate(|set| set.remove(&tx_id));

      Self::deposit_event(RawEvent::ProposalCanceled(who, space_id));
    }
  }
}

decl_event!(
  pub enum Event<T> where
    <T as system::Trait>::AccountId,
   {
    SpaceOwnersCreated(AccountId, SpaceId),
    ChangeProposed(AccountId, SpaceId, TransactionId),
    ProposalCanceled(AccountId, SpaceId),
    ChangeConfirmed(AccountId, SpaceId, TransactionId),
    SpaceOwnersUpdated(AccountId, SpaceId, TransactionId),
  }
);

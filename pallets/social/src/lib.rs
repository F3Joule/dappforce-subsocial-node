#![cfg_attr(not(feature = "std"), no_std)]

/// For more guidance on FRAME pallets, see the example.
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

pub mod defaults;
pub mod functions;
mod tests;

use defaults::*;
use sp_std::prelude::*;
use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure};
use sp_runtime::RuntimeDebug;
use system::ensure_signed;
use pallet_timestamp;
use pallet_utils::WhoAndWhen;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Blog<T: Trait> {
  pub id: BlogId,
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,

  // Can be updated by the owner:
  pub writers: Vec<T::AccountId>,
  pub slug: Vec<u8>,
  pub ipfs_hash: Vec<u8>,

  pub posts_count: u16,
  pub followers_count: u32,

  pub edit_history: Vec<BlogHistoryRecord<T>>,

  pub score: i32,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct BlogUpdate<AccountId> {
  pub writers: Option<Vec<AccountId>>,
  pub slug: Option<Vec<u8>>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct BlogHistoryRecord<T: Trait> {
  pub edited: WhoAndWhen<T>,
  pub old_data: BlogUpdate<T::AccountId>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Post<T: Trait> {
  pub id: PostId,
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,

  pub blog_id: Option<BlogId>,
  pub extension: PostExtension,

  pub ipfs_hash: Vec<u8>,
  pub edit_history: Vec<PostHistoryRecord<T>>,

  pub total_replies_count: u16,
  pub shares_count: u16,
  pub upvotes_count: u16,
  pub downvotes_count: u16,

  pub score: i32,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostUpdate {
  pub blog_id: Option<BlogId>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostHistoryRecord<T: Trait> {
  pub edited: WhoAndWhen<T>,
  pub old_data: PostUpdate,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum PostExtension {
  RegularPost,
  Comment(CommentExt),
  SharedPost(PostId),
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub struct CommentExt {
  parent_id: Option<PostId>,
  root_post_id: PostId,
}

impl Default for PostExtension {
  fn default() -> Self {
    PostExtension::RegularPost
  }
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum ReactionKind {
  Upvote,
  Downvote,
}

impl Default for ReactionKind {
  fn default() -> Self {
    ReactionKind::Upvote
  }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Reaction<T: Trait> {
  pub id: ReactionId,
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,
  pub kind: ReactionKind,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SocialAccount<T: Trait> {
  pub followers_count: u32,
  pub following_accounts_count: u16,
  pub following_blogs_count: u16,
  pub reputation: u32,
  pub profile: Option<Profile<T>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Profile<T: Trait> {
  pub created: WhoAndWhen<T>,
  pub updated: Option<WhoAndWhen<T>>,

  pub username: Vec<u8>,
  pub ipfs_hash: Vec<u8>,

  pub edit_history: Vec<ProfileHistoryRecord<T>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileUpdate {
  pub username: Option<Vec<u8>>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileHistoryRecord<T: Trait> {
  pub edited: WhoAndWhen<T>,
  pub old_data: ProfileUpdate,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum ScoringAction {
  UpvotePost,
  DownvotePost,
  SharePost,
  CreateComment,
  UpvoteComment,
  DownvoteComment,
  ShareComment,
  FollowBlog,
  FollowAccount,
}

impl Default for ScoringAction {
  fn default() -> Self {
    ScoringAction::FollowAccount
  }
}

pub type BlogId = u64;
pub type PostId = u64;
pub type ReactionId = u64;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_timestamp::Trait {
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// Blog was not found by id
    BlogNotFound,
    /// Blog slug is too short
    SlugIsTooShort,
    /// Blog slug is too long
    SlugIsTooLong,
    /// Blog slug is not unique
    SlugIsNotUnique,
    /// Nothing to update in blog
    NoUpdatesInBlog,
    /// Only blog owner can manage their blog
    NotABlogOwner,

    /// Post was not found by id
    PostNotFound,
    /// Nothing to update in post
    NoUpdatesInPost,
    /// Only post author can manage their blog
    NotAnAuthor,
    /// Overflow caused adding post on blog
    OverflowAddingPostOnBlog,
    /// Cannot create post not defining blog_id
    BlogIdIsUndefined,

    /// Unknown parent comment id
    UnknownParentComment,
    /// Post by root_post_id is not of RegularPost extension
    NoPostByRootPostId,
    /// New comment IPFS-hash is the same as old one
    CommentIPFSHashNotDiffer,
    /// Overflow adding comment on post
    OverflowAddingCommentOnPost,
    /// Cannot update blog id on Comment
    CannotUpdateBlogIdOnComment,

    /// Reaction was not found by id
    ReactionNotFound,
    /// Account has already reacted to this post/comment
    AccountAlreadyReacted,
    /// Account has not reacted to this post/comment yet
    AccountNotYetReacted,
    /// There is no post/comment reaction by account that could be deleted
    ReactionByAccountNotFound,
    /// Overflow caused upvoting post/comment
    OverflowUpvoting,
    /// Overflow caused downvoting post/comment
    OverflowDownvoting,
    /// Only reaction owner can update their reaction
    NotAReactionOwner,
    /// New reaction kind is the same as old one
    NewReactionKindNotDiffer,

    /// Account is already following this blog
    AccountIsFollowingBlog,
    /// Account is not following this blog
    AccountIsNotFollowingBlog,
    /// Account can not follow itself
    AccountCannotFollowItself,
    /// Account can not unfollow itself
    AccountCannotUnfollowItself,
    /// Account is already followed
    AccountIsAlreadyFollowed,
    /// Account is not followed by current follower
    AccountIsNotFollowed,
    /// Underflow unfollowing blog
    UnderflowUnfollowingBlog,
    /// Overflow caused following blog
    OverflowFollowingBlog,
    /// Overflow caused following account
    OverflowFollowingAccount,
    /// Underflow caused unfollowing account
    UnderflowUnfollowingAccount,

    /// Social account was not found by id
    SocialAccountNotFound,
    /// Follower social account was not found by id
    FollowerAccountNotFound,
    /// Social account that is being followed was not found by id
    FollowedAccountNotFound,

    /// IPFS-hash is not correct
    IpfsIsIncorrect,

    /// Out of bounds updating blog score
    OutOfBoundsUpdatingBlogScore,
    /// Out of bounds reverting blog score
    OutOfBoundsRevertingBlogScore,
    /// Out of bounds updating post score
    OutOfBoundsUpdatingPostScore,
    /// Out of bounds reverting post score
    OutOfBoundsRevertingPostScore,
    /// Post extension is a comment
    PostIsAComment,
    /// Out of bounds updating comment score
    OutOfBoundsUpdatingCommentScore,
    /// Out of bounds reverting comment score
    OutOfBoundsRevertingCommentScore,
    /// Post extension is not a comment
    PostIsNotAComment,
    /// Out of bounds updating social account reputation
    OutOfBoundsUpdatingAccountReputation,
    /// Scored account reputation difference by account and action not found
    ReputationDiffNotFound,

    /// Original post not found when sharing
    OriginalPostNotFound,
    /// Overflow caused on total shares counter when sharing post
    OverflowTotalSharesSharingPost,
    /// Overflow caused on shares by account counter when sharing post
    OverflowPostSharesSharingPost,
    /// Cannot share post that is not a regular post
    CannotShareSharedPost,
    /// Overflow caused on total shares counter when sharing comment
    OverflowTotalSharesSharingComment,
    /// Overflow caused on shares by account counter when sharing comment
    OverflowCommentSharesByAccount,

    /// Profile for this account already exists
    ProfileAlreadyExists,
    /// Nothing to update in a profile
    NoUpdatesInProfile,
    /// Account has no profile yet
    ProfileDoesNotExist,
    /// Profile username is busy
    UsernameIsBusy,
    /// Username is too short
    UsernameIsTooShort,
    /// Username is too long
    UsernameIsTooLong,
    /// Username is not alphanumeric
    UsernameIsNotAlphanumeric,
  }
}

// This pallet's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as TemplateModule {
    pub SlugMinLen get(slug_min_len): u32 = DEFAULT_SLUG_MIN_LEN;
    pub SlugMaxLen get(slug_max_len): u32 = DEFAULT_SLUG_MAX_LEN;

    pub IpfsHashLen get(ipfs_hash_len): u32 = DEFAULT_IPFS_HASH_LEN;

    pub UsernameMinLen get(username_min_len): u32 = DEFAULT_USERNAME_MIN_LEN;
    pub UsernameMaxLen get(username_max_len): u32 = DEFAULT_USERNAME_MAX_LEN;

    pub BlogMaxLen get(blog_max_len): u32 = DEFAULT_BLOG_MAX_LEN;
    pub PostMaxLen get(post_max_len): u32 = DEFAULT_POST_MAX_LEN;
    pub CommentMaxLen get(comment_max_len): u32 = DEFAULT_COMMENT_MAX_LEN;

    pub UpvotePostActionWeight get (upvote_post_action_weight): i16 = DEFAULT_UPVOTE_POST_ACTION_WEIGHT;
    pub DownvotePostActionWeight get (downvote_post_action_weight): i16 = DEFAULT_DOWNVOTE_POST_ACTION_WEIGHT;
    pub SharePostActionWeight get (share_post_action_weight): i16 = DEFAULT_SHARE_POST_ACTION_WEIGHT;
    pub CreateCommentActionWeight get (create_comment_action_weight): i16 = DEFAULT_CREATE_COMMENT_ACTION_WEIGHT;
    pub UpvoteCommentActionWeight get (upvote_comment_action_weight): i16 = DEFAULT_UPVOTE_COMMENT_ACTION_WEIGHT;
    pub DownvoteCommentActionWeight get (downvote_comment_action_weight): i16 = DEFAULT_DOWNVOTE_COMMENT_ACTION_WEIGHT;
    pub ShareCommentActionWeight get (share_comment_action_weight): i16 = DEFAULT_SHARE_COMMENT_ACTION_WEIGHT;
    pub FollowBlogActionWeight get (follow_blog_action_weight): i16 = DEFAULT_FOLLOW_BLOG_ACTION_WEIGHT;
    pub FollowAccountActionWeight get (follow_account_action_weight): i16 = DEFAULT_FOLLOW_ACCOUNT_ACTION_WEIGHT;

    pub BlogById get(blog_by_id): map BlogId => Option<Blog<T>>;
    pub PostById get(post_by_id): map PostId => Option<Post<T>>;
    pub ReactionById get(reaction_by_id): map ReactionId => Option<Reaction<T>>;
    pub SocialAccountById get(social_account_by_id): map T::AccountId => Option<SocialAccount<T>>;

    pub BlogIdsByOwner get(blog_ids_by_owner): map T::AccountId => Vec<BlogId>;
    pub PostIdsByBlogId get(post_ids_by_blog_id): map BlogId => Vec<PostId>;
    pub CommentIdsByPostId get(comment_ids_by_post_id): map PostId => Vec<PostId>;

    pub ReactionIdsByPostId get(reaction_ids_by_post_id): map PostId => Vec<ReactionId>;
    pub ReactionIdsByCommentId get(reaction_ids_by_comment_id): map PostId => Vec<ReactionId>;
    pub PostReactionIdByAccount get(post_reaction_id_by_account): map (T::AccountId, PostId) => ReactionId;
    pub CommentReactionIdByAccount get(comment_reaction_id_by_account): map (T::AccountId, PostId) => ReactionId;

    pub BlogIdBySlug get(blog_id_by_slug): map Vec<u8> => Option<BlogId>;

    pub BlogsFollowedByAccount get(blogs_followed_by_account): map T::AccountId => Vec<BlogId>;
    pub BlogFollowers get(blog_followers): map BlogId => Vec<T::AccountId>;
    pub BlogFollowedByAccount get(blog_followed_by_account): map (T::AccountId, BlogId) => bool;

    pub AccountFollowedByAccount get(account_followed_by_account): map (T::AccountId, T::AccountId) => bool;
    pub AccountsFollowedByAccount get(accounts_followed_by_account): map T::AccountId => Vec<T::AccountId>;
    pub AccountFollowers get(account_followers): map T::AccountId => Vec<T::AccountId>;

    pub NextBlogId get(next_blog_id): BlogId = 1;
    pub NextPostId get(next_post_id): PostId = 1;
    pub NextReactionId get(next_reaction_id): ReactionId = 1;

    pub AccountReputationDiffByAccount get(account_reputation_diff_by_account): map (T::AccountId, T::AccountId, ScoringAction) => Option<i16>; // TODO shorten name (?refactor)
    pub PostScoreByAccount get(post_score_by_account): map (T::AccountId, PostId, ScoringAction) => Option<i16>;
    pub CommentScoreByAccount get(comment_score_by_account): map (T::AccountId, PostId, ScoringAction) => Option<i16>;

    pub PostSharesByAccount get(post_shares_by_account): map (T::AccountId, PostId) => u16;
    pub SharedPostIdsByOriginalPostId get(shared_post_ids_by_original_post_id): map PostId => Vec<PostId>;

    pub CommentSharesByAccount get(comment_shares_by_account): map (T::AccountId, PostId) => u16;
    pub SharedPostIdsByOriginalCommentId get(shared_post_ids_by_original_comment_id): map PostId => Vec<PostId>;

    pub AccountByProfileUsername get(account_by_profile_username): map Vec<u8> => Option<T::AccountId>;
  }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    // Initializing events
    // this is needed only if you are using events in your pallet
    fn deposit_event() = default;

    pub fn create_blog(origin, slug: Vec<u8>, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      ensure!(slug.len() >= Self::slug_min_len() as usize, Error::<T>::SlugIsTooShort);
      ensure!(slug.len() <= Self::slug_max_len() as usize, Error::<T>::SlugIsTooLong);
      ensure!(!BlogIdBySlug::exists(slug.clone()), Error::<T>::SlugIsNotUnique);
      Self::is_ipfs_hash_valid(ipfs_hash.clone())?;

      let blog_id = Self::next_blog_id();
      let ref mut new_blog: Blog<T> = Blog {
        id: blog_id,
        created: WhoAndWhen::<T>::new(owner.clone()),
        updated: None,
        writers: vec![],
        slug: slug.clone(),
        ipfs_hash,
        posts_count: 0,
        followers_count: 0,
        edit_history: vec![],
        score: 0
      };

      // Blog creator automatically follows their blog:
      Self::add_blog_follower_and_insert_blog(owner.clone(), new_blog, true)?;

      <BlogIdsByOwner<T>>::mutate(owner.clone(), |ids| ids.push(blog_id));
      BlogIdBySlug::insert(slug, blog_id);
      NextBlogId::mutate(|n| { *n += 1; });
    }

    pub fn update_blog(origin, blog_id: BlogId, update: BlogUpdate<T::AccountId>) {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.writers.is_some() ||
        update.slug.is_some() ||
        update.ipfs_hash.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesInBlog);

      let mut blog = Self::blog_by_id(blog_id).ok_or(Error::<T>::BlogNotFound)?;

      // TODO ensure: blog writers also should be able to edit this blog:
      ensure!(owner == blog.created.account, Error::<T>::NotABlogOwner);

      let mut fields_updated = 0;
      let mut new_history_record = BlogHistoryRecord {
        edited: WhoAndWhen::<T>::new(owner.clone()),
        old_data: BlogUpdate {writers: None, slug: None, ipfs_hash: None}
      };

      if let Some(writers) = update.writers {
        if writers != blog.writers {
          // TODO validate writers.
          // TODO update BlogIdsByWriter: insert new, delete removed, update only changed writers.
          new_history_record.old_data.writers = Some(blog.writers);
          blog.writers = writers;
          fields_updated += 1;
        }
      }

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != blog.ipfs_hash {
          Self::is_ipfs_hash_valid(ipfs_hash.clone())?;
          new_history_record.old_data.ipfs_hash = Some(blog.ipfs_hash);
          blog.ipfs_hash = ipfs_hash;
          fields_updated += 1;
        }
      }

      if let Some(slug) = update.slug {
        if slug != blog.slug {
          let slug_len = slug.len();
          ensure!(slug_len >= Self::slug_min_len() as usize, Error::<T>::SlugIsTooShort);
          ensure!(slug_len <= Self::slug_max_len() as usize, Error::<T>::SlugIsTooLong);
          ensure!(!BlogIdBySlug::exists(slug.clone()), Error::<T>::SlugIsNotUnique);

          BlogIdBySlug::remove(blog.slug.clone());
          BlogIdBySlug::insert(slug.clone(), blog_id);
          new_history_record.old_data.slug = Some(blog.slug);
          blog.slug = slug;
          fields_updated += 1;
        }
      }

      // Update this blog only if at least one field should be updated:
      if fields_updated > 0 {
        blog.updated = Some(WhoAndWhen::<T>::new(owner.clone()));
        blog.edit_history.push(new_history_record);
        <BlogById<T>>::insert(blog_id, blog);
        Self::deposit_event(RawEvent::BlogUpdated(owner.clone(), blog_id));
      }
    }

    pub fn follow_blog(origin, blog_id: BlogId) {
      let follower = ensure_signed(origin)?;

      let ref mut blog = Self::blog_by_id(blog_id).ok_or(Error::<T>::BlogNotFound)?;
      ensure!(!Self::blog_followed_by_account((follower.clone(), blog_id)), Error::<T>::AccountIsFollowingBlog);

      Self::add_blog_follower_and_insert_blog(follower.clone(), blog, false)?;
    }

    pub fn unfollow_blog(origin, blog_id: BlogId) {
      let follower = ensure_signed(origin)?;

      let ref mut blog = Self::blog_by_id(blog_id).ok_or(Error::<T>::BlogNotFound)?;
      ensure!(Self::blog_followed_by_account((follower.clone(), blog_id)), Error::<T>::AccountIsNotFollowingBlog);

      let mut social_account = Self::social_account_by_id(follower.clone()).ok_or(Error::<T>::SocialAccountNotFound)?;
      social_account.following_blogs_count = social_account.following_blogs_count
        .checked_sub(1)
        .ok_or(Error::<T>::UnderflowUnfollowingBlog)?;
      blog.followers_count = blog.followers_count.checked_sub(1).ok_or(Error::<T>::UnderflowUnfollowingBlog)?;

      if blog.created.account != follower {
        let author = blog.created.account.clone();
        if let Some(score_diff) = Self::account_reputation_diff_by_account((follower.clone(), author.clone(), ScoringAction::FollowBlog)) {
          blog.score = blog.score.checked_sub(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingBlogScore)?;
          Self::change_social_account_reputation(author.clone(), follower.clone(), score_diff * -1, ScoringAction::FollowBlog)?;
        }
      }

      <BlogsFollowedByAccount<T>>::mutate(follower.clone(), |blog_ids| Self::vec_remove_on(blog_ids, blog_id));
      <BlogFollowers<T>>::mutate(blog_id, |account_ids| Self::vec_remove_on(account_ids, follower.clone()));
      <BlogFollowedByAccount<T>>::remove((follower.clone(), blog_id));
      <SocialAccountById<T>>::insert(follower.clone(), social_account);
      <BlogById<T>>::insert(blog_id, blog);

      Self::deposit_event(RawEvent::BlogUnfollowed(follower.clone(), blog_id));
    }

    pub fn follow_account(origin, account: T::AccountId) {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, Error::<T>::AccountCannotFollowItself);
      ensure!(!<AccountFollowedByAccount<T>>::exists((follower.clone(), account.clone())), Error::<T>::AccountIsAlreadyFollowed);

      let mut follower_account = Self::get_or_new_social_account(follower.clone());
      let mut followed_account = Self::get_or_new_social_account(account.clone());

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_add(1).ok_or(Error::<T>::OverflowFollowingAccount)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_add(1).ok_or(Error::<T>::OverflowFollowingAccount)?;

      Self::change_social_account_reputation(account.clone(), follower.clone(),
        Self::get_score_diff(follower_account.reputation.clone(), ScoringAction::FollowAccount),
        ScoringAction::FollowAccount
      )?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);
      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |ids| ids.push(follower.clone()));
      <AccountFollowedByAccount<T>>::insert((follower.clone(), account.clone()), true);

      Self::deposit_event(RawEvent::AccountFollowed(follower, account));
    }

    pub fn unfollow_account(origin, account: T::AccountId) {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, Error::<T>::AccountCannotUnfollowItself);

      let mut follower_account = Self::social_account_by_id(follower.clone()).ok_or(Error::<T>::FollowerAccountNotFound)?;
      let mut followed_account = Self::social_account_by_id(account.clone()).ok_or(Error::<T>::FollowedAccountNotFound)?;

      ensure!(<AccountFollowedByAccount<T>>::exists((follower.clone(), account.clone())), Error::<T>::AccountIsNotFollowed);

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_sub(1).ok_or(Error::<T>::UnderflowUnfollowingAccount)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_sub(1).ok_or(Error::<T>::UnderflowUnfollowingAccount)?;

      let reputation_diff = Self::account_reputation_diff_by_account(
        (follower.clone(), account.clone(), ScoringAction::FollowAccount)
      ).ok_or(Error::<T>::ReputationDiffNotFound)?;
      Self::change_social_account_reputation(account.clone(), follower.clone(),
        reputation_diff,
        ScoringAction::FollowAccount
      )?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);
      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |account_ids| Self::vec_remove_on(account_ids, account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |account_ids| Self::vec_remove_on(account_ids, follower.clone()));
      <AccountFollowedByAccount<T>>::remove((follower.clone(), account.clone()));

      Self::deposit_event(RawEvent::AccountUnfollowed(follower, account));
    }

    pub fn create_profile(origin, username: Vec<u8>, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      let mut social_account = Self::get_or_new_social_account(owner.clone());
      ensure!(social_account.profile.is_none(), Error::<T>::ProfileAlreadyExists);
      Self::is_username_valid(username.clone())?;
      Self::is_ipfs_hash_valid(ipfs_hash.clone())?;

      social_account.profile = Some(
        Profile {
          created: WhoAndWhen::<T>::new(owner.clone()),
          updated: None,
          username: username.clone(),
          ipfs_hash,
          edit_history: vec![]
        }
      );
      <AccountByProfileUsername<T>>::insert(username.clone(), owner.clone());
      <SocialAccountById<T>>::insert(owner.clone(), social_account.clone());

      Self::deposit_event(RawEvent::ProfileCreated(owner.clone()));
    }

    pub fn update_profile(origin, update: ProfileUpdate) {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.username.is_some() ||
        update.ipfs_hash.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesInProfile);

      let mut social_account = Self::social_account_by_id(owner.clone()).ok_or(Error::<T>::SocialAccountNotFound)?;
      let mut profile = social_account.profile.ok_or(Error::<T>::ProfileDoesNotExist)?;
      let mut is_update_applied = false;
      let mut new_history_record = ProfileHistoryRecord {
        edited: WhoAndWhen::<T>::new(owner.clone()),
        old_data: ProfileUpdate {username: None, ipfs_hash: None}
      };

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != profile.ipfs_hash {
          Self::is_ipfs_hash_valid(ipfs_hash.clone())?;
          new_history_record.old_data.ipfs_hash = Some(profile.ipfs_hash);
          profile.ipfs_hash = ipfs_hash;
          is_update_applied = true;
        }
      }

      if let Some(username) = update.username {
        if username != profile.username {
          Self::is_username_valid(username.clone())?;
          <AccountByProfileUsername<T>>::remove(profile.username.clone());
          <AccountByProfileUsername<T>>::insert(username.clone(), owner.clone());
          new_history_record.old_data.username = Some(profile.username);
          profile.username = username;
          is_update_applied = true;
        }
      }

      if is_update_applied {
        profile.updated = Some(WhoAndWhen::<T>::new(owner.clone()));
        profile.edit_history.push(new_history_record);
        social_account.profile = Some(profile);
        <SocialAccountById<T>>::insert(owner.clone(), social_account);

        Self::deposit_event(RawEvent::ProfileUpdated(owner.clone()));
      }
    }

    pub fn create_post(origin, blog_id_opt: Option<BlogId>, extension: PostExtension, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      let new_post_id = Self::next_post_id();
      let mut is_comment = None;

      // Sharing functions contain check for post/comment existence
      match extension {
        PostExtension::RegularPost => {
          Self::is_ipfs_hash_valid(ipfs_hash.clone())?;
        },
        PostExtension::Comment(comment_ext) => {
          Self::is_ipfs_hash_valid(ipfs_hash.clone())?;

          if let Some(parent_id) = comment_ext.parent_id {
            let parent_comment = Self::post_by_id(parent_id).ok_or(Error::<T>::UnknownParentComment)?;
            ensure!(parent_comment.is_comment(), Error::<T>::PostNotFound);
          }
          is_comment = Some(comment_ext);
        },
        PostExtension::SharedPost(post_id) => {
          let post = Self::post_by_id(post_id).ok_or(Error::<T>::OriginalPostNotFound)?;
          ensure!(!post.is_shared_post(), Error::<T>::CannotShareSharedPost);
          Self::share_post_by_extension(owner.clone(), post_id, new_post_id)?;
        },
      }

      let new_post: Post<T> = Post {
        id: new_post_id,
        created: WhoAndWhen::<T>::new(owner.clone()),
        updated: None,
        blog_id: blog_id_opt,
        extension,
        ipfs_hash,
        edit_history: vec![],
        total_replies_count: 0,
        shares_count: 0,
        upvotes_count: 0,
        downvotes_count: 0,
        score: 0,
      };

      if let Some(comment_ext) = is_comment {
        let ref mut root_post = Self::post_by_id(comment_ext.root_post_id).ok_or(Error::<T>::PostNotFound)?;
        ensure!(root_post.extension == PostExtension::RegularPost, Error::<T>::NoPostByRootPostId);

        root_post.total_replies_count = root_post.total_replies_count.checked_add(1).ok_or(Error::<T>::OverflowAddingCommentOnPost)?;
        Self::change_post_score_by_extension(owner.clone(), root_post, ScoringAction::CreateComment)?;

        CommentIdsByPostId::mutate(comment_ext.root_post_id, |ids| ids.push(new_post_id));
        <PostById<T>>::insert(comment_ext.root_post_id, root_post);
        Self::deposit_event(RawEvent::CommentCreated(owner.clone(), new_post_id));
      } else if let Some(blog_id) = blog_id_opt {
          let mut blog = Self::blog_by_id(blog_id).ok_or(Error::<T>::BlogNotFound)?;
          blog.posts_count = blog.posts_count.checked_add(1).ok_or(Error::<T>::OverflowAddingPostOnBlog)?;

          <BlogById<T>>::insert(blog_id, blog);
          PostIdsByBlogId::mutate(blog_id, |ids| ids.push(new_post_id));
          Self::deposit_event(RawEvent::PostCreated(owner.clone(), new_post_id));
      } else {
        return Err(Error::<T>::BlogIdIsUndefined.into());
      }

      <PostById<T>>::insert(new_post_id, new_post);
      NextPostId::mutate(|n| { *n += 1; });
    }

    pub fn update_post(origin, post_id: PostId, update: PostUpdate) {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.blog_id.is_some() ||
        update.ipfs_hash.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesInPost);

      let mut post = Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?;

      // TODO ensure: blog writers also should be able to edit this post:
      ensure!(owner == post.created.account, Error::<T>::NotAnAuthor);

      let mut fields_updated = 0;
      let mut new_history_record = PostHistoryRecord {
        edited: WhoAndWhen::<T>::new(owner.clone()),
        old_data: PostUpdate {blog_id: None, ipfs_hash: None}
      };

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != post.ipfs_hash {
          Self::is_ipfs_hash_valid(ipfs_hash.clone())?;
          new_history_record.old_data.ipfs_hash = Some(post.ipfs_hash);
          post.ipfs_hash = ipfs_hash;
          fields_updated += 1;
        } else {
          if post.is_comment() {
            return Err(Error::<T>::CommentIPFSHashNotDiffer.into());
          }
        }
      }

      // Move this post to another blog:
      if let Some(blog_id) = update.blog_id {
        ensure!(!post.is_comment(), Error::<T>::CannotUpdateBlogIdOnComment);

        if let Some(post_blog_id) = post.blog_id {
          if blog_id != post_blog_id {
            Self::ensure_blog_exists(blog_id)?;

            // Remove post_id from its old blog:
            PostIdsByBlogId::mutate(post_blog_id, |post_ids| Self::vec_remove_on(post_ids, post_id));

            // Add post_id to its new blog:
            PostIdsByBlogId::mutate(blog_id.clone(), |ids| ids.push(post_id));
            new_history_record.old_data.blog_id = post.blog_id;
            post.blog_id = Some(blog_id);
            fields_updated += 1;
          }
        }
      }

      // Update this post only if at least one field should be updated:
      if fields_updated > 0 {
        post.updated = Some(WhoAndWhen::<T>::new(owner.clone()));
        post.edit_history.push(new_history_record);
        <PostById<T>>::insert(post_id, post.clone());

        if post.is_comment() {
          Self::deposit_event(RawEvent::PostUpdated(owner.clone(), post_id));
        } else {
          Self::deposit_event(RawEvent::PostUpdated(owner.clone(), post_id));
        }
      }
    }

    pub fn create_post_reaction(origin, post_id: PostId, kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      let ref mut post = Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?;
      if post.is_comment() {
        ensure!(
          !<CommentReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
          Error::<T>::AccountAlreadyReacted
        );
      } else {
        ensure!(
          !<PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
          Error::<T>::AccountAlreadyReacted
        );
      }

      let reaction_id = Self::new_reaction(owner.clone(), kind.clone());

      match kind {
        ReactionKind::Upvote => post.upvotes_count = post.upvotes_count.checked_add(1).ok_or(Error::<T>::OverflowUpvoting)?,
        ReactionKind::Downvote => post.downvotes_count = post.downvotes_count.checked_add(1).ok_or(Error::<T>::OverflowDownvoting)?,
      }
      let action = Self::scoring_action_by_post_extension(post.extension, kind, false);

      if post.created.account != owner {
        Self::change_post_score_by_extension(owner.clone(), post, action)?;
      }
      else {
        <PostById<T>>::insert(post_id, post.clone());
      }

      if post.is_comment() {
        ReactionIdsByCommentId::mutate(post_id, |ids| ids.push(reaction_id));
        <CommentReactionIdByAccount<T>>::insert((owner.clone(), post_id), reaction_id);
      } else {
        ReactionIdsByPostId::mutate(post_id, |ids| ids.push(reaction_id));
        <PostReactionIdByAccount<T>>::insert((owner.clone(), post_id), reaction_id);
      }

      Self::deposit_event(RawEvent::PostReactionCreated(owner.clone(), post_id, reaction_id));
    }

    pub fn update_post_reaction(origin, post_id: PostId, reaction_id: ReactionId, new_kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        Error::<T>::AccountNotYetReacted
      );

      let mut reaction = Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?;
      let ref mut post = Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?;

      ensure!(owner == reaction.created.account, Error::<T>::NotAReactionOwner);
      ensure!(reaction.kind != new_kind, Error::<T>::NewReactionKindNotDiffer);

      reaction.kind = new_kind;
      reaction.updated = Some(WhoAndWhen::<T>::new(owner.clone()));

      match new_kind {
        ReactionKind::Upvote => {
          post.upvotes_count += 1;
          post.downvotes_count -= 1;
        },
        ReactionKind::Downvote => {
          post.downvotes_count += 1;
          post.upvotes_count -= 1;
        },
      }
      let action = Self::scoring_action_by_post_extension(post.extension, new_kind, false);
      let action_to_cancel = Self::scoring_action_by_post_extension(post.extension, new_kind, true);

      Self::change_post_score_by_extension(owner.clone(), post, action_to_cancel)?;
      Self::change_post_score_by_extension(owner.clone(), post, action)?;

      <ReactionById<T>>::insert(reaction_id, reaction);
      <PostById<T>>::insert(post_id, post);

      Self::deposit_event(RawEvent::PostReactionUpdated(owner.clone(), post_id, reaction_id));
    }

    pub fn delete_post_reaction(origin, post_id: PostId, reaction_id: ReactionId) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        Error::<T>::ReactionByAccountNotFound
      );

      let reaction = Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?;
      let ref mut post = Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?;

      ensure!(owner == reaction.created.account, Error::<T>::NotAReactionOwner);

      match reaction.kind {
        ReactionKind::Upvote => post.upvotes_count -= 1,
        ReactionKind::Downvote => post.downvotes_count -= 1,
      }
      let action_to_cancel = Self::scoring_action_by_post_extension(post.extension, reaction.kind, false);

      Self::change_post_score_by_extension(owner.clone(), post, action_to_cancel)?;

      <PostById<T>>::insert(post_id, post);
      <ReactionById<T>>::remove(reaction_id);
      ReactionIdsByPostId::mutate(post_id, |ids| Self::vec_remove_on(ids, reaction_id));
      <PostReactionIdByAccount<T>>::remove((owner.clone(), post_id));

      Self::deposit_event(RawEvent::PostReactionDeleted(owner.clone(), post_id, reaction_id));
    }
  }
}

decl_event!(
  pub enum Event<T> where
    <T as system::Trait>::AccountId,
   {
    BlogCreated(AccountId, BlogId),
    BlogUpdated(AccountId, BlogId),
    BlogDeleted(AccountId, BlogId),

    BlogFollowed(AccountId, BlogId),
    BlogUnfollowed(AccountId, BlogId),

    AccountReputationChanged(AccountId, ScoringAction, u32),

    AccountFollowed(AccountId, AccountId),
    AccountUnfollowed(AccountId, AccountId),

    PostCreated(AccountId, PostId),
    PostUpdated(AccountId, PostId),
    PostDeleted(AccountId, PostId),
    PostShared(AccountId, PostId),

    CommentCreated(AccountId, PostId),
    CommentUpdated(AccountId, PostId),
    CommentDeleted(AccountId, PostId),
    CommentShared(AccountId, PostId),

    PostReactionCreated(AccountId, PostId, ReactionId),
    PostReactionUpdated(AccountId, PostId, ReactionId),
    PostReactionDeleted(AccountId, PostId, ReactionId),

    CommentReactionCreated(AccountId, PostId, ReactionId),
    CommentReactionUpdated(AccountId, PostId, ReactionId),
    CommentReactionDeleted(AccountId, PostId, ReactionId),

    ProfileCreated(AccountId),
    ProfileUpdated(AccountId),
  }
);

use super::*;

use frame_support::{dispatch::{DispatchResult, DispatchError}};

impl<T: Trait> Module<T> {

    pub fn ensure_blog_exists(blog_id: BlogId) -> DispatchResult {
        ensure!(<BlogById<T>>::exists(blog_id), Error::<T>::BlogNotFound);
        Ok(())
    }

    // TODO: maybe don't add reaction in storage before checks in 'create_reaction' are done?
    pub fn new_reaction(account: T::AccountId, kind: ReactionKind) -> ReactionId {
        let reaction_id = Self::next_reaction_id();
        let new_reaction: Reaction<T> = Reaction {
            id: reaction_id,
            created: WhoAndWhen::<T>::new(account),
            updated: None,
            kind
        };

        <ReactionById<T>>::insert(reaction_id, new_reaction);
        NextReactionId::mutate(|n| { *n += 1; });

        reaction_id
    }

    pub fn add_blog_follower_and_insert_blog(
        follower: T::AccountId,
        blog: &mut Blog<T>,
        is_new_blog: bool
    ) -> DispatchResult {

        let blog_id = blog.id;
        let mut social_account = Self::get_or_new_social_account(follower.clone());
        social_account.following_blogs_count = social_account.following_blogs_count
            .checked_add(1)
            .ok_or(Error::<T>::OverflowFollowingBlog)?;

        blog.followers_count = blog.followers_count.checked_add(1).ok_or(Error::<T>::OverflowFollowingBlog)?;
        if blog.created.account != follower {
            let author = blog.created.account.clone();
            let score_diff = Self::get_score_diff(social_account.reputation, ScoringAction::FollowBlog);
            blog.score = blog.score.checked_add(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingBlogScore)?;
            Self::change_social_account_reputation(author, follower.clone(), score_diff, ScoringAction::FollowBlog)?;
        }

        <BlogById<T>>::insert(blog_id, blog);
        <SocialAccountById<T>>::insert(follower.clone(), social_account);
        <BlogsFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(blog_id));
        <BlogFollowers<T>>::mutate(blog_id, |ids| ids.push(follower.clone()));
        <BlogFollowedByAccount<T>>::insert((follower.clone(), blog_id), true);

        if is_new_blog {
            Self::deposit_event(RawEvent::BlogCreated(follower.clone(), blog_id));
        }

        Self::deposit_event(RawEvent::BlogFollowed(follower, blog_id));

        Ok(())
    }

    pub fn get_or_new_social_account(account: T::AccountId) -> SocialAccount<T> {
        if let Some(social_account) = Self::social_account_by_id(account) {
            social_account
        } else {
            SocialAccount {
                followers_count: 0,
                following_accounts_count: 0,
                following_blogs_count: 0,
                reputation: 1,
                profile: None
            }
        }
    }

    pub fn vec_remove_on<F: PartialEq>(vector: &mut Vec<F>, element: F) {
        if let Some(index) = vector.iter().position(|x| *x == element) {
            vector.swap_remove(index);
        }
    }

    pub fn change_post_score_by_extension(account: T::AccountId, post: &mut Post<T>, action: ScoringAction) -> DispatchResult {
        if post.is_comment() {
            Self::change_comment_score(account, post, action)?;
        } else {
            Self::change_post_score(account, post, action)?;
        }

        Ok(())
    }

    fn change_post_score(account: T::AccountId, post: &mut Post<T>, action: ScoringAction) -> DispatchResult {
        let social_account = Self::get_or_new_social_account(account.clone());
        <SocialAccountById<T>>::insert(account.clone(), social_account.clone());

        let post_id = post.id;
        ensure!(<PostById<T>>::exists(post_id), Error::<T>::PostNotFound);
        ensure!(!post.is_comment(), Error::<T>::PostIsAComment);

        if let Some(post_blog_id) = post.blog_id {
            let mut blog = Self::blog_by_id(post_blog_id).ok_or(Error::<T>::BlogNotFound)?;

            if post.created.account != account {
                if let Some(score_diff) = Self::post_score_by_account((account.clone(), post_id, action)) {
                    let reputation_diff = Self::account_reputation_diff_by_account((account.clone(), post.created.account.clone(), action)).ok_or(Error::<T>::ReputationDiffNotFound)?;
                    post.score = post.score.checked_add(score_diff as i32 * -1).ok_or(Error::<T>::OutOfBoundsRevertingPostScore)?;
                    blog.score = blog.score.checked_add(score_diff as i32 * -1).ok_or(Error::<T>::OutOfBoundsRevertingBlogScore)?;
                    Self::change_social_account_reputation(post.created.account.clone(), account.clone(), reputation_diff * -1, action)?;
                    <PostScoreByAccount<T>>::remove((account, post_id, action));
                } else {
                    match action {
                        ScoringAction::UpvotePost => {
                            if Self::post_score_by_account((account.clone(), post_id, ScoringAction::DownvotePost)).is_some() {
                                Self::change_post_score(account.clone(), post, ScoringAction::DownvotePost)?;
                            }
                        },
                        ScoringAction::DownvotePost => {
                            if Self::post_score_by_account((account.clone(), post_id, ScoringAction::UpvotePost)).is_some() {
                                Self::change_post_score(account.clone(), post, ScoringAction::UpvotePost)?;
                            }
                        },
                        _ => (),
                    }
                    let score_diff = Self::get_score_diff(social_account.reputation, action);
                    post.score = post.score.checked_add(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingPostScore)?;
                    blog.score = blog.score.checked_add(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingBlogScore)?;
                    Self::change_social_account_reputation(post.created.account.clone(), account.clone(), score_diff, action)?;
                    <PostScoreByAccount<T>>::insert((account, post_id, action), score_diff);
                }

                <PostById<T>>::insert(post_id, post.clone());
                <BlogById<T>>::insert(post_blog_id, blog);
            }
        }

        Ok(())
    }

    fn change_comment_score(account: T::AccountId, comment: &mut Post<T>, action: ScoringAction) -> DispatchResult {
        let social_account = Self::get_or_new_social_account(account.clone());
        <SocialAccountById<T>>::insert(account.clone(), social_account.clone());

        let comment_id = comment.id;
        ensure!(<PostById<T>>::exists(comment_id), Error::<T>::PostNotFound);
        let comment_ext = comment.get_comment_ext()?;

        ensure!(comment.is_comment(), Error::<T>::PostIsNotAComment);

        if comment.created.account != account {
            if let Some(score_diff) = Self::post_score_by_account((account.clone(), comment_id, action)) {
                let reputation_diff = Self::account_reputation_diff_by_account((account.clone(), comment.created.account.clone(), action)).ok_or(Error::<T>::ReputationDiffNotFound)?;
                comment.score = comment.score.checked_add(score_diff as i32 * -1).ok_or(Error::<T>::OutOfBoundsRevertingCommentScore)?;
                Self::change_social_account_reputation(comment.created.account.clone(), account.clone(), reputation_diff * -1, action)?;
                <PostScoreByAccount<T>>::remove((account.clone(), comment_id, action));
            } else {
                match action {
                    ScoringAction::UpvoteComment => {
                        if Self::post_score_by_account((account.clone(), comment_id, ScoringAction::DownvoteComment)).is_some() {
                            Self::change_comment_score(account.clone(), comment, ScoringAction::DownvoteComment)?;
                        }
                    },
                    ScoringAction::DownvoteComment => {
                        if Self::post_score_by_account((account.clone(), comment_id, ScoringAction::UpvoteComment)).is_some() {
                            Self::change_comment_score(account.clone(), comment, ScoringAction::UpvoteComment)?;
                        }
                    },
                    ScoringAction::CreateComment => {
                        let ref mut post = Self::post_by_id(comment_ext.root_post_id).ok_or(Error::<T>::PostNotFound)?;
                        Self::change_post_score(account.clone(), post, action)?;
                    }
                    _ => (),
                }
                let score_diff = Self::get_score_diff(social_account.reputation, action);
                comment.score = comment.score.checked_add(score_diff as i32).ok_or(Error::<T>::OutOfBoundsUpdatingCommentScore)?;
                Self::change_social_account_reputation(comment.created.account.clone(), account.clone(), score_diff, action)?;
                <PostScoreByAccount<T>>::insert((account, comment_id, action), score_diff);
            }
            <PostById<T>>::insert(comment_id, comment.clone());
        }

        Ok(())
    }

    pub fn change_social_account_reputation(account: T::AccountId, scorer: T::AccountId, mut score_diff: i16, action: ScoringAction) -> DispatchResult {
        let mut social_account = Self::get_or_new_social_account(account.clone());

        if social_account.reputation as i64 + score_diff as i64 <= 1 {
            social_account.reputation = 1;
            score_diff = 0;
        }

        if score_diff < 0 {
            social_account.reputation = social_account.reputation.checked_sub((score_diff * -1) as u32).ok_or(Error::<T>::OutOfBoundsUpdatingAccountReputation)?;
        } else {
            social_account.reputation = social_account.reputation.checked_add(score_diff as u32).ok_or(Error::<T>::OutOfBoundsUpdatingAccountReputation)?;
        }

        if Self::account_reputation_diff_by_account((scorer.clone(), account.clone(), action)).is_some() {
            <AccountReputationDiffByAccount<T>>::remove((scorer.clone(), account.clone(), action));
        } else {
            <AccountReputationDiffByAccount<T>>::insert((scorer.clone(), account.clone(), action), score_diff);
        }

        <SocialAccountById<T>>::insert(account.clone(), social_account.clone());

        Self::deposit_event(RawEvent::AccountReputationChanged(account, action, social_account.reputation));

        Ok(())
    }

    pub fn get_score_diff(reputation: u32, action: ScoringAction) -> i16 {
        let r = Self::log_2(reputation);
        let d = (reputation - (2 as u32).pow(r)) * 100 / (2 as u32).pow(r);
        let score_diff = ((r + 1) * 100 + d) / 100;

        score_diff as i16 * Self::weight_of_scoring_action(action)
    }

    // TODO write unit tests for this method.
    pub fn weight_of_scoring_action(action: ScoringAction) -> i16 {
        match action {
            ScoringAction::UpvotePost => Self::upvote_post_action_weight(),
            ScoringAction::DownvotePost => Self::downvote_post_action_weight(),
            ScoringAction::SharePost => Self::share_post_action_weight(),
            ScoringAction::CreateComment => Self::create_comment_action_weight(),
            ScoringAction::UpvoteComment => Self::upvote_comment_action_weight(),
            ScoringAction::DownvoteComment => Self::downvote_comment_action_weight(),
            ScoringAction::ShareComment => Self::share_comment_action_weight(),
            ScoringAction::FollowBlog => Self::follow_blog_action_weight(),
            ScoringAction::FollowAccount => Self::follow_account_action_weight(),
        }
    }

    fn num_bits<P>() -> usize { sp_std::mem::size_of::<P>() * 8 }

    pub fn log_2(x: u32) -> u32 {
        assert!(x > 0);
        Self::num_bits::<u32>() as u32 - x.leading_zeros() - 1
    }

    pub fn is_username_valid(username: Vec<u8>) -> DispatchResult {
        ensure!(Self::account_by_profile_username(username.clone()).is_none(), Error::<T>::UsernameIsBusy);
        ensure!(username.len() >= Self::username_min_len() as usize, Error::<T>::UsernameIsTooShort);
        ensure!(username.len() <= Self::username_max_len() as usize, Error::<T>::UsernameIsTooLong);
        ensure!(username.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::UsernameIsNotAlphanumeric);

        Ok(())
    }

    pub fn is_ipfs_hash_valid(ipfs_hash: Vec<u8>) -> DispatchResult {
        ensure!(ipfs_hash.len() == Self::ipfs_hash_len() as usize, Error::<T>::IpfsIsIncorrect);

        Ok(())
    }

    pub fn share_post(account: T::AccountId, original_post: &mut Post<T>, shared_post_id: PostId) -> DispatchResult {
        original_post.shares_count = original_post.shares_count.checked_add(1).ok_or(Error::<T>::OverflowTotalShares)?;

        let original_post_id = original_post.id;

        let mut shares_count = Self::post_shares_by_account((account.clone(), original_post_id));
        shares_count = shares_count.checked_add(1).ok_or(Error::<T>::OverflowPostShares)?;

        if shares_count == 1 {
            Self::change_post_score_by_extension(account.clone(), original_post, {
                if original_post.is_comment() { ScoringAction::ShareComment } else {ScoringAction::SharePost}
            })?;
        }

        <PostById<T>>::insert(original_post_id, original_post.clone());
        <PostSharesByAccount<T>>::insert((account.clone(), original_post_id), shares_count);
        SharedPostIdsByOriginalPostId::mutate(original_post_id, |ids| ids.push(shared_post_id));

        if original_post.is_comment() {
            Self::deposit_event(RawEvent::CommentShared(account, original_post_id));
        } else {
            Self::deposit_event(RawEvent::PostShared(account, original_post_id));
        }

        Ok(())
    }

    pub fn scoring_action_by_post_extension(extension: PostExtension, reaction_kind: ReactionKind, reverse: bool) -> ScoringAction {
        let scoring_action;

        match extension {
            PostExtension::RegularPost | PostExtension::SharedPost(_) => match reaction_kind {
                ReactionKind::Upvote => scoring_action = if reverse {ScoringAction::DownvotePost} else {ScoringAction::UpvotePost},
                ReactionKind::Downvote => scoring_action = if reverse {ScoringAction::UpvotePost} else {ScoringAction::DownvotePost},
            },
            PostExtension::Comment(_) => match reaction_kind {
                ReactionKind::Upvote => scoring_action = if reverse {ScoringAction::DownvoteComment} else {ScoringAction::UpvoteComment},
                ReactionKind::Downvote => scoring_action = if reverse {ScoringAction::UpvoteComment} else {ScoringAction::DownvoteComment},
            },
        }

        scoring_action
    }
}

impl<T: Trait> Post<T> {
    pub fn is_comment(&self) -> bool {
        return match self.extension {
            PostExtension::Comment(_) => true,
            _ => false,
        }
    }

    pub fn is_shared_post(&self) -> bool {
        return match self.extension {
            PostExtension::SharedPost(_) => true,
            _ => false,
        }
    }

    // TODO: Look for cases where this function can be used
    pub fn get_comment_ext(&self) -> Result<CommentExt, DispatchError> {
        return match self.extension {
            PostExtension::Comment(comment_ext) => Ok(comment_ext),
            _ => Err(Error::<T>::PostIsNotAComment.into()),
        }
    }
}

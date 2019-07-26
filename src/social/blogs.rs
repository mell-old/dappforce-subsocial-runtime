use rstd::prelude::*;
use parity_codec::Codec;
use parity_codec_derive::{Encode, Decode};
use srml_support::{StorageMap, StorageValue, decl_module, decl_storage, decl_event, dispatch, ensure, fail, Parameter};
use runtime_primitives::traits::{SimpleArithmetic, As, Member, MaybeDebug, MaybeSerializeDebug};
use system::{self, ensure_signed};
use runtime_io::print;
use {timestamp};

pub const DEFAULT_SLUG_MAX_LEN: u32 = 50;
pub const DEFAULT_SLUG_MIN_LEN: u32 = 5;

pub const DEFAULT_BLOG_MAX_LEN: u32 = 1_000;
pub const DEFAULT_POST_MAX_LEN: u32 = 10_000;
pub const DEFAULT_COMMENT_MAX_LEN: u32 = 1_000;

pub const MSG_BLOG_NOT_FOUND: &str = "Blog was not found by id";
pub const MSG_BLOG_SLUG_IS_TOO_SHORT: &str = "Blog slug is too short";
pub const MSG_BLOG_SLUG_IS_TOO_LONG: &str = "Blog slug is too long";
pub const MSG_BLOG_SLUG_IS_NOT_UNIQUE: &str = "Blog slug is not unique";
pub const MSG_NOTHING_TO_UPDATE_IN_BLOG: &str = "Nothing to update in a blog";
pub const MSG_ONLY_BLOG_OWNER_CAN_UPDATE_BLOG: &str = "Only a blog owner can update their blog";

pub const MSG_POST_NOT_FOUND: &str = "Post was not found by id";
pub const MSG_POST_SLUG_IS_TOO_SHORT: &str = "Post slug is too short";
pub const MSG_POST_SLUG_IS_TOO_LONG: &str = "Post slug is too long";
pub const MSG_POST_SLUG_IS_NOT_UNIQUE: &str = "Post slug is not unique";
pub const MSG_NOTHING_TO_UPDATE_IN_POST: &str = "Nothing to update in a post";
pub const MSG_ONLY_POST_OWNER_CAN_UPDATE_POST: &str = "Only post owner can update their post";

pub const MSG_COMMENT_NOT_FOUND: &str = "Comment was not found by id";
pub const MSG_UNKNOWN_PARENT_COMMENT: &str = "Unknown parent comment id";
pub const MSG_ONLY_COMMENT_AUTHOR_CAN_UPDATE_COMMENT: &str = "Only comment author can update their comment";
pub const MSG_NEW_COMMENT_HASH_DO_NOT_DIFFER: &str = "New comment IPFS-hash is the same as old one";

pub const MSG_REACTION_NOT_FOUND: &str = "Reaction was not found by id";
pub const MSG_ACCOUNT_ALREADY_REACTED_TO_POST: &str = "Account has already reacted to this post. To change a kind of reaction call update_post_reaction()";
pub const MSG_ACCOUNT_HAS_NOT_REACTED_TO_POST: &str = "Account has not reacted to this post yet. Use create_post_reaction()";
pub const MSG_NO_POST_REACTION_BY_ACCOUNT_TO_DELETE: &str = "There is no post reaction by account that could be deleted";
pub const MSG_ACCOUNT_ALREADY_REACTED_TO_COMMENT: &str = "Account has already reacted to this comment. To change a kind of reaction call pub update_comment_reaction()";
pub const MSG_ACCOUNT_HAS_NOT_REACTED_TO_COMMENT: &str = "Account has not reacted to this comment yet. Use create_comment_reaction()";
pub const MSG_NO_COMMENT_REACTION_BY_ACCOUNT_TO_DELETE: &str = "There is no comment reaction by account that could be deleted";
pub const MSG_ONLY_REACTION_OWNER_CAN_UPDATE_REACTION: &str = "Only reaction owner can update their reaction";
pub const MSG_NEW_REACTION_KIND_DO_NOT_DIFFER: &str = "New reaction kind is the same as old one";

pub const MSG_ACCOUNT_IS_FOLLOWING_BLOG: &str = "Account is already following this blog";
pub const MSG_ACCOUNT_IS_NOT_FOLLOWING_BLOG: &str = "Account is not following this blog";
pub const MSG_ACCOUNT_CANNOT_FOLLOW_ITSELF: &str = "Account can not follow itself";
pub const MSG_ACCOUNT_CANNOT_UNFOLLOW_ITSELF: &str = "Account can not unfollow itself";
pub const MSG_ACCOUNT_IS_ALREADY_FOLLOWED: &str = "Account is already followed";
pub const MSG_UNDERFLOW_UNFOLLOWING_BLOG: &str = "Underflow unfollowing blog";
pub const MSG_OVERFLOW_FOLLOWING_BLOG: &str = "Overflow following blog";
pub const MSG_OVERFLOW_FOLLOWING_ACCOUNT: &str = "Overflow following account";
pub const MSG_UNDERFLOW_UNFOLLOWING_ACCOUNT: &str = "Overflow following account";

pub const MSG_SOCIAL_ACCOUNT_NOT_FOUND: &str = "Social account was not found by id";
pub const MSG_FOLLOWER_ACCOUNT_NOT_FOUND: &str = "Follower social account was not found by id";
pub const MSG_FOLLOWED_ACCOUNT_NOT_FOUND: &str = "Followed social account was not found by id";

pub trait Trait: system::Trait + timestamp::Trait + MaybeDebug {

  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

  type BlogId: Parameter + Member + SimpleArithmetic + Codec + Default + Copy
    + As<usize> + As<u64> + MaybeSerializeDebug + PartialEq;

  type PostId: Parameter + Member + SimpleArithmetic + Codec + Default + Copy
    + As<usize> + As<u64> + MaybeSerializeDebug + PartialEq;

  type CommentId: Parameter + Member + SimpleArithmetic + Codec + Default + Copy
    + As<usize> + As<u64> + MaybeSerializeDebug + PartialEq;

  type ReactionId: Parameter + Member + SimpleArithmetic + Codec + Default + Copy
    + As<usize> + As<u64> + MaybeSerializeDebug + PartialEq;
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Copy, Encode, Decode, PartialEq)]
pub struct Change<T: Trait> {
  account: T::AccountId,
  block: T::BlockNumber,
  time: T::Moment,
}

// TODO add a schema along w/ JSON, maybe create a struct?

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Encode, Decode, PartialEq)]
pub struct Blog<T: Trait> {
  id: T::BlogId,
  created: Change<T>,
  updated: Option<Change<T>>,

  // Can be updated by the owner:
  writers: Vec<T::AccountId>,
  slug: Vec<u8>,
  ipfs_hash: Vec<u8>,

  posts_count: u16,
  followers_count: u32,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Encode, Decode, PartialEq)]
pub struct BlogUpdate<T: Trait> {
  pub writers: Option<Vec<T::AccountId>>,
  pub slug: Option<Vec<u8>>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Encode, Decode, PartialEq)]
pub struct Post<T: Trait> {
  id: T::PostId,
  blog_id: T::BlogId,
  created: Change<T>,
  updated: Option<Change<T>>,

  // Next fields can be updated by the owner only:

  // TODO make slug optional for post or even remove it
  slug: Vec<u8>,
  ipfs_hash: Vec<u8>,

  comments_count: u16,
  upvotes_count: u16,
  downvotes_count: u16,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Encode, Decode, PartialEq)]
pub struct PostUpdate<T: Trait> {
  pub blog_id: Option<T::BlogId>,
  pub slug: Option<Vec<u8>>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Encode, Decode, PartialEq)]
pub struct Comment<T: Trait> {
  id: T::CommentId,
  parent_id: Option<T::CommentId>,
  post_id: T::PostId,
  created: Change<T>,
  updated: Option<Change<T>>,

  // Can be updated by the owner:
  ipfs_hash: Vec<u8>,

  upvotes_count: u16,
  downvotes_count: u16,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Encode, Decode, PartialEq)]
pub struct CommentUpdate {
  ipfs_hash: Vec<u8>,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq)]
pub enum ReactionKind {
    Upvote,
    Downvote,
}

impl Default for ReactionKind {
    fn default() -> Self {
        ReactionKind::Upvote
    }
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Encode, Decode, PartialEq)]
pub struct Reaction<T: Trait> {
  id: T::ReactionId,
  created: Change<T>,
  updated: Option<Change<T>>,
  kind: ReactionKind,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Encode, Decode, PartialEq)]
pub struct SocialAccount {
  followers_count: u32,
  following_accounts_count: u16,
  following_blogs_count: u16,
}

decl_storage! {
  trait Store for Module<T: Trait> as Blogs {

    SlugMinLen get(slug_min_len): u32 = DEFAULT_SLUG_MIN_LEN;
    SlugMaxLen get(slug_max_len): u32 = DEFAULT_SLUG_MAX_LEN;

    BlogMaxLen get(blog_max_len): u32 = DEFAULT_BLOG_MAX_LEN;
    PostMaxLen get(post_max_len): u32 = DEFAULT_POST_MAX_LEN;
    CommentMaxLen get(comment_max_len): u32 = DEFAULT_COMMENT_MAX_LEN;

    BlogById get(blog_by_id): map T::BlogId => Option<Blog<T>>;
    PostById get(post_by_id): map T::PostId => Option<Post<T>>;
    CommentById get(comment_by_id): map T::CommentId => Option<Comment<T>>;
    ReactionById get(reaction_by_id): map T::ReactionId => Option<Reaction<T>>;
    SocialAccountById get(social_account_by_id): map T::AccountId => Option<SocialAccount>;

    BlogIdsByOwner get(blog_ids_by_owner): map T::AccountId => Vec<T::BlogId>;
    PostIdsByBlogId get(post_ids_by_blog_id): map T::BlogId => Vec<T::PostId>;
    CommentIdsByPostId get(comment_ids_by_post_id): map T::PostId => Vec<T::CommentId>;

    ReactionIdsByPostId get(reaction_ids_by_post_id): map T::PostId => Vec<T::ReactionId>;
    ReactionIdsByCommentId get(reaction_ids_by_comment_id): map T::CommentId => Vec<T::ReactionId>;
    PostReactionIdByAccount get(post_reaction_id_by_account): map (T::AccountId, T::PostId) => T::ReactionId;
    CommentReactionIdByAccount get(comment_reaction_id_by_account): map (T::AccountId, T::CommentId) => T::ReactionId;

    BlogIdBySlug get(blog_id_by_slug): map Vec<u8> => Option<T::BlogId>;
    PostIdBySlug get(post_id_by_slug): map Vec<u8> => Option<T::PostId>;

    BlogsFollowedByAccount get(blogs_followed_by_account): map T::AccountId => Vec<T::BlogId>;
    BlogFollowers get(blog_followers): map T::BlogId => Vec<T::AccountId>;
    BlogFollowedByAccount get(blog_followed_by_account): map (T::AccountId, T::BlogId) => bool;

    AccountFollowedByAccount get(account_followed_by_account): map (T::AccountId, T::AccountId) => bool;
    AccountsFollowedByAccount get(accounts_followed_by_account): map T::AccountId => Vec<T::AccountId>;
    AccountFollowers get(account_followers): map T::AccountId => Vec<T::AccountId>;

    NextBlogId get(next_blog_id): T::BlogId = T::BlogId::sa(1);
    NextPostId get(next_post_id): T::PostId = T::PostId::sa(1);
    NextCommentId get(next_comment_id): T::CommentId = T::CommentId::sa(1);
    NextReactionId get(next_reaction_id): T::ReactionId = T::ReactionId::sa(1);
  }
}

decl_event! {
  pub enum Event<T> where
    <T as system::Trait>::AccountId,
    <T as Trait>::BlogId,
    <T as Trait>::PostId,
    <T as Trait>::CommentId,
    <T as Trait>::ReactionId
  {
    BlogCreated(AccountId, BlogId),
    BlogUpdated(AccountId, BlogId),
    BlogDeleted(AccountId, BlogId),

    BlogFollowed(AccountId, BlogId),
    BlogUnfollowed(AccountId, BlogId),

    AccountFollowed(AccountId, AccountId),
    AccountUnfollowed(AccountId, AccountId),

    PostCreated(AccountId, PostId),
    PostUpdated(AccountId, PostId),
    PostDeleted(AccountId, PostId),

    CommentCreated(AccountId, CommentId),
    CommentUpdated(AccountId, CommentId),
    CommentDeleted(AccountId, CommentId),

    PostReactionCreated(AccountId, PostId, ReactionId),
    PostReactionUpdated(AccountId, PostId, ReactionId),
    PostReactionDeleted(AccountId, PostId, ReactionId),

    CommentReactionCreated(AccountId, CommentId, ReactionId),
    CommentReactionUpdated(AccountId, CommentId, ReactionId),
    CommentReactionDeleted(AccountId, CommentId, ReactionId),
  }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    fn deposit_event<T>() = default;

    fn on_initialize(_now: T::BlockNumber) {
      // Stub
    }

    fn on_finalize(_now: T::BlockNumber) {
      // Stub
    }

    // TODO use BlogUpdate to pass data
    pub fn create_blog(origin, slug: Vec<u8>, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      ensure!(slug.len() >= Self::slug_min_len() as usize, MSG_BLOG_SLUG_IS_TOO_SHORT);
      ensure!(slug.len() <= Self::slug_max_len() as usize, MSG_BLOG_SLUG_IS_TOO_LONG);
      ensure!(!<BlogIdBySlug<T>>::exists(slug.clone()), MSG_BLOG_SLUG_IS_NOT_UNIQUE);
      // TODO validate ipfs_hash

      let blog_id = Self::next_blog_id();
      let new_blog: Blog<T> = Blog {
        id: blog_id,
        created: Self::new_change(owner.clone()),
        updated: None,
        writers: vec![],
        slug: slug.clone(),
        ipfs_hash,
        posts_count: 0,
        followers_count: 0
      };

      <BlogIdsByOwner<T>>::mutate(owner.clone(), |ids| ids.push(blog_id));
      <BlogIdBySlug<T>>::insert(slug, blog_id);
      <NextBlogId<T>>::mutate(|n| { *n += T::BlogId::sa(1); });

      // Blog creator automatically follows their blog:
      Self::add_blog_follower_and_insert_blog(owner.clone(), blog_id, new_blog, true)?;
    }

    fn follow_blog(origin, blog_id: T::BlogId) {
      let follower = ensure_signed(origin)?;

      let blog = Self::blog_by_id(blog_id).ok_or(MSG_BLOG_NOT_FOUND)?;
      ensure!(!Self::blog_followed_by_account((follower.clone(), blog_id)), MSG_ACCOUNT_IS_FOLLOWING_BLOG);

      Self::add_blog_follower_and_insert_blog(follower.clone(), blog_id, blog, false)?;
    }

    fn unfollow_blog(origin, blog_id: T::BlogId) {
      let follower = ensure_signed(origin)?;

      let mut blog = Self::blog_by_id(blog_id).ok_or(MSG_BLOG_NOT_FOUND)?;
      ensure!(Self::blog_followed_by_account((follower.clone(), blog_id)), MSG_ACCOUNT_IS_NOT_FOLLOWING_BLOG);

      <BlogsFollowedByAccount<T>>::mutate(follower.clone(), |blog_ids| Self::vec_remove_on(blog_ids, blog_id));
      <BlogFollowers<T>>::mutate(blog_id, |account_ids| Self::vec_remove_on(account_ids, follower.clone()));
      <BlogFollowedByAccount<T>>::remove((follower.clone(), blog_id));

      let mut social_account = Self::social_account_by_id(follower.clone()).ok_or(MSG_SOCIAL_ACCOUNT_NOT_FOUND)?;
      social_account.following_blogs_count = social_account.following_blogs_count
        .checked_sub(1)
        .ok_or(MSG_UNDERFLOW_UNFOLLOWING_BLOG)?;
      blog.followers_count = blog.followers_count.checked_sub(1).ok_or(MSG_UNDERFLOW_UNFOLLOWING_BLOG)?;

      <SocialAccountById<T>>::insert(follower.clone(), social_account);
      <BlogById<T>>::insert(blog_id, blog);

      Self::deposit_event(RawEvent::BlogUnfollowed(follower.clone(), blog_id));
    }

    fn follow_account(origin, account: T::AccountId) {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, MSG_ACCOUNT_CANNOT_FOLLOW_ITSELF);
      ensure!(!<AccountFollowedByAccount<T>>::exists((follower.clone(), account.clone())), MSG_ACCOUNT_IS_ALREADY_FOLLOWED);

      let mut follower_account = Self::get_or_new_social_account(follower.clone());
      let mut followed_account = Self::get_or_new_social_account(account.clone());

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_add(1).ok_or(MSG_OVERFLOW_FOLLOWING_ACCOUNT)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_add(1).ok_or(MSG_OVERFLOW_FOLLOWING_ACCOUNT)?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);

      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |ids| ids.push(follower.clone()));
      <AccountFollowedByAccount<T>>::insert((follower.clone(), account.clone()), true);
      Self::deposit_event(RawEvent::AccountFollowed(follower, account));
    }

    fn unfollow_account(origin, account: T::AccountId) {
      let follower = ensure_signed(origin)?;

      ensure!(follower != account, MSG_ACCOUNT_CANNOT_UNFOLLOW_ITSELF);

      <AccountsFollowedByAccount<T>>::mutate(follower.clone(), |account_ids| Self::vec_remove_on(account_ids, account.clone()));
      <AccountFollowers<T>>::mutate(account.clone(), |account_ids| Self::vec_remove_on(account_ids, follower.clone()));
      <AccountFollowedByAccount<T>>::remove((follower.clone(), account.clone()));

      let mut follower_account = Self::social_account_by_id(follower.clone()).ok_or(MSG_FOLLOWER_ACCOUNT_NOT_FOUND)?;
      let mut followed_account = Self::social_account_by_id(account.clone()).ok_or(MSG_FOLLOWED_ACCOUNT_NOT_FOUND)?;

      follower_account.following_accounts_count = follower_account.following_accounts_count
        .checked_sub(1).ok_or(MSG_UNDERFLOW_UNFOLLOWING_ACCOUNT)?;
      followed_account.followers_count = followed_account.followers_count
        .checked_sub(1).ok_or(MSG_UNDERFLOW_UNFOLLOWING_ACCOUNT)?;

      <SocialAccountById<T>>::insert(follower.clone(), follower_account);
      <SocialAccountById<T>>::insert(account.clone(), followed_account);

      Self::deposit_event(RawEvent::AccountUnfollowed(follower, account));
    }

    // TODO use PostUpdate to pass data?
    pub fn create_post(origin, blog_id: T::BlogId, slug: Vec<u8>, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      let mut blog = Self::blog_by_id(blog_id).ok_or(MSG_BLOG_NOT_FOUND)?;

      ensure!(slug.len() >= Self::slug_min_len() as usize, MSG_POST_SLUG_IS_TOO_SHORT);
      ensure!(slug.len() <= Self::slug_max_len() as usize, MSG_POST_SLUG_IS_TOO_LONG);
      ensure!(!<PostIdBySlug<T>>::exists(slug.clone()), MSG_POST_SLUG_IS_NOT_UNIQUE);
      // TODO validate ipfs_hash

      let post_id = Self::next_post_id();
      let new_post: Post<T> = Post {
        id: post_id,
        blog_id,
        created: Self::new_change(owner.clone()),
        updated: None,
        slug: slug.clone(),
        ipfs_hash,
        comments_count: 0,
        upvotes_count: 0,
        downvotes_count: 0,
      };

      <PostById<T>>::insert(post_id, new_post);
      <PostIdsByBlogId<T>>::mutate(blog_id, |ids| ids.push(post_id));
      <PostIdBySlug<T>>::insert(slug, post_id);
      <NextPostId<T>>::mutate(|n| { *n += T::PostId::sa(1); });
      Self::deposit_event(RawEvent::PostCreated(owner.clone(), post_id));

      blog.posts_count += 1;
      <BlogById<T>>::insert(blog_id, blog); // TODO maybe use mutate instead of insert?
    }

    // TODO use CommentUpdate to pass data?
    fn create_comment(origin, post_id: T::PostId, parent_id: Option<T::CommentId>, ipfs_hash: Vec<u8>) {
      let owner = ensure_signed(origin)?;

      let mut post = Self::post_by_id(post_id).ok_or(MSG_POST_NOT_FOUND)?;

      if let Some(id) = parent_id {
        ensure!(<CommentById<T>>::exists(id), MSG_UNKNOWN_PARENT_COMMENT);
      }

      let comment_id = Self::next_comment_id();
      let new_comment: Comment<T> = Comment {
        id: comment_id,
        parent_id,
        post_id,
        created: Self::new_change(owner.clone()),
        updated: None,
        ipfs_hash,
        upvotes_count: 0,
        downvotes_count: 0,
      };

      <CommentById<T>>::insert(comment_id, new_comment);
      <CommentIdsByPostId<T>>::mutate(post_id, |ids| ids.push(comment_id));
      <NextCommentId<T>>::mutate(|n| { *n += T::CommentId::sa(1); });
      Self::deposit_event(RawEvent::CommentCreated(owner.clone(), comment_id));

      post.comments_count += 1;
      <PostById<T>>::insert(post_id, post); // TODO maybe use mutate instead of insert?
    }

    fn create_post_reaction(origin, post_id: T::PostId, kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      ensure!(
        !<PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        MSG_ACCOUNT_ALREADY_REACTED_TO_POST
      );

      let mut post = Self::post_by_id(post_id).ok_or(MSG_POST_NOT_FOUND)?;
      let reaction_id = Self::new_reaction(owner.clone(), kind.clone());

      <ReactionIdsByPostId<T>>::mutate(post_id, |ids| ids.push(reaction_id));
      <PostReactionIdByAccount<T>>::insert((owner.clone(), post_id), reaction_id);

      match kind {
        ReactionKind::Upvote => post.upvotes_count += 1,
        ReactionKind::Downvote => post.downvotes_count += 1,
      }
      // TODO maybe use mutate instead of insert?
      <PostById<T>>::insert(post_id, post);

      Self::deposit_event(RawEvent::PostReactionCreated(owner.clone(), post_id, reaction_id));
    }

    fn create_comment_reaction(origin, comment_id: T::CommentId, kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      ensure!(
        !<CommentReactionIdByAccount<T>>::exists((owner.clone(), comment_id)),
        MSG_ACCOUNT_ALREADY_REACTED_TO_COMMENT
      );

      let mut comment = Self::comment_by_id(comment_id).ok_or(MSG_COMMENT_NOT_FOUND)?;
      let reaction_id = Self::new_reaction(owner.clone(), kind.clone());

      <ReactionIdsByCommentId<T>>::mutate(comment_id, |ids| ids.push(reaction_id));
      <CommentReactionIdByAccount<T>>::insert((owner.clone(), comment_id), reaction_id);

      match kind {
        ReactionKind::Upvote => comment.upvotes_count += 1,
        ReactionKind::Downvote => comment.downvotes_count += 1,
      }
      // TODO maybe use mutate instead of insert?
      <CommentById<T>>::insert(comment_id, comment);

      Self::deposit_event(RawEvent::CommentReactionCreated(owner.clone(), comment_id, reaction_id));
    }

    pub fn update_blog(origin, blog_id: T::BlogId, update: BlogUpdate<T>) {
      let owner = ensure_signed(origin)?;
      
      let has_updates = 
        update.writers.is_some() ||
        update.slug.is_some() ||
        update.ipfs_hash.is_some();

      ensure!(has_updates, MSG_NOTHING_TO_UPDATE_IN_BLOG);

      let mut blog = Self::blog_by_id(blog_id).ok_or(MSG_BLOG_NOT_FOUND)?;

      // TODO ensure: blog writers also should be able to edit this blog:
      ensure!(owner == blog.created.account, MSG_ONLY_BLOG_OWNER_CAN_UPDATE_BLOG);

      let mut fields_updated = 0;

      if let Some(writers) = update.writers {
        if writers != blog.writers {
          // TODO validate writers.
          // TODO update BlogIdsByWriter: insert new, delete removed, update only changed writers.
          blog.writers = writers;
          fields_updated += 1;
        }
      }

      if let Some(slug) = update.slug {
        if slug != blog.slug {
          // TODO validate slug.
          ensure!(!<BlogIdBySlug<T>>::exists(slug.clone()), MSG_BLOG_SLUG_IS_NOT_UNIQUE);
          <BlogIdBySlug<T>>::remove(blog.slug);
          <BlogIdBySlug<T>>::insert(slug.clone(), blog_id);
          blog.slug = slug;
          fields_updated += 1;
        }
      }

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != blog.ipfs_hash {
          // TODO validate ipfs_hash
          blog.ipfs_hash = ipfs_hash;
          fields_updated += 1;
        }
      }

      // Update this blog only if at lest one field should be updated:
      if fields_updated > 0 {
        blog.updated = Some(Self::new_change(owner.clone()));
        <BlogById<T>>::insert(blog_id, blog);
        Self::deposit_event(RawEvent::BlogUpdated(owner.clone(), blog_id));
      }
    }
    
    pub fn update_post(origin, post_id: T::PostId, update: PostUpdate<T>) {
      let owner = ensure_signed(origin)?;
      
      let has_updates = 
        update.blog_id.is_some() ||
        update.slug.is_some() ||
        update.ipfs_hash.is_some();

      ensure!(has_updates, MSG_NOTHING_TO_UPDATE_IN_POST);

      let mut post = Self::post_by_id(post_id).ok_or(MSG_POST_NOT_FOUND)?;

      // TODO ensure: blog writers also should be able to edit this post:
      ensure!(owner == post.created.account, MSG_ONLY_POST_OWNER_CAN_UPDATE_POST);

      let mut fields_updated = 0;

      if let Some(slug) = update.slug {
        if slug != post.slug {
          // TODO validate slug.
          ensure!(!<PostIdBySlug<T>>::exists(slug.clone()), MSG_POST_SLUG_IS_NOT_UNIQUE);
          <PostIdBySlug<T>>::remove(post.slug);
          <PostIdBySlug<T>>::insert(slug.clone(), post_id);
          post.slug = slug;
          fields_updated += 1;
        }
      }

      if let Some(ipfs_hash) = update.ipfs_hash {
        if ipfs_hash != post.ipfs_hash {
          // TODO validate ipfs_hash
          post.ipfs_hash = ipfs_hash;
          fields_updated += 1;
        }
      }

      // Move this post to another blog:
      if let Some(blog_id) = update.blog_id {
        if blog_id != post.blog_id {
          Self::ensure_blog_exists(blog_id)?;
          
          // Remove post_id from its old blog:
          <PostIdsByBlogId<T>>::mutate(post.blog_id, |post_ids| Self::vec_remove_on(post_ids, post_id));
          
          // Add post_id to its new blog:
          <PostIdsByBlogId<T>>::mutate(blog_id.clone(), |ids| ids.push(post_id));
          post.blog_id = blog_id;
          fields_updated += 1;
        }
      }

      // Update this post only if at lest one field should be updated:
      if fields_updated > 0 {
        post.updated = Some(Self::new_change(owner.clone()));
        <PostById<T>>::insert(post_id, post);
        Self::deposit_event(RawEvent::PostUpdated(owner.clone(), post_id));
      }
    }
    
    fn update_comment(origin, comment_id: T::CommentId, update: CommentUpdate) {
      let owner = ensure_signed(origin)?;

      let mut comment = Self::comment_by_id(comment_id).ok_or(MSG_COMMENT_NOT_FOUND)?;
      ensure!(owner == comment.created.account, MSG_ONLY_COMMENT_AUTHOR_CAN_UPDATE_COMMENT);

      let ipfs_hash = update.ipfs_hash;
      ensure!(ipfs_hash != comment.ipfs_hash, MSG_NEW_COMMENT_HASH_DO_NOT_DIFFER);

      comment.ipfs_hash = ipfs_hash;
      comment.updated = Some(Self::new_change(owner.clone()));
      <CommentById<T>>::insert(comment_id, comment);
      Self::deposit_event(RawEvent::CommentUpdated(owner.clone(), comment_id));
    }

    fn update_post_reaction(origin, post_id: T::PostId, reaction_id: T::ReactionId, new_kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        MSG_ACCOUNT_HAS_NOT_REACTED_TO_POST
      );

      let mut reaction = Self::reaction_by_id(reaction_id).ok_or(MSG_REACTION_NOT_FOUND)?;
      ensure!(owner == reaction.created.account, MSG_ONLY_REACTION_OWNER_CAN_UPDATE_REACTION);
      ensure!(reaction.kind != new_kind, MSG_NEW_REACTION_KIND_DO_NOT_DIFFER);

      reaction.kind = new_kind;
      reaction.updated = Some(Self::new_change(owner.clone()));
      <ReactionById<T>>::insert(reaction_id, reaction);

      let mut post = Self::post_by_id(post_id).ok_or(MSG_POST_NOT_FOUND)?;
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
      // TODO maybe use mutate instead of insert?
      <PostById<T>>::insert(post_id, post);

      Self::deposit_event(RawEvent::PostReactionUpdated(owner.clone(), post_id, reaction_id));
    }

    fn update_comment_reaction(origin, comment_id: T::CommentId, reaction_id: T::ReactionId, new_kind: ReactionKind) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <CommentReactionIdByAccount<T>>::exists((owner.clone(), comment_id)),
        MSG_ACCOUNT_HAS_NOT_REACTED_TO_COMMENT
      );

      let mut reaction = Self::reaction_by_id(reaction_id).ok_or(MSG_REACTION_NOT_FOUND)?;
      ensure!(owner == reaction.created.account, MSG_ONLY_REACTION_OWNER_CAN_UPDATE_REACTION);
      ensure!(reaction.kind != new_kind, MSG_NEW_REACTION_KIND_DO_NOT_DIFFER);

      reaction.kind = new_kind;
      reaction.updated = Some(Self::new_change(owner.clone()));
      <ReactionById<T>>::insert(reaction_id, reaction);

      let mut comment = Self::comment_by_id(comment_id).ok_or(MSG_COMMENT_NOT_FOUND)?;
      match new_kind {
        ReactionKind::Upvote => {
          comment.upvotes_count += 1;
          comment.downvotes_count -= 1;
        },
        ReactionKind::Downvote => {
          comment.downvotes_count += 1;
          comment.upvotes_count -= 1;
        },
      }
      // TODO maybe use mutate instead of insert?
      <CommentById<T>>::insert(comment_id, comment);

      Self::deposit_event(RawEvent::CommentReactionUpdated(owner.clone(), comment_id, reaction_id));
    }

    // TODO fn delete_blog(origin, blog_id: T::BlogId) {
      // TODO only owner can delete
      // TODO unfollow all blog followers
    // }
    
    // TODO fn delete_post(origin, post_id: T::PostId) {}
    
    // TODO fn delete_comment(origin, comment_id: T::CommentId) {}

    fn delete_post_reaction(origin, post_id: T::PostId, reaction_id: T::ReactionId) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::exists((owner.clone(), post_id)),
        MSG_NO_POST_REACTION_BY_ACCOUNT_TO_DELETE
      );
      
      let reaction = Self::reaction_by_id(reaction_id).ok_or(MSG_REACTION_NOT_FOUND)?;
      ensure!(owner == reaction.created.account, MSG_ONLY_REACTION_OWNER_CAN_UPDATE_REACTION);

      <ReactionIdsByPostId<T>>::mutate(post_id, |ids| Self::vec_remove_on(ids, reaction_id));

      let mut post = Self::post_by_id(post_id).ok_or(MSG_POST_NOT_FOUND)?;
      match reaction.kind {
        ReactionKind::Upvote => post.upvotes_count -= 1,
        ReactionKind::Downvote => post.downvotes_count -= 1,
      }
      // TODO maybe use mutate instead of insert?
      <PostById<T>>::insert(post_id, post);

      <ReactionById<T>>::remove(reaction_id);
      <PostReactionIdByAccount<T>>::remove((owner.clone(), post_id));

      Self::deposit_event(RawEvent::PostReactionDeleted(owner.clone(), post_id, reaction_id));
    }

    fn delete_comment_reaction(origin, comment_id: T::CommentId, reaction_id: T::ReactionId) {
      let owner = ensure_signed(origin)?;

      ensure!(
        <CommentReactionIdByAccount<T>>::exists((owner.clone(), comment_id)),
        MSG_NO_COMMENT_REACTION_BY_ACCOUNT_TO_DELETE
      );
      
      let reaction = Self::reaction_by_id(reaction_id).ok_or(MSG_REACTION_NOT_FOUND)?;
      ensure!(owner == reaction.created.account, MSG_ONLY_REACTION_OWNER_CAN_UPDATE_REACTION);

      <ReactionIdsByCommentId<T>>::mutate(comment_id, |ids| Self::vec_remove_on(ids, reaction_id));
      
      let mut comment = Self::comment_by_id(comment_id).ok_or(MSG_COMMENT_NOT_FOUND)?;
      match reaction.kind {
        ReactionKind::Upvote => comment.upvotes_count -= 1,
        ReactionKind::Downvote => comment.downvotes_count -= 1,
      }
      // TODO maybe use mutate instead of insert?
      <CommentById<T>>::insert(comment_id, comment);

      <ReactionById<T>>::remove(reaction_id);
      <CommentReactionIdByAccount<T>>::remove((owner.clone(), comment_id));

      Self::deposit_event(RawEvent::CommentReactionDeleted(owner.clone(), comment_id, reaction_id));
    }

    // TODO spend some tokens on: create/update a blog/post/comment.
  }
}

impl<T: Trait> Module<T> {

  fn ensure_blog_exists(blog_id: T::BlogId) -> dispatch::Result {
    ensure!(<BlogById<T>>::exists(blog_id), MSG_BLOG_NOT_FOUND);
    Ok(())
  }

  fn new_change(account: T::AccountId) -> Change<T> {
    Change {
      account,
      block: <system::Module<T>>::block_number(),
      time: <timestamp::Module<T>>::now(),
    }
  }

  fn new_reaction(account: T::AccountId, kind: ReactionKind) -> T::ReactionId {
    let reaction_id = Self::next_reaction_id();
    let new_reaction: Reaction<T> = Reaction {
      id: reaction_id,
      created: Self::new_change(account),
      updated: None,
      kind
    };

    <ReactionById<T>>::insert(reaction_id, new_reaction);
    <NextReactionId<T>>::mutate(|n| { *n += T::ReactionId::sa(1); });

    reaction_id
  }

  fn add_blog_follower_and_insert_blog(
    follower: T::AccountId,
    blog_id: T::BlogId,
    mut blog: Blog<T>,
    is_new_blog: bool
  ) -> dispatch::Result {

    let mut social_account = Self::get_or_new_social_account(follower.clone());
    social_account.following_blogs_count = social_account.following_blogs_count
      .checked_add(1)
      .ok_or(MSG_OVERFLOW_FOLLOWING_BLOG)?;

    <SocialAccountById<T>>::insert(follower.clone(), social_account);

    blog.followers_count = blog.followers_count.checked_add(1).ok_or(MSG_OVERFLOW_FOLLOWING_BLOG)?;
    <BlogById<T>>::insert(blog_id, blog);
    if is_new_blog {
      Self::deposit_event(RawEvent::BlogCreated(follower.clone(), blog_id));
    }

    <BlogsFollowedByAccount<T>>::mutate(follower.clone(), |ids| ids.push(blog_id));
    <BlogFollowers<T>>::mutate(blog_id, |ids| ids.push(follower.clone()));
    <BlogFollowedByAccount<T>>::insert((follower.clone(), blog_id), true);

    Self::deposit_event(RawEvent::BlogFollowed(follower, blog_id));
    Ok(())
  }

  fn get_or_new_social_account(account: T::AccountId) -> SocialAccount {
    if let Some(social_account) = Self::social_account_by_id(account) {
      social_account
    } else {
      SocialAccount {
        followers_count: 0,
        following_accounts_count: 0,
        following_blogs_count: 0
      }
    }
  }

  fn vec_remove_on<F: PartialEq>(vector: &mut Vec<F>, element: F) {
    if let Some(index) = vector.iter().position(|x| *x == element) {
      vector.swap_remove(index);
    }
  }
}

#![cfg(test)]

use super::mock::*;
use super::blogs::*;

use runtime_io::with_externalities;
use srml_support::*;

const ACCOUNT1 : u64 = 1;
const ACCOUNT2 : u64 = 2;

fn blog_slug() -> Vec<u8> {
  b"blog_slug".to_vec()
}

fn blog_json() -> Vec<u8> {
  b"{\"name\":\"Blog name\",\"desc\":\"Blog content\",\"image\":\"\",\"tags\":[]}".to_vec()
}

fn blog_update(writers: Option<Vec<u64>>, slug: Option<Vec<u8>>, json: Option<Vec<u8>>) -> BlogUpdate<Test> {
  BlogUpdate {
    writers,
    slug,
    json
  }
}

fn post_slug() -> Vec<u8> {
  b"post_slug".to_vec()
}

fn post_json() -> Vec<u8> {
  b"{\"title\":\"Post name\",\"body\":\"Post content\",\"image\":\"\",\"tags\":[]}".to_vec()
}

fn post_update(blog_id: Option<u32>, slug: Option<Vec<u8>>, json: Option<Vec<u8>>) -> PostUpdate<Test> {
  PostUpdate {
    blog_id,
    slug,
    json
  }
}

fn _create_default_blog() -> dispatch::Result {
  _create_blog(None, None, None)
}

fn _create_blog(origin: Option<Origin>, slug: Option<Vec<u8>>, json: Option<Vec<u8>>) -> dispatch::Result {
  Blogs::create_blog(
    origin.unwrap_or(Origin::signed(ACCOUNT1)),
    slug.unwrap_or(self::blog_slug()),
    json.unwrap_or(self::blog_json())
  )
}

fn _update_blog(origin: Option<Origin>, blog_id: Option<u32>, update: Option<BlogUpdate<Test>>) -> dispatch::Result {
  Blogs::update_blog(
    origin.unwrap_or(Origin::signed(ACCOUNT1)),
    blog_id.unwrap_or(1),
    update.unwrap_or(self::blog_update(None, None, None))
  )
}

fn _create_default_post() -> dispatch::Result {
  _create_post(None, None, None, None)
}

fn _create_post(origin: Option<Origin>, blog_id: Option<u32>, slug: Option<Vec<u8>>, json: Option<Vec<u8>>) -> dispatch::Result {
  Blogs::create_post(
    origin.unwrap_or(Origin::signed(ACCOUNT1)),
    blog_id.unwrap_or(1),
    slug.unwrap_or(self::post_slug()),
    json.unwrap_or(self::post_json())
  )
}

// pub fn update_post(origin, post_id: T::PostId, update: PostUpdate<T>)
fn _update_post(origin: Option<Origin>, post_id: Option<u32>, update: Option<PostUpdate<Test>>) -> dispatch::Result {
  Blogs::update_post(
    origin.unwrap_or(Origin::signed(ACCOUNT1)),
    post_id.unwrap_or(1),
    update.unwrap_or(self::post_update(None, None, None))
  )
}

// Blog tests
#[test]
fn create_blog_should_work() {
  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());

    // Check whether blog ID is added into an ownership of the account
    assert_eq!(Blogs::blog_ids_by_owner(ACCOUNT1), vec![1]);
    // Check whether we can find blog ID by it's slug
    assert_eq!(Blogs::blog_id_by_slug(self::blog_slug()), Some(1));
    // Check whether NextBlogId changes correctly
    assert_eq!(Blogs::next_blog_id(), 2);

    // Check whether something is written in BlogById by the blog ID
    assert_eq!(Blogs::blog_by_id(1).is_some(), true);

    // TODO check for expected blog
  });
}

#[test]
fn create_blog_should_fail_short_slug() {
  // Initialize a slug var with 1 symbol 'a' in vector
  let slug : Vec<u8> = vec![97];

  with_externalities(&mut build_ext(), || {
    // Try to catch an error creating a blog with too short slug
    assert_noop!(_create_blog(None, Some(slug), None), MSG_BLOG_SLUG_IS_TOO_SHORT);
  });
}

#[test]
fn create_blog_should_fail_long_slug() {
  // Initialize a slug var with 51 symbols 'a' in vector
  let slug : Vec<u8> = vec![97; 51];

  with_externalities(&mut build_ext(), || {
    // Try to catch an error creating a blog with too long slug
    assert_noop!(_create_blog(None, Some(slug), None), MSG_BLOG_SLUG_IS_TOO_LONG);
  });
}

#[test]
fn create_blog_should_fail_not_unique_slug() {

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Try to catch an error creating a blog with not unique slug
    assert_noop!(_create_default_blog(), MSG_BLOG_SLUG_IS_NOT_UNIQUE);
  });
}

#[test]
fn create_blog_should_fail_long_json() {
  // Initialize a json var with 1001 symbols 'a' in vector
  let json : Vec<u8> = vec![97; 1001];

  with_externalities(&mut build_ext(), || {
    // Try to catch an error creating a blog with too long json
    assert_noop!(_create_blog(None, None, Some(json)), MSG_BLOG_JSON_IS_TOO_LONG);
  });
}

#[test]
fn update_blog_should_work() {
  // Initialize a custom slug
  let slug : Vec<u8> = String::from("new_slug").as_bytes().to_vec();

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());

    // Blog update with ID 1 should be fine
    assert_ok!(_update_blog(None, None,
      Some(
        self::blog_update(
          None,
          Some(slug),
          None
        )
      )
    ));
  });
}

#[test]
fn update_blog_should_fail_nothing_to_update() {
  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
  
    // Try to catch an error updating a blog with no changes
    assert_noop!(_update_blog(None, None, None), MSG_NOTHING_TO_UPDATE_IN_BLOG);
  });
}

#[test]
fn update_blog_should_fail_blog_not_found() {
  // Initialize a custom slug
  let slug : Vec<u8> = String::from("new_slug").as_bytes().to_vec();

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
  
    // Try to catch an error updating a blog with wrong blog ID
    assert_noop!(_update_blog(None, Some(2),
      Some(
        self::blog_update(
          None, 
          Some(slug),
          None
        )
      )
    ), MSG_BLOG_NOT_FOUND);
  });
}

#[test]
fn update_blog_should_fail_not_an_owner() {
  // Initialize a custom slug
  let slug : Vec<u8> = String::from("new_slug").as_bytes().to_vec();

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
  
    // Try to catch an error updating a blog with different account
    assert_noop!(_update_blog(Some(Origin::signed(ACCOUNT2)), None,
      Some(
        self::blog_update(
          None, 
          Some(slug),
          None
        )
      )
    ), MSG_ONLY_BLOG_OWNER_CAN_UPDATE_BLOG);
  });
}

#[test]
fn update_blog_should_fail_short_slug() {
  // Initialize a slug var with 1 symbol 'a' in vector
  let slug : Vec<u8> = vec![97];

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
  
    // Try to catch an error updating a blog with too short slug
    assert_noop!(_update_blog(None, None,
      Some(
        self::blog_update(
          None, 
          Some(slug),
          None
        )
      )
    ), MSG_BLOG_SLUG_IS_TOO_SHORT);
  });
}

#[test]
fn update_blog_should_fail_long_slug() {
  // Initialize a slug var with 51 symbols 'a' in vector
  let slug : Vec<u8> = vec![97; 51];

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
  
    // Try to catch an error updating a blog with too long slug
    assert_noop!(_update_blog(None, None,
      Some(
        self::blog_update(
          None, 
          Some(slug),
          None
        )
      )
    ), MSG_BLOG_SLUG_IS_TOO_LONG);
  });
}

#[test]
fn update_blog_should_fail_not_unique_slug() {
  // Initialize a variable with custom slug
  let slug : Vec<u8> = String::from("unique_slug").as_bytes().to_vec();

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Create a blog with ID 2 and a custom slug
    assert_ok!(_create_blog(
      None,
      Some(slug.clone()),
      None
    ));
  
    // Try to catch an error updating a blog on ID 1 with a slug of blog on ID 2
    assert_noop!(_update_blog(None, Some(1),
      Some(
        self::blog_update(
          None, 
          Some(slug),
          None
        )
      )
    ), MSG_BLOG_SLUG_IS_NOT_UNIQUE);
  });
}

#[test]
fn update_blog_should_fail_long_json() {
  // Initialize a json var with 1001 symbols 'a' in vector
  let json : Vec<u8> = vec![97; 1001];

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
  
    // Try to catch an error updating a blog with too long json
    assert_noop!(_update_blog(None, None,
      Some(
        self::blog_update(
          None, 
          None,
          Some(json)
        )
      )
    ), MSG_BLOG_JSON_IS_TOO_LONG);
  });
}
// TODO blog writers tests

// Post tests
#[test]
fn create_post_should_work() {
  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Create a post with ID 1 on a blog
    assert_ok!(_create_default_post());

    // Check whether post ID is added into blog posts list
    assert_eq!(Blogs::post_ids_by_blog_id(1), vec![1]);
    // Check whether we can find post ID by it's slug
    assert_eq!(Blogs::post_id_by_slug(self::post_slug()), Some(1));
    // Check whether NextPostId changes correctly
    assert_eq!(Blogs::next_post_id(), 2);

    // Check whether something is written in PostById by the post ID
    assert_eq!(Blogs::post_by_id(1).is_some(), true);

    // TODO check for expected post
  });
}

#[test]
fn create_post_should_fail_short_slug() {
  // Initialize a slug var with 1 symbol 'a' in vector
  let slug : Vec<u8> = vec![97];

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());

    // Try to catch an error creating a post with too short slug
    assert_noop!(_create_post(None, None, Some(slug), None), MSG_POST_SLUG_IS_TOO_SHORT);
  });
}

#[test]
fn create_post_should_fail_long_slug() {
  // Initialize a slug var with 51 symbols 'a' in vector
  let slug : Vec<u8> = vec![97; 51];

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());

    // Try to catch an error creating a post with too long slug
    assert_noop!(_create_post(None, None, Some(slug), None), MSG_POST_SLUG_IS_TOO_LONG);
  });
}

#[test]
fn create_post_should_fail_not_unique_slug() {

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Create a post with ID 1
    assert_ok!(_create_default_post());

    // Try to catch an error creating a post with not unique slug
    assert_noop!(_create_default_post(), MSG_POST_SLUG_IS_NOT_UNIQUE);
  });
}

#[test]
fn create_post_should_fail_long_json() {
  // Initialize a json var with 10001 symbols 'a' in vector
  let json : Vec<u8> = vec![97; 10001];

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());

    // Try to catch an error creating a post with too long json
    assert_noop!(_create_post(None, None, None, Some(json)), MSG_POST_JSON_IS_TOO_LONG);
  });
}

#[test]
fn update_post_should_work() {
  // Initialize a custom slug
  let slug : Vec<u8> = String::from("new_slug").as_bytes().to_vec();

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Create a post with ID 1
    assert_ok!(_create_default_post());

    // Post update with ID 1 should be fine
    assert_ok!(_update_post(None, None,
      Some(
        self::post_update(
          None,
          Some(slug),
          None
        )
      )
    ));
  });
}

#[test]
fn update_post_should_fail_nothing_to_update() {
  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Create a post with ID 1
    assert_ok!(_create_default_post());
  
    // Try to catch an error updating a post with no changes
    assert_noop!(_update_post(None, None, None), MSG_NOTHING_TO_UPDATE_IN_POST);
  });
}

#[test]
fn update_post_should_fail_post_not_found() {
  // Initialize a custom slug
  let slug : Vec<u8> = String::from("new_slug").as_bytes().to_vec();

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Create a post with ID 1
    assert_ok!(_create_default_post());
  
    // Try to catch an error updating a post with wrong post ID
    assert_noop!(_update_post(None, Some(2),
      Some(
        self::post_update(
          None, 
          Some(slug),
          None
        )
      )
    ), MSG_POST_NOT_FOUND);
  });
}

#[test]
fn update_post_should_fail_not_an_owner() {
  // Initialize a custom slug
  let slug : Vec<u8> = String::from("new_slug").as_bytes().to_vec();

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Create a post with ID 1
    assert_ok!(_create_default_post());
  
    // Try to catch an error updating a post with different account
    assert_noop!(_update_post(Some(Origin::signed(ACCOUNT2)), None,
      Some(
        self::post_update(
          None, 
          Some(slug),
          None
        )
      )
    ), MSG_ONLY_POST_OWNER_CAN_UPDATE_POST);
  });
}

#[test]
fn update_post_should_fail_short_slug() {
  // Initialize a slug var with 1 symbol 'a' in vector
  let slug : Vec<u8> = vec![97];

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Create a post with ID 1
    assert_ok!(_create_default_post());
  
    // Try to catch an error updating a post with too short slug
    assert_noop!(_update_post(None, None,
      Some(
        self::post_update(
          None, 
          Some(slug),
          None
        )
      )
    ), MSG_POST_SLUG_IS_TOO_SHORT);
  });
}

#[test]
fn update_post_should_fail_long_slug() {
  // Initialize a slug var with 51 symbols 'a' in vector
  let slug : Vec<u8> = vec![97; 51];

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Create a post with ID 1
    assert_ok!(_create_default_post());
  
    // Try to catch an error updating a post with too long slug
    assert_noop!(_update_post(None, None,
      Some(
        self::post_update(
          None, 
          Some(slug),
          None
        )
      )
    ), MSG_POST_SLUG_IS_TOO_LONG);
  });
}

#[test]
fn update_post_should_fail_not_unique_slug() {
  // Initialize a variable with custom slug
  let slug : Vec<u8> = String::from("unique_slug").as_bytes().to_vec();

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Create a post with ID 1
    assert_ok!(_create_default_post());
    // Create a post with ID 2 and a custom slug
    assert_ok!(_create_post(
      None,
      None,
      Some(slug.clone()),
      None
    ));
  
    // Try to catch an error updating a post on ID 1 with a slug of post on ID 2
    assert_noop!(_update_post(None, Some(1),
      Some(
        self::post_update(
          None, 
          Some(slug),
          None
        )
      )
    ), MSG_POST_SLUG_IS_NOT_UNIQUE);
  });
}

#[test]
fn update_post_should_fail_long_json() {
  // Initialize a json var with 10001 symbols 'a' in vector
  let json : Vec<u8> = vec![97; 10001];

  with_externalities(&mut build_ext(), || {
    // Create a blog with ID 1
    assert_ok!(_create_default_blog());
    // Create a post with ID 1
    assert_ok!(_create_default_post());
  
    // Try to catch an error updating a post with too long json
    assert_noop!(_update_post(None, None,
      Some(
        self::post_update(
          None, 
          None,
          Some(json)
        )
      )
    ), MSG_POST_JSON_IS_TOO_LONG);
  });
}


// Comment tests
// TODO Reaction tests

// TODO Blog (un)follow tests
// TODO Account (un)follow tests
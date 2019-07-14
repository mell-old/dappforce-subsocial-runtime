#![cfg(test)]

use super::mock::*;

use runtime_io::with_externalities;
use srml_support::*;

const ACCOUNT1 : u64 = 1;

fn blog_slug() -> Vec<u8> {
  b"blog_slug".to_vec()
}

fn blog_json() -> Vec<u8> {
  b"{\"name\":\"Blog name\",\"desc\":\"Blog content\",\"image\":\"\",\"tags\":[]}".to_vec()
}

fn _create_default_blog() -> dispatch::Result {
  _create_blog(None, None, None)
}

fn _create_blog(origin: Option<u64>, slug: Option<Vec<u8>>, json: Option<Vec<u8>>) -> dispatch::Result {
  Blogs::create_blog(
    Origin::signed(origin.unwrap_or(ACCOUNT1)),
    slug.unwrap_or(self::blog_slug()),
    json.unwrap_or(self::blog_json())
  )
}

// Blog tests
#[test]
fn create_blog_should_work() {
  with_externalities(&mut build_ext(), || {
    assert_ok!(_create_default_blog());
    assert_eq!(Blogs::blog_ids_by_owner(ACCOUNT1), vec![1]);
    assert_eq!(Blogs::blog_id_by_slug(self::blog_slug()), Some(1));
    assert_eq!(Blogs::next_blog_id(), 2);

    assert_eq!(Blogs::blog_by_id(1).is_some(), true);
    // TODO check for expected blog
  });
}

#[test]
fn create_blog_should_fail_short_slug() {
  let slug : Vec<u8> = vec![97];

  with_externalities(&mut build_ext(), || {
    assert_noop!(_create_blog(None, Some(slug), None), "Blog slug is too short");
  });
}

#[test]
fn create_blog_should_fail_long_slug() {
  let slug : Vec<u8> = vec![97; 51];

  with_externalities(&mut build_ext(), || {
    assert_noop!(_create_blog(None, Some(slug), None), "Blog slug is too long");
  });
}

#[test]
fn create_blog_should_fail_not_unique_slug() {

  with_externalities(&mut build_ext(), || {
    assert_ok!(_create_default_blog());
    assert_noop!(_create_default_blog(), "Blog slug is not unique");
  });
}

#[test]
fn create_blog_should_fail_long_json() {
  let json : Vec<u8> = vec![97; 1001];

  with_externalities(&mut build_ext(), || {
    assert_noop!(_create_blog(None, None, Some(json)), "Blog JSON is too long");
  });
}

#[test]
fn update_blog_should_work() {
  with_externalities(&mut build_ext(), || {
    // TODO use BlogUpdate struct
  });
}

#[test]
fn update_blog_should_fail_nothing_to_update() {
  
}

#[test]
fn update_blog_should_fail_blog_not_found() {
  
}

#[test]
fn update_blog_should_fail_not_an_owner() {
  
}

#[test]
fn update_blog_should_fail_short_slug() {
  
}

#[test]
fn update_blog_should_fail_long_slug() {
  
}

#[test]
fn update_blog_should_fail_not_unique_slug() {
  
}

#[test]
fn update_blog_should_fail_long_json() {

}

// Post tests
#[test]
fn create_post_should_work() {

}

#[test]
fn create_post_fail_blog_not_found() {

}

#[test]
fn create_post_fail_short_slug() {

}

#[test]
fn create_post_fail_long_slug() {

}

#[test]
fn create_post_fail_not_unique_slug() {

}

#[test]
fn create_post_fail_long_json() {

}

#[test]
fn update_post_should_work() {
  with_externalities(&mut build_ext(), || {
    // TODO use PostUpdate struct
  });
}

#[test]
fn update_post_should_fail_nothing_to_update() {
  
}

#[test]
fn update_post_should_fail_blog_not_found() {
  
}

#[test]
fn update_post_should_fail_not_an_owner() {
  
}

#[test]
fn update_post_should_fail_short_slug() {
  
}

#[test]
fn update_post_should_fail_long_slug() {
  
}

#[test]
fn update_post_should_fail_not_unique_slug() {
  
}

#[test]
fn update_post_should_fail_long_json() {

}

// TODO Comment tests
// TODO Reaction tests
// TODO Blog (un)follow tests
// TODO Account (un)follow tests
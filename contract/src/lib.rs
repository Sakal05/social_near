use std::num::Wrapping;
use std::u8;

// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::{ near_bindgen, AccountId, env, Balance, Promise };
use near_sdk::serde::{ Serialize, Deserialize };
use uuid::Uuid;
use near_sdk::json_types::U128;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Post {
    pub id: String,
    pub author: AccountId, //account id
    pub title: String,
    pub body: String,
    //add donation information
    pub donation_amount: U128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Posts {
    pub posts: Vec<Post>,
}

#[near_bindgen]
impl Posts {
    pub fn new() -> Self {
        Self {
            posts: Vec::new(),
        }
    }

    //function to create a new post
    pub fn new_post(&mut self, title: String, body: String) {
        self.posts.push(Post {
            id: Uuid::new_v4().to_string(),
            author: env::predecessor_account_id(),
            title,
            body,
            donation_amount: U128::from(0),
        });
        env::log_str("Post Created Successfully");
    }

    //function to get all posts
    pub fn get_posts(&self) -> Vec<Post> {
        self.posts.clone()
    }

    //function to search for posts
    pub fn search_posts(&self, search_string: String) -> Vec<Post> {
        let search_result = self.posts
            .iter()
            .filter(|post| post.title.contains(&search_string))
            .cloned()
            .collect();
        return search_result;
    }

    //function to delete a post
    pub fn delete_post(&mut self, post_id: String) {
        self.posts.retain(|post| post.id != post_id);
    }

    //function to donate a author of the post
    #[payable]
    pub fn donate_author(&mut self, post_id: String, amount: U128) {
        match self.posts.iter_mut().find(|post| post.id == post_id) {
            Some(post) => {
                let donor: AccountId = env::signer_account_id();
                let receiver_accound_id = &post.author;
                let amount_transfer: Balance = amount.into();
                Promise::new(receiver_accound_id.clone()).transfer(amount_transfer);
            }
            None => {
                // post not found
                env::log_str(&format!("Couldn't find post '{}'", post_id));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn new_post_with_title() {
        let mut post = Posts::new();
        post.new_post("title".to_string(), "body".to_string());
        post.new_post("title 1".to_string(), "body 1".to_string());
        assert_eq!(post.posts.len(), 2);
    }

    //testing to get all posts
    #[test]
    pub fn get_posts() {
        let mut post = Posts::new();
        post.new_post("title".to_string(), "body".to_string());
        post.new_post("title 1".to_string(), "body 1".to_string());
        let posts = post.get_posts();
        println!("Id: {}, Author: {}", posts[0].id, posts[0].author);
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].body, "body".to_string());
    }

    //test search post function
    #[test]
    pub fn search_posts() {
        let mut post = Posts::new();
        post.new_post("title".to_string(), "body".to_string());
        post.new_post("title 1".to_string(), "body 1".to_string());
        let posts = post.search_posts("title".to_string());
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[1].body, "body 1".to_string());
        println!("{:?}", posts);
    }

    //test delete posts
    #[test]
    pub fn delete_post() {
        let mut post = Posts::new();
        post.new_post("title".to_string(), "body".to_string());
        post.new_post("title 1".to_string(), "body 1".to_string());
        post.delete_post(post.posts[0].id.to_string());
        let posts = post.get_posts();
        assert_eq!(posts.len(), 1);
        println!("{:?}", posts);
    }

    //test donate function
    #[test]
    pub fn donate_author() {
        let mut post = Posts::new();
        post.new_post("title".to_string(), "body".to_string());
        post.new_post("title 1".to_string(), "body 1".to_string());
        post.donate_author(post.posts[0].id.to_string(), U128::from(100));
        
        
    }
}
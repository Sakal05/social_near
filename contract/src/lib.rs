use std::u8;
// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::{ near_bindgen, AccountId, env, Balance, Promise };
use near_sdk::serde::{ Serialize, Deserialize };
use uuid::Uuid;
use near_sdk::json_types::U128;
use ipfs_api::{ IpfsClient, IpfsApi };
use std::io::Cursor;

//for testing purpose 
const IMAGE: &str = "https://static.vecteezy.com/packs/media/vectors/term-bg-1-3d6355ab.jpg";

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Post {
    pub id: String,
    pub author: AccountId, //account id
    pub title: String,
    pub body: String,
    //add image variable as optional parameter
    pub image: Option<String>,
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
    pub fn new_post(&mut self, title: String, body: String, image: Option<String>) {
        let image_to_ipfs = write_image_to_ipfs(image.unwrap_or_default());
        let image_hash = match image_to_ipfs {
            Ok(hash) => Some(hash.clone()),
            Err(err) => None
        };
        self.posts.push(Post {
            id: Uuid::new_v4().to_string(),
            author: env::predecessor_account_id(),
            title,
            body,
            image: image_hash,
            donation_amount: U128::from(0),
        });
        env::log_str("Post Created Successfully");
        env::log_str(&format!("{:?}", self));

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
                let mut total_donation: u128 = post.donation_amount.clone().into();
                total_donation += amount_transfer;
                Promise::new(receiver_accound_id.clone()).transfer(amount_transfer);
                post.donation_amount = U128::from(total_donation);
            }
            None => {
                // post not found
                env::log_str(&format!("Couldn't find post '{}'", post_id));
            }
        }
    }

    //function to get all donations from the post
    pub fn get_donations(&mut self, post_id: String) -> Option<u128> {
        match self.posts.iter().find(|post| post.id == post_id) {
            Some(post) => Some(post.donation_amount.into()),
            None => None,
        }
    }
}

//function to write image into ipfs
fn write_image_to_ipfs(image_url: String) -> Result<String, ipfs_api::Error> {
    let client = IpfsClient::default();
    let data = Cursor::new(image_url);
    let res = client.add(data);
    let res = tokio::runtime::Runtime::new().unwrap().block_on(res);
    match res {
        Ok(res) => Ok(res.hash),
        Err(e) => Err(e)
    }
}


#[cfg(test)]
mod test_ipfs {
    use std::io::Cursor;
    use ipfs_api::{ IpfsClient, IpfsApi };
    use crate::IMAGE;
    //test write image to ipfs
    #[test]
    fn test_write_image_to_ipfs() {
        let client = IpfsClient::default();
        let data = Cursor::new(IMAGE.to_string());
        let res = client.add(data);
        let res = tokio::runtime::Runtime::new().unwrap().block_on(res);
        match res {
            Ok(res) => println!("Hash value: {}", res.hash),
            Err(e) => println!("Error: {}", e),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    //for testing purposes
    use crate::IMAGE;

    #[test]
    pub fn new_post_with_title() {
        let mut post = Posts::new();
        post.new_post("title".to_string(), "body".to_string(), Some(IMAGE.to_string()));
        post.new_post("title 1".to_string(), "body 1".to_string(), Some(IMAGE.to_string()));
        assert_eq!(post.posts.len(), 2);
    }

    //testing to get all posts
    #[test]
    pub fn get_posts() {
        let mut post = Posts::new();
        post.new_post("title".to_string(), "body".to_string(), Some(IMAGE.to_string()));
        post.new_post("title 1".to_string(), "body 1".to_string(), Some(IMAGE.to_string()));
        let posts = post.get_posts();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].body, "body".to_string());
    }

    //test search post function
    #[test]
    pub fn search_posts() {
        let mut post = Posts::new();
        post.new_post("title".to_string(), "body".to_string(), Some(IMAGE.to_string()));
        post.new_post("title 1".to_string(), "body 1".to_string(), Some(IMAGE.to_string()));
        let posts = post.search_posts("title".to_string());
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[1].body, "body 1".to_string());
    }

    //test delete posts
    #[test]
    pub fn delete_post() {
        let mut post = Posts::new();
        post.new_post("title".to_string(), "body".to_string(), Some(IMAGE.to_string()));
        post.new_post("title 1".to_string(), "body 1".to_string(), Some(IMAGE.to_string()));
        post.delete_post(post.posts[0].id.to_string());
        let posts = post.get_posts();
        assert_eq!(posts.len(), 1);
    }

    //test success donate function
    #[test]
    pub fn sucess_donate_author() {
        let mut post = Posts::new();
        post.new_post("title".to_string(), "body".to_string(), Some(IMAGE.to_string()));
        post.new_post("title 1".to_string(), "body 1".to_string(), Some(IMAGE.to_string()));
        let donate1 = post.donate_author(post.posts[0].id.to_string(), U128::from(100));
        let donate2 = post.donate_author(post.posts[0].id.to_string(), U128::from(100));
        let donate3 = post.donate_author(post.posts[0].id.to_string(), U128::from(100));
        assert_eq!(post.posts[0].donation_amount, U128::from(300));
    }

    //test fail donate function
    #[test]
    pub fn fail_donate_author() {
        let mut post = Posts::new();
        post.new_post("title".to_string(), "body".to_string(), None);
        post.new_post("title 1".to_string(), "body 1".to_string(), None);
        let donate1 = post.donate_author(post.posts[0].id.to_string(), U128::from(100));
        let donate2 = post.donate_author(post.posts[0].id.to_string(), U128::from(100));
        let donate3 = post.donate_author(post.posts[0].id.to_string(), U128::from(100));
        assert_ne!(post.posts[0].donation_amount, U128::from(400));
    }
}
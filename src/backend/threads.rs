use jmap_client::{client::Client, thread::Thread};

pub struct Threads {
    threads: Vec<Thread>,
    state: String,
}

impl Threads {
    pub async fn new(client: &Client) -> Self {
        todo!()
    }
}

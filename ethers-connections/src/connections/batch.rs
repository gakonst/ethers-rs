use std::cell::RefCell;

use tokio::sync::oneshot;

use crate::Connection;

pub struct Batch<C> {
    requests: RefCell<Vec<(u64, String)>>,
    connection: C,
}

impl<C: Connection> Batch<C> {
    pub async fn send_batch(&mut self) -> Result<(), ()> {
        todo!()
    }
}

impl<C: Connection> Connection for Batch<C> {
    fn request_id(&self) -> u64 {
        self.connection.request_id()
    }

    fn send_raw_request(&self, id: u64, request: String) -> crate::RequestFuture<'_> {
        self.requests.borrow_mut().push((id, request));

        let (tx, rx) = oneshot::channel();
        Box::pin(async move {
            let raw = rx.try_recv().expect("TODO");
            todo!()
        })
    }
}

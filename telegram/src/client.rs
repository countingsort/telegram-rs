use futures::{Future, Stream};
use hyper::client::HttpConnector;
use hyper::{self, Body};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_mtproto::Identifiable;
use tokio::reactor::Core;

use error;
use request::Request;
use response::Response;

pub struct Client {
    core: Core,
    http_client: hyper::Client<HttpConnector, Body>,
}

impl Client {
    /// Create a new Telegram client.
    #[inline]
    pub fn new() -> error::Result<Client> {
        let core = Core::new()?;
        let http_client = hyper::Client::new(&core.handle());

        Ok(Client {
            core: core,
            http_client: http_client,
        })
    }

    // Send a constructed request using this Client.
    pub fn send<F, T, U, R>(&mut self, req: Request<T>, on_receive_handler: F) -> error::Result<R>
    where
        F: FnOnce(Response<U>) -> R,
        T: Serialize + Identifiable,
        U: 'static + DeserializeOwned + Identifiable,
    {
        let http_request = req.to_http_request()?;

        let promise = Box::new(
            self.http_client
                .request(http_request)
                .and_then(|res| res.body().concat2())
                .map(|data| Response::from_reader(&*data))
                .flatten()
                .map_err(|err| err.into()),
        ).map(on_receive_handler);

        self.core.run(promise)
    }
}

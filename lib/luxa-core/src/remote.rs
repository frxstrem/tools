use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::*;
use crate::luxa::*;

pub struct RemoteHandle<T: RemoteTransport> {
    transport: T,
}

#[async_trait(?Send)]
impl<T: RemoteTransport> Luxafor for RemoteHandle<T> {
    async fn solid(&self, color: Color) -> Result<(), LuxaError> {
        self.transport.send(Request::Solid(color)).await?.ok()
    }

    async fn fade(&self, color: Color, duration: u8) -> Result<(), LuxaError> {
        self.transport
            .send(Request::Fade(color, duration))
            .await?
            .ok()
    }
}

pub struct Remote<L: Luxafor> {
    inner: L,
}

impl<L: Luxafor> Remote<L> {
    pub async fn handle(&self, request: Request) -> Result<Response, LuxaError> {
        match request {
            Request::Solid(color) => self.inner.solid(color).await.map(|_| Response::Ok),
            Request::Fade(color, duration) => {
                self.inner.fade(color, duration).await.map(|_| Response::Ok)
            }
        }
    }
}

#[async_trait(?Send)]
pub trait RemoteTransport {
    async fn send(&self, request: Request) -> Result<Response, LuxaError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Solid(Color),
    Fade(Color, u8),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Ok,
    Err(LuxaError),
}

impl Response {
    pub fn ok(self) -> Result<(), LuxaError> {
        match self {
            Response::Ok => Ok(()),
            Response::Err(err) => Err(err),
        }
    }
}

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
        let response = self.transport.send(Request::Solid(color)).await?;

        match response {
            Response::Ok => Ok(()),
        }
    }

    async fn fade(&self, color: Color, duration: u8) -> Result<(), LuxaError> {
        let response = self.transport.send(Request::Fade(color, duration)).await?;

        match response {
            Response::Ok => Ok(()),
        }
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

pub enum Request {
    Solid(Color),
    Fade(Color, u8),
}

pub enum Response {
    Ok,
}

use thiserror::Error;
use anyhow;

use winit::error::{EventLoopError, OsError};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    InitializationError(#[from] InitializationError),
    #[error(transparent)]
    StartLoopError(#[from] StartLoopError),
    #[error(transparent)]
    WindowCreationError(#[from] WindowCreationError),
    #[error(transparent)]
    WindowAccessError(#[from] WindowAccessError),
    #[error(transparent)]
    EventLoopProxyError(#[from] EventLoopProxyError),
    #[error(transparent)]
    ContextError(#[from] anyhow::Error)
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("Failed to create event loop")]
    EventLoopCreationError(#[from] EventLoopError)
    
}

#[derive(Error, Debug)]
pub enum StartLoopError {
    #[error("Event loop has already been used and taken!")]
    EventLoopAlreadyConsumedError,
    #[error("Failed to run app")]
    EventLoopRunAppError(#[from] EventLoopError)
}

#[derive(Error, Debug)]
pub enum WindowAccessError {
    #[error("Window corresponding to window id {0} not found in AppHandler.windows")]
    WindowNotFoundError(u64)
}

#[derive(Error, Debug)]
pub enum WindowCreationError {
    #[error("Failed to create window")]
    OSWindowCreationError(#[from] OsError),
}

#[derive(Error, Debug)]
pub enum EventLoopProxyError {
    #[error("Failed to send event through event loop proxy ; Event loop closed")]
    EventLoopProxySendEventError,
}

use thiserror::Error;
use anyhow;

use winit::error::{EventLoopError, OsError};
use crate::application_handler::WindowIdentifier;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    InitializationError(#[from] InitializationError),
    #[error(transparent)]
    StartLoopError(#[from] StartLoopError),
    #[error(transparent)]
    WindowObjectError(#[from] WindowObjectError),
    #[error(transparent)]
    WindowError(#[from] WindowError),
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
pub enum WindowError {
    #[error("Window corresponding to {0:?} not found in AppHandler.windows")]
    WindowNotFoundError(WindowIdentifier),
    #[error("Failed to create window")]
    WindowCreationError(#[from] OsError),
}

#[derive(Error, Debug)]
pub enum WindowObjectError {
    #[error("Unable to resume window")]
    WindowResumeError(#[from] WindowError),
}

#[derive(Error, Debug)]
pub enum EventLoopProxyError {
    #[error("Failed to send event through event loop proxy ; Event loop closed")]
    EventLoopProxySendEventError,
}

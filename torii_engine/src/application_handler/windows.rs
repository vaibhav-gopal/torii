use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use super::error::*;

pub type WindowIdentifierMask = u32;
pub type WindowIdentifierUUID = u32;

#[derive(Debug, Clone, Copy)]
pub enum WindowIdentifier {
    // User given window mask (up to 32)
    Mask(WindowIdentifierMask),
    // User given window ID
    UUID(WindowIdentifierUUID),
    // WindowId (part of winit crate)
    WindowId(WindowId)
}

pub type WindowObjectBuilderPreCallback = Option<Box<dyn FnMut(&mut WindowObjectBuilder)>>;
pub type WindowObjectBuilderPostCallback = Option<Box<dyn FnMut(&mut WindowObjectBuilder, &mut WindowObject)>>;

pub struct WindowObjectBuilder {
    pub attr_state: WindowAttributes,
    pub mask_state: WindowIdentifierMask,
    pub uuid_state: WindowIdentifierUUID,
    pub prebuild_callback: WindowObjectBuilderPreCallback,
    pub postbuild_callback: WindowObjectBuilderPostCallback
}

impl Default for WindowObjectBuilder {
    fn default() -> Self {
        WindowObjectBuilder {
            attr_state: WindowAttributes::default()
                .with_title("Torii Application")
                .with_inner_size(LogicalSize::new(800, 600))
                .with_position(LogicalPosition::new(0, 0)),
            uuid_state: 0,
            mask_state: 0b1,
            prebuild_callback: None,
            postbuild_callback: None
        }
    }
}

impl WindowObjectBuilder {
    pub fn new(attr: Option<WindowAttributes>, prebuild_callback: WindowObjectBuilderPreCallback, postbuild_callback: WindowObjectBuilderPostCallback) -> Self {
        let mut obj = WindowObjectBuilder::default();
        if let Some(attr) = attr { obj.attr_state = attr };
        obj.prebuild_callback = prebuild_callback;
        obj.postbuild_callback = postbuild_callback;
        obj
    }

    pub fn build(&mut self) -> WindowObject {
        self.run_prebuild();
        let mut obj = WindowObject {
            attr: self.attr_state.clone(),
            mask: self.mask_state,
            uuid: self.uuid_state,
            state: WindowState::Suspended,
        };
        self.run_postbuild(&mut obj);
        obj
    }

    pub fn run_prebuild(&mut self) {
        if let Some(mut callback) = self.prebuild_callback.take() {
            let callbackf = &mut callback;
            callbackf(self);
            self.prebuild_callback = Some(callback);
        }
    }

    pub fn run_postbuild(&mut self, obj: &mut WindowObject) {
        if let Some(mut callback) = self.postbuild_callback.take() {
            let callbackf = &mut callback;
            callbackf(self, obj);
            self.postbuild_callback = Some(callback);
        }
    }
}

pub enum WindowState {
    Active(Window),
    Suspended
}

pub struct WindowObject {
    pub state: WindowState,
    pub attr: WindowAttributes,
    pub mask: WindowIdentifierMask,
    pub uuid: WindowIdentifierUUID,
}

impl WindowObject {
    pub(super) fn resume(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        match self.state {
            WindowState::Active(_) => {
                Ok(())
            }
            WindowState::Suspended => {
                let window = event_loop
                    .create_window(self.attr.clone())
                    .map_err(|e| WindowObjectError::WindowResumeError(WindowError::WindowCreationError(e)))?;
                self.state = WindowState::Active(window);
                Ok(())
            }
        }
    }
    pub(super) fn suspend(&mut self) -> Result<()> {
        match self.state {
            WindowState::Active(_) => {
                self.state = WindowState::Suspended;
                Ok(())
            }
            WindowState::Suspended => {
                Ok(())
            }
        }
    }
    pub(super) fn check_match(&self, id: WindowIdentifier) -> bool {
        match id {
            WindowIdentifier::Mask(mask) => {
                (self.mask & mask) > 1
            }
            WindowIdentifier::UUID(uuid) => {
                self.uuid == uuid
            }
            WindowIdentifier::WindowId(id) => {
                match &self.state {
                    WindowState::Active(window) => {
                        window.id() == id
                    }
                    WindowState::Suspended => {
                        false
                    }
                }
            }
        }
    }
}
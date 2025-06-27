use torii_engine::*;
use anyhow::Result;
use torii_engine::application_handler::{AppEvents, LogicalPosition, LogicalSize, WindowAttributes};

fn main() -> Result<()> {
    let app = application_handler::AppHandler::new()?
        .start_loop()?;
    
    // app.send_event(AppEvents::CreateWindow {
    //     attr: Some(WindowAttributes::default()
    //         .with_title("Testing")
    //         .with_position(LogicalPosition::new(0, 0))
    //         .with_inner_size(LogicalSize::new(800, 600))),
    //     mask: 0x1,
    //     uuid: 1,
    // })?;
    
    Ok(())
}
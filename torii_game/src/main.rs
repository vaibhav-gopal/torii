use std::sync::{Arc, Mutex};
use torii_engine::*;
use anyhow::Result;

fn main() -> Result<()> {
    let app = application_handler::AppHandler::new()?;
    let app = Arc::new(Mutex::new(app));
    
    let event_loop = std::thread::spawn(move || {
        let main_loop = move || -> Result<()> {
            app.lock()?.
            app.lock()?.start_loop()?; // will permanently keep the lock due to start loop never returning -> need to get underlying event_loop with take then run manually here
            Ok(())
        };
        
        if let Err(err) = main_loop() {
            
        }
    });
    
    app.lock()?.send_event();
    
    
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
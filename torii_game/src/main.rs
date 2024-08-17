use torii_engine::*;
use anyhow::Result;

fn main() -> Result<()> {
    let _app = application_handler::AppHandler::new()?
        .start_loop()?;
    
    Ok(())
}
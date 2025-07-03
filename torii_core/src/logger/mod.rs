use tracing::{info, warn, debug, error, trace};
use tracing_subscriber::{fmt, EnvFilter, prelude::*};
use std::fs::File;
use std::io::Write;

#[cfg(feature = "tracing-timing")]
mod timing_layer {
    use tracing_timing::{Builder, Histogram};
    use tracing_subscriber::Layer;

    pub fn build<S>() -> impl Layer<S>
    where
        S: tracing::Subscriber,
    {
        let histogram = || {
            Histogram::new_with_max(1_000_000, 2)
                .expect("Failed to create histogram");
        };
        Builder::default().build(histogram)
    }
}

/// Returns fmt layer (JSON or plain)
macro_rules! fmt_layer {
    () => {
        {
            #[cfg(feature = "tracing-json-logs")]
            {
                fmt::layer().json()
            }
            #[cfg(not(feature = "tracing-json-logs"))]
            {
                fmt::layer()
            }
        }
    };
}

/// Returns file layer with writer (JSON or plain)
macro_rules! file_layer {
    ($writer:expr) => {
        {
            #[cfg(feature = "tracing-json-logs")]
            {
                fmt::layer().json().with_writer($writer)
            }
            #[cfg(not(feature = "tracing-json-logs"))]
            {
                fmt::layer().with_writer($writer)
            }
        }
    };
}

/// Sets up registry with conditional timing, file, stdout.
macro_rules! setup_subscriber {
    // With file layer
    ($filter:expr, $stdout_layer:expr, Some($file_layer:expr)) => {{
        let reg = tracing_subscriber::registry()
            .with($filter)
            .with($stdout_layer)
            .with($file_layer);

        #[cfg(feature = "tracing-timing")]
        {
            reg.with(timing_layer::build()).init();
        }
        #[cfg(not(feature = "tracing-timing"))]
        {
            reg.init();
        }
    }};

    // Without file layer
    ($filter:expr, $stdout_layer:expr, None) => {{
        let reg = tracing_subscriber::registry()
            .with($filter)
            .with($stdout_layer);

        #[cfg(feature = "tracing-timing")]
        {
            reg.with(timing_layer::build()).init();
        }
        #[cfg(not(feature = "tracing-timing"))]
        {
            reg.init();
        }
    }};
}

pub struct EngineLogger;

impl EngineLogger {
    /// Logs only to stdout.
    pub fn init_no_file() {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
        let stdout_layer = fmt_layer!();

        setup_subscriber!(filter, stdout_layer, None);
        Self::set_panic_hook();

        info!(
            "Logger initialized: stdout only, json-logs={}, timing={}",
            cfg!(feature = "tracing-json-logs"),
            cfg!(feature = "tracing-timing")
        );
    }

    /// Logs to stdout + default file ("engine.log").
    pub fn init() {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
        let stdout_layer = fmt_layer!();

        let file = File::create("engine.log").expect("Failed to create log file");
        let file_layer = file_layer!(file);

        setup_subscriber!(filter, stdout_layer, Some(file_layer));
        Self::set_panic_hook();

        info!(
            "Logger initialized: stdout + engine.log, json-logs={}, timing={}",
            cfg!(feature = "tracing-json-logs"),
            cfg!(feature = "tracing-timing")
        );
    }

    /// Logs to stdout + custom writer.
    pub fn init_with_writer<W>(writer: W)
    where
        W: Write + Send + Sync + 'static,
    {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
        let stdout_layer = fmt_layer!();
        let file_layer = file_layer!(writer);

        setup_subscriber!(filter, stdout_layer, Some(file_layer));
        Self::set_panic_hook();

        info!(
            "Logger initialized: stdout + custom writer, json-logs={}, timing={}",
            cfg!(feature = "tracing-json-logs"),
            cfg!(feature = "tracing-timing")
        );
    }

    fn set_panic_hook() {
        std::panic::set_hook(Box::new(|info| {
            error!("PANIC: {}", info);
        }));
    }

    pub fn info(msg: &str) { info!(%msg); }
    pub fn debug(msg: &str) { debug!(%msg); }
    pub fn warn(msg: &str) { warn!(%msg); }
    pub fn error(msg: &str) { error!(%msg); }
    pub fn trace(msg: &str) { trace!(%msg); }
}

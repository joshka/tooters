use std::{io::stderr, panic};

use color_eyre::{
    eyre::{self},
    Result,
};
use ratatui::backend::CrosstermBackend;

/// This replaces the standard `color_eyre` panic and error hooks with hooks that
/// restore the terminal before printing the panic or error.
pub fn install_hooks() -> Result<()> {
    let hook_builder = color_eyre::config::HookBuilder::default();
    let (panic_hook, eyre_hook) = hook_builder.into_hooks();

    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = CrosstermBackend::restore(stderr());
        panic_hook(panic_info);
    }));

    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(move |error| {
        let _ = CrosstermBackend::restore(stderr());
        eyre_hook(error)
    }))?;

    Ok(())
}

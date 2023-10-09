use anyhow::{Context, Result};
use crossbeam_channel::Sender;
use log::info;

pub fn termination_handler(tx: &Sender<()>) -> Result<()> {
	let tx = tx.clone();
	ctrlc::set_handler(move || {
		info!("Shutting down due to signal");
		tx.send(())
			.expect("Could not send shutdown signal on channel.")
	})
	.context("failed to set ctrlc/termiantion handler")
}

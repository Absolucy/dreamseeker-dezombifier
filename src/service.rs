use log::error;
use std::{ffi::OsString, time::Duration};
use windows_service::{
	define_windows_service,
	service::{
		ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
		ServiceType,
	},
	service_control_handler::{self, ServiceControlHandlerResult},
	service_dispatcher, Result,
};

const SERVICE_NAME: &str = "dreamseeker-dezombifier";
const SERVICE_TYPE: ServiceType = ServiceType::USER_OWN_PROCESS;

define_windows_service!(ffi_service_main, my_service_main);

pub fn run() -> Result<()> {
	// Register generated `ffi_service_main` with the system and start the service,
	// blocking this thread until the service is stopped.
	service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

pub fn my_service_main(_arguments: Vec<OsString>) {
	if let Err(err) = run_service() {
		error!("Service errored: {}", err);
	}
}

pub fn run_service() -> Result<()> {
	let (shutdown_tx, shutdown_rx) = crossbeam_channel::bounded::<()>(1);

	dreamseeker_dezombifier::terminate::termination_handler(&shutdown_tx)
		.expect("failed to setup termination handler");

	let event_handler = move |control_event| -> ServiceControlHandlerResult {
		match control_event {
			ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
			ServiceControl::Stop => {
				shutdown_tx
					.send(())
					.expect("failed to send shutdown message");
				ServiceControlHandlerResult::NoError
			}
			_ => ServiceControlHandlerResult::NotImplemented,
		}
	};
	let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;
	status_handle.set_service_status(ServiceStatus {
		service_type: SERVICE_TYPE,
		current_state: ServiceState::Running,
		controls_accepted: ServiceControlAccept::STOP,
		exit_code: ServiceExitCode::Win32(0),
		checkpoint: 0,
		wait_hint: Duration::default(),
		process_id: None,
	})?;

	let exit_code = match dreamseeker_dezombifier::main_loop(shutdown_rx) {
		Ok(_) => ServiceExitCode::Win32(0),
		Err(err) => {
			error!("main loop failed: {:?}", err);
			ServiceExitCode::Win32(1)
		}
	};

	status_handle.set_service_status(ServiceStatus {
		service_type: SERVICE_TYPE,
		current_state: ServiceState::Stopped,
		controls_accepted: ServiceControlAccept::empty(),
		exit_code,
		checkpoint: 0,
		wait_hint: Duration::default(),
		process_id: None,
	})?;

	Ok(())
}

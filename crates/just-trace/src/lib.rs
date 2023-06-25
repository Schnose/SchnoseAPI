#[macro_export]
macro_rules! timer {
	() => {{
		let format =
			::time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

		::tracing_subscriber::fmt::time::UtcTime::new(format)
	}};
}

#[macro_export]
macro_rules! layer {
	() => {
		$crate::layer!(false)
	};
	(minimal) => {
		$crate::layer!(false)
	};
	(verbose) => {
		$crate::layer!(true)
	};

	($verbose:expr) => {{
		use ::tracing_subscriber::{
			fmt::format::FmtSpan, layer::SubscriberExt as _, util::SubscriberInitExt as _,
			Layer as _,
		};

		::tracing_subscriber::fmt::layer()
			.pretty()
			.with_timer($crate::timer!())
			.with_ansi(true)
			.with_file($verbose)
			.with_line_number($verbose)
			.with_span_events(FmtSpan::ACTIVE)
			.with_filter(
				::tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or("INFO".into()),
			)
	}};
}

#[macro_export]
macro_rules! registry {
	() => {
		$crate::registry!(false)
	};
	(minimal) => {
		$crate::registry!(false)
	};
	(verbose) => {
		$crate::registry!(true)
	};

	($verbose:expr) => {{
		use ::tracing_subscriber::layer::SubscriberExt;

		::tracing_subscriber::registry().with($crate::layer!($verbose))
	}};
}

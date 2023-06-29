#[macro_export]
macro_rules! tracing_setup {
	($debug:expr) => {{
		use {
			time::macros::format_description,
			tracing_subscriber::{
				fmt::{format::FmtSpan, time::UtcTime},
				layer::SubscriberExt,
				util::SubscriberInitExt,
				EnvFilter, Layer,
			},
		};

		let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

		let timer = UtcTime::new(format);
		let filter = EnvFilter::try_from_default_env().unwrap_or("ERROR,schnose_api=INFO".into());

		let layer = tracing_subscriber::fmt::layer()
			.pretty()
			.with_timer(timer)
			.with_file($debug)
			.with_line_number($debug)
			.with_thread_ids($debug)
			.with_thread_names($debug)
			.with_span_events(FmtSpan::ENTER)
			.with_filter(filter);

		let registry = tracing_subscriber::registry().with(layer);

		registry.init();
	}};
}

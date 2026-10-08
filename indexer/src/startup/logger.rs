use log::Level;
use std::io::Write;

pub fn init_logger() {
    let mut builder = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,sqlx=warn"),
    );
    builder.format(|buf, record| {
        let icon = match record.level() {
            Level::Error => "🔴",
            Level::Warn => "🟡",
            Level::Info => "🟢",
            Level::Debug => "🔎",
            Level::Trace => "·",
        };
        writeln!(
            buf,
            "{} {} [{}] {}",
            icon,
            buf.timestamp(),
            record.target(),
            record.args()
        )
    });
    let _ = builder.try_init();
}

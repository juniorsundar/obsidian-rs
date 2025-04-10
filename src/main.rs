mod config;
mod data;
mod watcher;
mod util;

fn main() {
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter("RUST_LOG")
            .write_style("LOG_STYLE"),
    );
    
   if let Err(e) = watcher::run_watcher() {
        log::error!("Watcher failed to run: {}", e);
        std::process::exit(1);
   } else {
       log::info!("Watcher finished successfully.");
   }
}

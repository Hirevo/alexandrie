use kv_log_macro as log;

fn main() {
    femme::with_level(femme::LevelFilter::Trace);
    log::error!("Buffer has to be 16 bytes in length");
    log::warn!("Unauthorized access attempt", { route: "/login", user_id: "827756627", });
    log::info!("Server listening", { port: "8080" });
    log::info!("Request handled", { method: "GET", path: "/foo/bar", status: 200, elapsed: "4ms" });
    log::debug!("Getting String as bson value type");
    log::trace!("Task spawned", {task_id: "567", thread_id: "12"});
}

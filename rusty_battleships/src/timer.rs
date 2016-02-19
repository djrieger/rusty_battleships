use std::sync::mpsc;
use std::time::Duration;
use std::thread;

pub fn timer_periodic(ms: u64) -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel();
    let duration = Duration::from_millis(ms);
    thread::spawn(move || {
        loop {
            thread::sleep(duration);
            if tx.send(()).is_err() {
                break;
            }
        }
    });
    rx
}

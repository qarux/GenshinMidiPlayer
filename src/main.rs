mod cli;
mod hotkey;
mod midi;

use crate::hotkey::{Hotkey, HotkeyManager};
use crate::midi::KeyPressEvent;
use autopilot::key;
use autopilot::key::Character;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

const POLLING_RATE_MILLIS: u64 = 100;

fn main() {
    let args = cli::read_args();
    let events = midi::read_events(&args.path_to_midi).expect("Failed to read midi file");
    let playing = Arc::new(AtomicBool::new(false));
    let hotkey = Hotkey::new();

    println!("Press {} to play/pause", hotkey);
    let hotkey_manager = {
        let playing = Arc::clone(&playing);
        HotkeyManager::register_hotkey(hotkey, move || {
            playing
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| Some(!value))
                .unwrap();
        })
    };

    // Delay before start, otherwise some keys won't be pressed. Bug?
    while !playing.load(Ordering::SeqCst) {
        sleep(Duration::from_millis(POLLING_RATE_MILLIS));
    }
    sleep(Duration::from_millis(500));

    for event in events {
        while !playing.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(POLLING_RATE_MILLIS));
        }

        match event {
            KeyPressEvent::PressKey(key) => key::tap(&Character(key), &[], 0, 0),
            KeyPressEvent::WaitUs(time) => sleep(Duration::from_micros(time)),
        }
    }

    hotkey_manager.unregister();
}

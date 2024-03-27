use core::sync::atomic::{AtomicBool, Ordering};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

pub struct PlayState {
    channel: Channel<CriticalSectionRawMutex, bool, 1>,
    is_playing_state: AtomicBool,
}

// Allows for lock free checking of the play state
// This is important for playing sound which should not block when is_playing is true
impl PlayState {
    pub const fn new() -> Self {
        Self {
            channel: Channel::new(),
            is_playing_state: AtomicBool::new(true),
        }
    }

    // switches between is_playing true and false
    // and is responsible for notifying the wait function that the state has changed
    pub async fn toggle(&self) {
        if let Ok(is_playing_old) =
            self.is_playing_state
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(!x))
        {
            self.channel.send(!is_playing_old).await;
        }
    }

    // lock free checking of the play state
    pub fn is_playing(&self) -> bool {
        self.is_playing_state.load(Ordering::SeqCst)
    }

    // asynchronously waits is_playing to be set to true
    pub async fn wait(&self) {
        loop {
            if self.is_playing_state.load(Ordering::SeqCst) {
                return;
            } else {
                self.channel.receive().await;
            }
        }
    }
}

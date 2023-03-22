use std::sync::Mutex;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

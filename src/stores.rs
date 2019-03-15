use hashbrown::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use candidate::Candidate;
use std::sync::Once;

static mut STORE: Option<HashSet<String>> = None;
static mut RESULTS: Option<HashSet<Candidate>> = None;

static LOCKED: [AtomicBool; 2] = [AtomicBool::new(false), AtomicBool::new(false)];
static READY: AtomicBool = AtomicBool::new(false);
static ONCE: Once = Once::new();

pub(crate) fn get_ready() {
    READY.store(true, Ordering::SeqCst);
    ONCE.call_once(|| unsafe {
        STORE.replace(HashSet::with_capacity(256));
        RESULTS.replace(HashSet::with_capacity(64));
    });
}

pub(crate) fn reset() {
    READY.store(false, Ordering::SeqCst);

    unsafe {
        if let Some(store) = STORE.as_mut() {
            store.clear();
        }

        if let Some(res) = RESULTS.as_mut() {
            res.clear();
        }
    }
}

pub(crate) fn contains(word: &str) -> bool {
    if !READY.load(Ordering::SeqCst) {
        return true;
    }

    let mut contains = true;
    unsafe {
        if let Some(store) = STORE.as_mut() {
            if !store.contains(word) {
                lock(0);
                contains = !store.insert(word.to_owned());
                unlock(0);
            }
        }
    };
    contains
}

pub(crate) fn publish(candidate: Candidate) -> bool {
    if !READY.load(Ordering::SeqCst) {
        return true;
    }

    unsafe {
        if let Some(res) = RESULTS.as_mut() {
            if res.contains(&candidate) {
                return true;
            }

            lock(1);
            res.insert(candidate);
            unlock(1);

            return false;
        }
    }

    true
}

pub(crate) fn collect() -> Vec<Candidate> {
    lock(1);

    let res =
        if let Some(res) = unsafe { RESULTS.as_mut() } {
            let mut vec = Vec::with_capacity(res.len());
            for candidate in res.drain() {
                vec.push(candidate);
            }

            vec
        } else {
            vec![]
        };

    unlock(1);
    return res;
}

fn lock(id: usize) {
    if id < LOCKED.len() {
        let lock = &LOCKED[id];
        loop {
            if lock.compare_and_swap(false, true, Ordering::SeqCst) {
                break;
            }
        }
    }
}

fn unlock(id: usize) {
    if id < LOCKED.len() {
        LOCKED[id].store(false, Ordering::SeqCst);
    }
}

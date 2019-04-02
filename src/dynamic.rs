use crate::AutoCorrect;
use crate::crossbeam_channel as channel;
use crate::candidate::Candidate;
use crate::common;
use crate::config::Config;
use crate::config::SupportedLocale;
use crate::trie::Node;

pub(crate) fn initialize(service: &AutoCorrect) {
    if let Err(e) = populate_words_set(&service.config) {
        eprintln!("Failed to initialize: {}", e);
    }
}

pub(crate) fn candidate(
    word: String,
    edit: u8,
    max_edit: u8,
    locale: SupportedLocale,
    tx_async: &channel::Sender<Candidate>,
    marker: u32
) {
    if edit >= max_edit {
        return;
    }

    let word = word.trim().to_lowercase();
    if word.is_empty() {
        return;
    }

    if let Some(score) = Node::check(&word) {
        if tx_async.send(Candidate::new(word.to_owned(), score, edit)).is_err() {
            return;
        }
    }

/*
    if let Some(set) = dict_ref() {
        if set.contains_key(&word) {
            let candidate = Candidate::new(word.to_owned(), set[&word], edit);

            if let Some(tx) = tx_async {
                if let Err(_) = tx.send(candidate) {
                    return HashSet::new();
                }
            } else {
                results.insert(candidate);
            }
        }
    }
*/

    // if a misspell, find the correct one within 1 edit distance
    let (tx_curr, rx_curr) = channel::bounded(64);
    let current_edit = edit + 1;

    let (tx_next, tx_next_clone, rx_next) =
        if current_edit < max_edit {
            let (tx_raw, rx_raw) = channel::bounded(256);
            let tx_raw_clone = tx_raw.clone();
            (Some(tx_raw), Some(tx_raw_clone), Some(rx_raw))
        } else {
            (None, None, None)
        };

    let word_clone = word.clone();
    let tx_clone = tx_curr.clone();

    AutoCorrect::run_job(move || {
        common::ins_repl(
            &word_clone,
            current_edit,
            tx_clone,
            tx_next_clone,
            marker
        );

/*
            common::deprecated::delete_n_replace(
                word_clone,
                set,
                current_edit,
                tx_clone,
                tx_next_clone
            )
        }
*/
    });

    if let Some(chan) = rx_next {
        let tx = tx_async.clone();
        AutoCorrect::run_job(move || {
            find_next_edit_candidates(
                current_edit, max_edit, locale, chan, &tx
            );
        });
    }

    AutoCorrect::run_job(move || {
        common::del_tran(
            &word,
            current_edit,
            tx_curr,
            tx_next,
            marker
        );

/*
        if let Some(set) = dict_ref() {
            common::deprecated::transpose_n_insert(
                word,
                set,
                current_edit,
                tx,
                tx_next
            );
        }
*/
    });

    // move rx into the scope so it can drop afterwards
    for candidate in rx_curr {
        if tx_async.send(candidate).is_err() {
            return;
        }
    }
}

fn find_next_edit_candidates(
    edit: u8,
    max_edit: u8,
    locale: SupportedLocale,
    rx_next: channel::Receiver<(String, u32)>,
    tx_async: &channel::Sender<Candidate>
) {
    for (next, marker) in rx_next {
        candidate(
            next,
            edit,
            max_edit,
            locale,
            tx_async,
            marker
        );
    }
}

fn populate_words_set(config: &Config) -> Result<(), String> {
    let (tx, rx) = channel::unbounded();
    let dict_path = config.get_dict_path();

    AutoCorrect::run_job(move || {
        common::load_dict_async(dict_path, tx);
    });

    Node::build(rx);

    Ok(())
}

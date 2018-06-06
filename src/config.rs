#![allow(unreachable_patterns)]

static DEFAULT_MAX_EDIT: u8 = 1;
static MAX_EDIT_THRESHOLD: u8 = 3;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SupportedLocale {
    EnUs,
}

pub struct Config {
    max_edit: u8,
    locale: SupportedLocale,
    override_dict: String,
}

impl Config {
    #[inline]
    pub fn new() -> Config {
        Config::new_with_params(1, SupportedLocale::EnUs, "")
    }

    pub fn new_with_params(max_edit: u8, locale: SupportedLocale, override_dict: &str) -> Config {
        let mut config = Config {
            max_edit: 1,
            locale,
            override_dict: override_dict.to_owned(),
        };

        config.set_max_edit(max_edit);
        config
    }

    pub fn get_dict_path(&self) -> String {        
        if self.override_dict.is_empty() {
            let locale = match self.locale {
                SupportedLocale::EnUs => "en-us",
                _ => "en-us",
            };
            
            format!("./resources/{}/freq_50k.txt", locale)
        } else {
            self.override_dict.to_owned()
        }

        //format!("./resources/{}/words2.txt", locale)

        // let dict_loc = match dict_size {
        //     "high" => "uniq_high",
        //     "low" => "uniq_low",
        //     _ => "uniq_full",
        // };

        // format!("./resources/{}/{}.txt", locale, dict_loc)
    }
}

pub trait AutoCorrectConfig {
    fn set_max_edit(&mut self, max_edit: u8);
    fn get_max_edit(&self) -> u8;
    fn set_locale(&mut self, locale: SupportedLocale);
    fn get_locale(&self) -> SupportedLocale;
    fn set_override_dict(&mut self, dict_path: &str);
    fn get_override_dict(&self) -> String;
}

impl AutoCorrectConfig for Config {
    #[inline]
    fn set_locale(&mut self, locale: SupportedLocale) {
        self.locale = locale;
    }

    #[inline]
    fn get_locale(&self) -> SupportedLocale {
        self.locale.to_owned()
    }

    fn set_max_edit(&mut self, max_edit: u8) {
        // max edit only allowed between 1 and 3
        self.max_edit = if max_edit > MAX_EDIT_THRESHOLD {
            eprintln!("Only support max edits less or equal to {}.", MAX_EDIT_THRESHOLD);
            3
        } else if max_edit < 1 {
            eprintln!("Only support max edits greater or equal to 1.");
            1
        } else {
            max_edit
        };
    }

    #[inline]
    fn get_max_edit(&self) -> u8 {
        self.max_edit
    }
    
    fn set_override_dict(&mut self, dict_path: &str) {
        //TODO: reload dict if changed
        self.override_dict = dict_path.to_owned();
    }

    #[inline]
    fn get_override_dict(&self) -> String {
        self.override_dict.to_owned()
    }
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Config::new_with_params(self.max_edit, self.locale, &self.override_dict[..])
    }
}
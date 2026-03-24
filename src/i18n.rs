use std::sync::OnceLock;

use gettextrs::{LocaleCategory, bind_textdomain_codeset, bindtextdomain, setlocale, textdomain};

static I18N_INITIALIZED: OnceLock<()> = OnceLock::new();

pub fn init() {
    I18N_INITIALIZED.get_or_init(|| {
        setlocale(LocaleCategory::LcAll, "");
        bindtextdomain("riskie", "/usr/share/locale").ok();
        bind_textdomain_codeset("riskie", "UTF-8").ok();
        textdomain("riskie").ok();
    });
}

#[macro_export]
macro_rules! t {
    ($id:expr) => {
        gettextrs::gettext($id)
    };
    ($id:expr, $arg1:expr) => {
        gettextrs::gettext($id).replace("{}", &$arg1.to_string())
    };
    ($id:expr, $arg1:expr, $arg2:expr) => {
        gettextrs::gettext($id)
            .replacen("{}", &$arg1.to_string(), 1)
            .replacen("{}", &$arg2.to_string(), 1)
    };
}

#[macro_export]
macro_rules! tn {
    ($singular:expr, $plural:expr, $n:expr) => {
        gettextrs::ngettext($singular, $plural, $n as u64)
    };
}

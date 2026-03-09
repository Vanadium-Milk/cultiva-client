use sys_locale::get_locales;

//Compares provided locales from i18n with system locales, i18n must share the same package metadata
//with all crates
pub fn match_locales() -> Option<String> {
    let available = rust_i18n::available_locales!();
    for locale in get_locales() {
        if available.contains(&&*locale) {
            return Some(locale);
        }
        let language = &locale[0..2];
        if available.contains(&language) {
            return Some(language.to_string());
        }
    }

    None
}

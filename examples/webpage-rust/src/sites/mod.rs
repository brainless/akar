pub mod akar;
pub mod mimo;

use crate::site::Site;

pub fn create_site(name: &str) -> Box<dyn Site> {
    match name {
        "mimo" => Box::new(mimo::MimoSite::new()),
        "akar" => Box::new(akar::AkarSite::new()),
        _ => unreachable!(),
    }
}

pub fn available_sites() -> &'static [&'static str] {
    &["mimo", "akar"]
}

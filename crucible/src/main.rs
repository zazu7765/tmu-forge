#![allow(unused)]

use crucible::scraper::Scraper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = Scraper::new();
    let deps = s.get_departments("Faculty of Science")?;
    deps.iter().for_each(|dep| println!("{}", dep));
    Ok(())
}

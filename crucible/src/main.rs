#![allow(unused)]

use crucible::scraper::Scraper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = Scraper::new();
    let deps = s.get_departments("Faculty of Science")?;
    deps.iter().for_each(|dep| println!("{}", dep));
    let cres = s.get_courses(deps.first().expect("EMPTY"))?;
    cres.data
        .iter()
        .for_each(|course| println!("Name: {}\n", course.long_title));
    Ok(())
}

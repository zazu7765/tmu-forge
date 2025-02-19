#![allow(unused)]

use crucible::scraper::Scraper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = Scraper::new();
    let deps = s.get_departments("Faculty of Science")?;
    deps.iter().for_each(|dep| {
        s.get_courses(dep)
            .expect("Failed to Retrieve Courses for Department")
            .data
            .iter()
            .for_each(|course| {
                println!(
                    "Name: {}\nPrerequisites: {}\nAntirequisites: {}\nCustom Requisites: {}\n----------\n",
                    course.course_code,
                    course
                        .prerequisites
                        .as_ref()
                        .unwrap_or(&String::from("None")),
                    course
                        .antirequisites
                        .as_ref()
                        .unwrap_or(&String::from("None")),
                    course
                        .customrequisites
                        .as_ref()
                        .unwrap_or(&String::from("None")),
                )
            })
    });
    Ok(())
}

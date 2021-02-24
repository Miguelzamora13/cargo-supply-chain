use crate::common::*;
use crate::publishers::{fetch_owners_of_crates, PublisherKind};

pub fn crates(args: Vec<String>, max_age: std::time::Duration) -> Result<(), std::io::Error> {
    let dependencies = sourced_dependencies(args);
    complain_about_non_crates_io_crates(&dependencies);
    let (mut owners, publisher_teams) = fetch_owners_of_crates(&dependencies, max_age)?;

    for (crate_name, publishers) in publisher_teams {
        owners.entry(crate_name).or_default().extend(publishers)
    }

    let mut ordered_owners: Vec<_> = owners.into_iter().collect();
    // Put crates owned by teams first
    ordered_owners.sort_unstable_by_key(|(name, publishers)| {
        (
            publishers
                .iter()
                .find(|p| p.kind == PublisherKind::team)
                .is_none(), // contains at least one team
            usize::MAX - publishers.len(),
            name.clone(),
        )
    });
    for (_, publishers) in ordered_owners.iter_mut() {
        // For each crate put teams first
        publishers.sort_unstable_by_key(|p| (p.kind, p.login.clone()));
    }

    println!("\nDependency crates with the people and teams that can publish them to crates.io:\n");
    for (i, (crate_name, publishers)) in ordered_owners.iter().enumerate() {
        let pretty_publishers: Vec<String> = publishers
            .iter()
            .map(|p| match p.kind {
                PublisherKind::team => format!("team \"{}\"", p.login),
                PublisherKind::user => p.login.to_string(),
            })
            .collect();
        let publishers_list = comma_separated_list(&pretty_publishers);
        println!("{}. {}: {}", i + 1, crate_name, publishers_list);
    }

    if !ordered_owners.is_empty() {
        println!("\nNote: there may be outstanding publisher invitations. crates.io provides no way to list them.");
        println!("Invitations are also impossible to revoke, and they never expire.");
        println!("See https://github.com/rust-lang/crates.io/issues/2868 for more info.");
    }
    Ok(())
}

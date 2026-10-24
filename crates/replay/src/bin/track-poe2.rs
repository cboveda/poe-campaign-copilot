//! Inspect observed PoE2 transitions without a game, GUI or network calls.
//! Usage: cargo run -p replay --bin track-poe2 -- <Client.txt>
use session::{SessionEvent, SessionTracker};
use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: track-poe2 <Client.txt>")?;
    let input = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut tracker = SessionTracker::new(content::poe2::load_observed_areas()?);
    let mut count = 0;
    for line in input.lines() {
        for event in tracker.on_line(&line?) {
            match event {
                SessionEvent::AreaEntered {
                    area_id,
                    display_name,
                    new_instance,
                    ..
                }
                | SessionEvent::UnmappedArea {
                    area_id,
                    display_name,
                    new_instance,
                    ..
                } => {
                    count += 1;
                    println!(
                        "{count}\t{area_id}\t{display_name}\t{}",
                        if new_instance { "new" } else { "revisit" }
                    );
                }
                _ => {}
            }
        }
    }
    eprintln!("{count} confirmed transitions; quest completion is not inferred.");
    Ok(())
}

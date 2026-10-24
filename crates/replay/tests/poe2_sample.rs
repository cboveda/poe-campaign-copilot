use replay::{fixtures_dir, replay_lines};
use session::SessionEvent;

#[test]
fn supplied_poe2_sample_confirms_all_206_transitions_without_guessing_metadata() {
    let text = std::fs::read_to_string(fixtures_dir().join("poe2-sample.log")).unwrap();
    let expected =
        std::fs::read_to_string(fixtures_dir().join("poe2-sample-transitions.tsv")).unwrap();
    let events = replay_lines(text.lines(), content::poe2::load_observed_areas().unwrap());
    let actual: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            SessionEvent::AreaEntered {
                area_id,
                display_name,
                area_level,
                ..
            }
            | SessionEvent::UnmappedArea {
                area_id,
                display_name,
                area_level,
                ..
            } => Some(format!("{area_id}\t{display_name}\t{area_level}")),
            _ => None,
        })
        .collect();
    let expected: Vec<_> = expected
        .lines()
        .map(|line| line.rsplit_once('\t').unwrap().0.to_string())
        .collect();
    assert_eq!(expected.len(), 206);
    assert_eq!(actual, expected);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, SessionEvent::LevelUp { .. }))
            .count(),
        11
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, SessionEvent::Slain { .. }))
            .count(),
        3
    );
    assert!(events.iter().any(|event| matches!(event, SessionEvent::UnmappedArea { area_id, .. } if area_id.starts_with("Map"))));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SessionEvent::UnresolvedArea { .. }))
    );
}

#[test]
fn poe2_crlf_partial_writes_through_real_tailer_match_replay() {
    use std::io::Write;
    let text = std::fs::read_to_string(fixtures_dir().join("poe2-sample.log")).unwrap();
    let expected = replay_lines(text.lines(), content::poe2::load_observed_areas().unwrap());
    let path = std::env::temp_dir().join(format!("poe2-tailer-{}.log", std::process::id()));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let _cleanup = Cleanup(path.clone());
    let mut file = std::fs::File::create(&path).unwrap();
    let mut poller = input_log::FilePoller::new(path, false).unwrap();
    let mut tracker = session::SessionTracker::new(content::poe2::load_observed_areas().unwrap());
    let mut actual = Vec::new();
    for line in text.lines() {
        let split = line.len() / 2;
        file.write_all(&line.as_bytes()[..split]).unwrap();
        file.flush().unwrap();
        for part in poller.poll().unwrap() {
            actual.extend(tracker.on_line(&part));
        }
        file.write_all(&line.as_bytes()[split..]).unwrap();
        file.write_all(b"\r\n").unwrap();
        file.flush().unwrap();
        for part in poller.poll().unwrap() {
            actual.extend(tracker.on_line(&part));
        }
    }
    assert_eq!(actual, expected);
}

use event_parser::parse_line;
use session::{SessionEvent, SessionTracker};

fn tracker() -> SessionTracker {
    SessionTracker::new(content::poe2::load_observed_areas().unwrap())
}
fn line(pid: u32, tick: u64, message: &str) -> String {
    format!("2026/10/24 12:00:00 {tick} abc [INFO Client {pid}] {message}")
}
fn send(t: &mut SessionTracker, tick: u64, message: &str) -> Vec<SessionEvent> {
    t.on_line(&line(100, tick, message))
}
fn generate(t: &mut SessionTracker, tick: u64, area: &str, seed: u64) {
    send(
        t,
        tick,
        &format!("Generating level 4 area \"{area}\" with seed {seed}"),
    );
}
const LOAD: &str = "[LOADING SCREEN] (The Grelwood) Duration = 2.20582 seconds";

#[test]
fn creation_and_scene_changes_do_not_enter_until_loading_completes_once() {
    let mut t = tracker();
    generate(&mut t, 1_000, "G1_4", 7);
    assert_eq!(t.current_area_id(), None);
    assert!(send(&mut t, 1_010, "[SCENE] Set Source [The Grelwood]").is_empty());
    assert!(send(&mut t, 1_020, "[SCENE] Set Source [Atlas]").is_empty());
    let events = send(&mut t, 2_000, LOAD);
    assert!(
        matches!(&events[..], [SessionEvent::AreaEntered { area_id, new_instance: true, area_level: 4, .. }] if area_id == "G1_4")
    );
    assert_eq!(t.current_area_id(), Some("G1_4"));
    assert!(send(&mut t, 2_001, LOAD).is_empty());
}

#[test]
fn revisits_and_new_instances_use_generation_seeds() {
    let mut t = tracker();
    let mut flags = Vec::new();
    for (tick, seed) in [(1_000, 7), (4_000, 7), (8_000, 9)] {
        generate(&mut t, tick, "G1_4", seed);
        let events = send(&mut t, tick + 1, LOAD);
        let SessionEvent::AreaEntered { new_instance, .. } = events[0] else {
            panic!("expected entry")
        };
        flags.push(new_instance);
    }
    assert_eq!(flags, [true, false, true]);
}

#[test]
fn unknown_id_preserves_observed_location_without_inventing_campaign_metadata() {
    let mut t = tracker();
    generate(&mut t, 100, "MapFuture", 123);
    send(&mut t, 150, "[SCENE] Set Source [Future Map]");
    let events = send(
        &mut t,
        200,
        "[LOADING SCREEN] (Future Map) Duration = 1 seconds",
    );
    assert!(
        matches!(&events[..], [SessionEvent::UnmappedArea { area_id, display_name, new_instance: true, .. }] if area_id == "MapFuture" && display_name == "Future Map")
    );
    assert_eq!(t.current_area_id(), Some("MapFuture"));
    assert_eq!(t.current_act(), None);
}

#[test]
fn mismatched_completion_consumes_pending_without_name_fallback() {
    let mut t = tracker();
    generate(&mut t, 100, "G1_4", 123);
    let events = send(
        &mut t,
        200,
        "[LOADING SCREEN] (Clearfell) Duration = 1 seconds",
    );
    assert!(matches!(&events[..], [SessionEvent::UnresolvedArea { .. }]));
    assert!(send(&mut t, 201, LOAD).is_empty());
    assert_eq!(t.current_area_id(), None);
}

#[test]
fn a_new_connection_invalidates_an_aborted_load() {
    let mut t = tracker();
    generate(&mut t, 100, "G1_4", 7);
    send(
        &mut t,
        200,
        "Connecting to instance server at 192.0.2.1:21360 ",
    );
    assert!(send(&mut t, 300, LOAD).is_empty());
    generate(&mut t, 400, "G1_4", 8);
    assert!(matches!(
        &send(&mut t, 500, LOAD)[..],
        [SessionEvent::AreaEntered { .. }]
    ));
}

#[test]
fn stale_pending_does_not_resurrect_on_late_loading_completion() {
    let mut t = tracker();
    generate(&mut t, 100, "G1_4", 7);
    assert!(send(&mut t, 180_101, LOAD).is_empty());
    assert_eq!(t.current_area_id(), None);
}

#[test]
fn process_changes_and_clock_resets_invalidate_pending_and_character_identity() {
    for (pid, tick) in [(101, 200), (100, 1)] {
        let mut t = tracker();
        send(&mut t, 50, ": SampleExile (Mercenary) is now level 3");
        generate(&mut t, 100, "G1_4", 7);
        let events = t.on_line(&line(pid, tick, LOAD));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, SessionEvent::AreaEntered { .. }))
        );
        assert_eq!(t.pinned_character(), None);
        assert_eq!(t.current_area_id(), None);
    }
}

#[test]
fn login_boundary_clears_pending_and_pinned_character_in_same_process() {
    let mut t = tracker();
    send(&mut t, 10, ": FirstExile (Mercenary) is now level 3");
    generate(&mut t, 20, "G1_4", 7);
    send(&mut t, 30, "Async connecting to example.invalid:21269");
    assert!(
        send(&mut t, 40, LOAD)
            .iter()
            .all(|e| !matches!(e, SessionEvent::AreaEntered { .. }))
    );
    send(&mut t, 50, ": SecondExile (Sorceress) is now level 2");
    assert_eq!(t.pinned_character(), Some("SecondExile"));
    assert_eq!(t.player_level(), Some(2));
}

#[test]
fn log_parser_handles_crlf_and_parentheses_and_rejects_invalid_durations() {
    assert!(
        matches!(parse_line(&line(1, 1, "[LOADING SCREEN] (Example (Upper)) Duration = 1.2 seconds\r\n")), event_parser::RawEvent::LoadingCompleted { display_name, .. } if display_name == "Example (Upper)")
    );
    for invalid in [
        "NaN seconds",
        "inf seconds",
        "-1 seconds",
        "garbage",
        "1.2",
        "",
    ] {
        let raw = parse_line(&line(
            1,
            1,
            &format!("[LOADING SCREEN] (The Grelwood) Duration = {invalid}"),
        ));
        assert!(
            matches!(raw, event_parser::RawEvent::Unknown { .. }),
            "accepted {invalid}"
        );
    }
}

#[test]
fn unknown_generation_requires_a_matching_scene_before_loading() {
    for scene in [None, Some("Some Other Zone")] {
        let mut t = tracker();
        generate(&mut t, 100, "MapFuture", 7);
        if let Some(name) = scene {
            send(&mut t, 150, &format!("[SCENE] Set Source [{name}]"));
        }
        let events = send(
            &mut t,
            200,
            "[LOADING SCREEN] (Future Map) Duration = 1 seconds",
        );
        assert!(matches!(&events[..], [SessionEvent::UnresolvedArea { .. }]));
        assert_eq!(t.current_area_id(), None);
    }
}

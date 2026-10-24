# PoE2 sample provenance

`poe2-sample.log` is a minimal derivative of the user-supplied English
Client-sample.txt (20,000 lines; September 15–October 22, 2026).
It retains generation, connection, scene, loading-completion, level-up and
death messages. Chat, account/guild details, login destinations and unrelated
diagnostics were removed. Login destinations use example.invalid. Connection endpoints use documentation address
192.0.2.1; character names use SampleExile. Timestamps, logger process IDs,
area IDs/names, seeds, levels and durations preserve correlation evidence.
No full private log is committed.

`poe2-sample-transitions.tsv` is the independently extracted expected order:
area ID, loading name, area level, generation seed (206 rows). It is not
produced by the parser under test. No quest/reward completion is asserted.
The excerpt is partial campaign coverage and does not establish patch-wide
compatibility. Additional edge cases are explicitly synthetic unit tests.

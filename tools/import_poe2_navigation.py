#!/usr/bin/env python3
"""Combine pinned MIT cues with reviewed navigation summaries; no network."""
import argparse
import hashlib
import json
from import_poe2_act1 import ROOT, SOURCE, REF, ALIASES, Document

REVIEW = ROOT / 'content/poe2/navigation-review.json'
OUTPUT = ROOT / 'content/poe2/navigation.json'


def generate():
    source = SOURCE.read_text(encoding='utf-8')
    doc = Document()
    doc.feed(source)
    area = None
    entries = []
    # The Grim Tangle cue describes finding its own entrance from an unclear
    # origin. Keep it out until a source establishes which zone it applies to.
    allowed = {'The Grelwood', 'Cemetery of the Eternals', 'Hunting Grounds',
               'Freythorn', 'Ogham Village'}
    for node in doc.root.walk():
        if node.has('zone-header'):
            area = ''.join(c for c in node.children if isinstance(c, str)).strip()
            area = ALIASES.get(area, area)
        if node.has('note') and (
            (node.has('cue') and area in allowed)
            or (area == 'Clearfell' and node.has('tip') and 'bridge onward' in node.text())
        ):
            text = ' '.join(''.join(
                child.text() if not isinstance(child, str) else child
                for child in node.children
                if isinstance(child, str) or not child.has('label')
            ).split())
            entries.append({'act': 1, 'area': area, 'text': text,
                            'source_id': 'domistae-act1', 'source_label': 'domistae',
                            'patch_caveat': '',
                            'source_anchor': next((n.attrs.get('id') for n in doc.root.walk()
                                                   if n.has('zone-header') and n.attrs.get('id')
                                                   and ''.join(c for c in n.children if isinstance(c, str)).strip() == area), None)})
    if len(entries) != 7 or len({e['area'] for e in entries}) != 6:
        raise ValueError('source navigation coverage changed')
    review = json.loads(REVIEW.read_text(encoding='utf-8'))
    if review['format_version'] != 1 or not review['entries']:
        raise ValueError('invalid navigation review')
    old_areas = {entry['area'] for entry in entries}
    if not set(review['replace_areas']).issubset(old_areas):
        raise ValueError('review replaces an unknown source area')
    reviewed_areas = {entry['area'] for entry in review['entries']}
    if not set(review['replace_areas']).issubset(reviewed_areas):
        raise ValueError('review replacement has no navigation entries')
    for entry in review['entries']:
        if entry['act'] != 1 or not entry['area'].strip() or not entry['text'].strip() or entry['source_id'] != review['source']['id']:
            raise ValueError('invalid reviewed navigation cue')
    entries = [entry for entry in entries if entry['area'] not in review['replace_areas']]
    entries.extend(review['entries'])
    return {'format_version': 1, 'game': 'poe2', 'target_patch': '0.5.5',
            'source': {'url': 'https://domistae.github.io/poe2-leveling/poe2_act1_guide.html',
                       'revision': REF, 'license': 'MIT',
                       'sha256_utf8_lf': hashlib.sha256(source.encode('utf-8')).hexdigest()},
            'comparison_source': review['source'],
            'status': 'source_guidance_not_gameplay_verified', 'entries': entries}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    output = json.dumps(generate(), indent=2, ensure_ascii=False) + '\n'
    if args.check:
        if OUTPUT.read_text(encoding='utf-8') != output:
            raise SystemExit('navigation.json is out of date')
        print('PoE2 navigation import matches pinned source')
    else:
        OUTPUT.write_text(output, encoding='utf-8')

#!/usr/bin/env python3
"""Generate remaining campaign guidance from pinned MIT HTML, without network or scripts.
Act 4 and Interludes are area guidance, not a forced linear completion order.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path
from import_poe2_act1 import Document, ROOT, REF, generate as generate_act1

OUTPUT = ROOT / 'content/poe2/remaining-campaign.json'


def clean_title(title):
    title = re.sub(r'\s*\((Return|End of Act)\)\s*$', '', title).rstrip(' ⚠')
    return {'Kingsmarch — Hideout': 'Kingsmarch'}.get(title, title)


def aliases(name):
    result = [name]
    result.append(name[4:] if name.startswith('The ') else 'The ' + name)
    result += {'Trial of Chaos': ['Temple of Chaos', 'The Temple of Chaos'],
               'Trial of the Ancestors': ['Trial of Ancestors']}.get(name, [])
    return list(dict.fromkeys(result))


def parse_source(file, expected):
    source = ROOT / 'vendor/poe2-leveling' / file
    doc = Document()
    doc.feed(source.read_text(encoding='utf-8'))
    visits, steps = [], {}
    for node in doc.root.walk():
        if node.has('zone-header'):
            title = ''.join(c for c in node.children if isinstance(c, str)).strip()
            visits.append({'name': clean_title(title), 'source_heading': title,
                           'is_town': any(c.has('town') for c in node.walk()), 'source_steps': []})
        if node.has('step') and 'data-step' in node.attrs:
            content = next((c for c in node.walk() if c.has('step-content')), None)
            if content is not None:
                number = int(node.attrs['data-step'])
                if number in steps:
                    raise ValueError(f'duplicate source step {file}:{number}')
                steps[number] = ' '.join(content.text().split())
                if visits:
                    visits[-1]['source_steps'].append(number)
    if len(steps) != expected:
        raise ValueError(f'{file}: expected {expected} steps, got {len(steps)}')
    return visits, steps, {'project': 'domistae/poe2-leveling', 'reference': REF,
                         'file': file, 'license': 'MIT', 'claimed_patch': '0.5.5',
                         'checksum_normalization': 'UTF-8 with LF line endings',
            'sha256': hashlib.sha256(source.read_text(encoding='utf-8').encode('utf-8')).hexdigest()}


def generate():
    chapters = []
    for act, count in [(2, 78), (3, 81), (4, 67)]:
        visits, steps, source = parse_source(f'poe2_act{act}_guide.html', count)
        town_names = {v['name'] for v in visits if v['is_town']}
        excluded = {28, 81} if act == 3 else set()  # explicit league-mechanic objectives
        for visit in visits:
            visit['source_steps'] = [n for n in visit['source_steps'] if n not in excluded]
            visit['objectives'] = [steps[n] for n in visit['source_steps']]
            visit['aliases'] = aliases(visit['name'])
            visit['is_town'] = visit['name'] in town_names
            if not visit['objectives']:
                raise ValueError(f'empty visit: {source["file"]}:{visit["name"]}')
        chapters.append({'id': f'act{act}', 'label': f'Act {act}', 'act': act,
                         'mode': 'sequential' if act < 4 else 'area_guidance',
                         'source': source, 'visits': visits, 'context_objectives': [],
                         'excluded_source_steps': sorted(excluded)})

    # Named destinations embedded within objective text get supplemental
    # guidance, without consuming the parent route visit.
    for act, count, mapping in [
        (2, 78, [('The Venom Pit', 'Keth', [38]), ('The Galleria', 'The Lost City', [40])]),
        (4, 67, [('Shark Pit', 'Whakapanu Island', [7]),
                 ('Crabshell Cavern', 'Whakapanu Island', [8]),
                 ('Armoury', 'Abandoned Prison', [19]),
                 ('Forgotten Cells', 'Abandoned Prison', [19]),
                 ('Doomed Mine', 'Isle of Kin', [25]), ('Beast Pen', 'Isle of Kin', [26, 27]),
                 ('Primal Arena', 'Isle of Kin', [28]),
                 ('Volcanic Nest', 'Volcanic Warrens', [30]),
                 ('Abandoned Ship', 'Kedge Bay', [44]), ('Tidal Cay', 'Kedge Bay', [45])]),
    ]:
        _, side_steps, side_source = parse_source(f'poe2_act{act}_guide.html', count)
        visits = []
        for name, parent, refs in mapping:
            if not any(name.removeprefix('The ').lower() in side_steps[n].lower() for n in refs):
                raise ValueError(f'side-area source changed: {name}')
            visits.append({'name': name, 'source_heading': parent, 'aliases': aliases(name),
                           'parent': parent, 'is_town': False, 'source_steps': refs,
                           'objectives': [side_steps[n] for n in refs]})
        chapters.append({'id': f'act{act}-side-areas', 'label': f'Act {act}', 'act': act,
                         'mode': 'area_guidance', 'source': side_source, 'visits': visits,
                         'context_objectives': [], 'excluded_source_steps': []})

    sections, steps, source = parse_source('poe2_interludes_guide.html', 39)
    # Only use named locations directly mentioned in the guide. Shared
    # objectives with unclear area attribution remain chapter-wide reminders.
    specs = [
        ('interlude51', 'Interlude 5.1', [
            ('The Refuge', [1]), ('Scorched Farmlands', [2, 37]),
            ('Stones of Serle', [3]), ('Blackwood', [5]), ('Holten', [4, 6]),
            ('Wolvenhold', [7]), ('Holten Estate', [8, 9]),
        ], []),
        ('interlude52', 'Interlude 5.2', [
            ('Khari Bazaar', [10]), ('Khari Crossing', [11, 12, 13]),
            ('Pools of Kathal', [14]), ('Sel Khari Sanctuary', [38, 15]),
            ('Molten Shrine', [16]), ('Skullmaw Stairway', [16]),
            ('Qimah', [17, 18]), ('Qimah Reservoir', [19, 20]),
        ], [21, 22]),
        ('interlude53', 'Interlude 5.3', [
            ('The Glade', [23]), ('Ashen Forest', [24]), ('Kriar Village', [25]),
            ('Glacial Tarn', [26, 28]), ('Howling Caves', [26]),
            ('Kriar Peaks', [29]), ('Etched Ravine', [30]),
            ('The Cuachic Vault', [31, 39, 32]),
        ], [27]),
    ]
    for chapter_id, label, mapping, context in specs:
        visits = []
        for name, refs in mapping:
            if not all(name.removeprefix('The ') in steps[n] for n in refs):
                # A destination may be the context introduced by the previous
                # source step. Keep this explicit; never infer a game ID.
                permitted = {'Khari Crossing', 'Khari Bazaar', 'Scorched Farmlands', 'Pools of Kathal', 'Sel Khari Sanctuary',
                             'The Cuachic Vault', 'Holten Estate'}
                if name not in permitted:
                    raise ValueError(f'unreviewed area attribution: {name}')
            objectives = [steps[n] for n in refs]
            if name == 'Howling Caves':
                objectives = [steps[26][steps[26].index('slay The Abominable Yeti'):]]
            visits.append({'name': name, 'source_heading': label, 'aliases': aliases(name),
                           'is_town': False, 'source_steps': refs, 'objectives': objectives})
        chapters.append({'id': chapter_id, 'label': label, 'act': 5,
                         'mode': 'area_guidance', 'source': source, 'visits': visits,
                         'context_objectives': [steps[n] for n in context],
                         'context_source_steps': context, 'excluded_source_steps': []})
    # Conditional campaign hand-in, without enabling the Maps/Endgame section.
    hub = next(chapter for chapter in chapters if chapter['id'] == 'act4')['visits'][0]
    hub['objectives'].append(steps[33])
    hub['completion_source'] = {'file': source['file'], 'step': 33}
    hub['completion_condition'] = 'After all three Interludes; user-verified, never inferred'
    covered = {n for _, _, mapping, context in specs for _, refs in mapping for n in refs}
    covered |= {n for _, _, _, context in specs for n in context}
    if covered != (set(steps) - {33, 34, 35, 36}):
        raise ValueError('Interlude campaign step coverage changed')
    return {'format_version': 1, 'game': 'poe2', 'target_patch': '0.5.5',
            'validation': 'source-imported; Windows gameplay verification pending',
            'chapters': chapters, 'excluded_endgame_source_steps': [34, 35, 36]}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    outputs = [(ROOT / 'content/poe2/act1-route.json', generate_act1()), (OUTPUT, generate())]
    for path, data in outputs:
        text = json.dumps(data, ensure_ascii=False, indent=2) + '\n'
        if args.check:
            if path.read_text(encoding='utf-8') != text:
                raise SystemExit(f'{path.name} differs; regenerate and review')
        else:
            path.write_text(text, encoding='utf-8')
        print(('verified ' if args.check else 'wrote ') + path.name)

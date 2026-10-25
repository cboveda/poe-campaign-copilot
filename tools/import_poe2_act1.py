#!/usr/bin/env python3
"""Extract objective text only from the pinned, licensed Act 1 guide.

No scripts are executed and no network is used. --check verifies the generated
file without overwriting it. Source IDs preserve review links to each step.
"""
import argparse
import hashlib
import json
import re
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'vendor/poe2-leveling/poe2_act1_guide.html'
OUTPUT = ROOT / 'content/poe2/act1-route.json'
REF = '916bd138286fe1307113ec044bcb4d565432b516'
ALIASES = {
    'Riverbank': 'The Riverbank',
    'Grim Tangle': 'The Grim Tangle',
    'Cemetery of the Eternals': 'Cemetery of the Eternals',
}

class Node:
    def __init__(self, tag='', attrs=()):
        self.tag, self.attrs, self.children = tag, dict(attrs), []

    def has(self, cls):
        return cls in self.attrs.get('class', '').split()

    def text(self):
        return ''.join(c.text() if isinstance(c, Node) else c for c in self.children)

    def walk(self):
        yield self
        for child in self.children:
            if isinstance(child, Node):
                yield from child.walk()

class Document(HTMLParser):
    def __init__(self):
        super().__init__()
        self.root = Node()
        self.stack = [self.root]

    def handle_starttag(self, tag, attrs):
        node = Node(tag, attrs)
        self.stack[-1].children.append(node)
        if tag not in {'area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta', 'param', 'source', 'track', 'wbr'}:
            self.stack.append(node)

    def handle_endtag(self, tag):
        for i in range(len(self.stack)-1, 0, -1):
            if self.stack[i].tag == tag:
                del self.stack[i:]
                return

    def handle_data(self, data):
        self.stack[-1].children.append(data)


def generate():
    doc = Document()
    doc.feed(SOURCE.read_text(encoding='utf-8'))
    visits = []
    source_steps = {}
    for node in doc.root.walk():
        if node.has('step') and 'data-step' in node.attrs:
            content = next((child for child in node.walk() if child.has('step-content')), None)
            if content is not None:
                source_steps[int(node.attrs['data-step'])] = ' '.join(content.text().split())
        if node.has('zone-header'):
            title = ''.join(c for c in node.children if isinstance(c, str)).strip()
            name = re.sub(r'\s*\((Return|End of Act)\)\s*$', '', title).strip()
            name = ALIASES.get(name, name)
            visits.append({'name': name, 'source_heading': title, 'objectives': []})
        if node.has('step-content') and visits:
            text = ' '.join(node.text().split())
            visits[-1]['objectives'].append(text)
    if not visits or any(not visit['objectives'] for visit in visits):
        raise ValueError('guide structure changed: empty visit or no visits')
    steps = sum(len(v['objectives']) for v in visits)
    if steps != 65:
        raise ValueError(f'guide structure changed: expected 65 objectives, found {steps}')
    # These objectives occur inside parent-zone text rather than as route
    # visit headings. Keep them separate so entering a side area cannot
    # falsely consume the parent visit or reorder the main campaign.
    side_areas = []
    for name, aliases, parent, refs in [
        ('Mud Burrow', ['Mud Burrow', 'The Mud Burrow'], 'Clearfell', [9]),
        ('Mausoleum of the Praetor', ['Mausoleum of the Praetor'], 'Cemetery of the Eternals', [33, 34]),
        ('Tomb of the Consort', ['Tomb of the Consort'], 'Cemetery of the Eternals', [35]),
    ]:
        objectives = [source_steps[ref] for ref in refs]
        if not all(name in text for text in objectives):
            raise ValueError(f'guide side-area references changed: {name}')
        side_areas.append({'name': name, 'aliases': aliases, 'parent': parent,
                           'source_steps': refs, 'objectives': objectives})
    return {
        'format_version': 1,
        'game': 'poe2',
        'coverage': 'act1-preview',
        'target_patch': '0.5.5',
        'target_confirmation': 'user-reported client patch; gameplay content verification pending',
        'validation': 'source-imported; live-patch and gameplay review pending',
        'source': {
            'project': 'domistae/poe2-leveling', 'reference': REF,
            'url': 'https://domistae.github.io/poe2-leveling/poe2_act1_guide.html',
            'claimed_patch': '0.5.5', 'license': 'MIT',
            'checksum_normalization': 'UTF-8 with LF line endings',
            'sha256': hashlib.sha256(SOURCE.read_text(encoding='utf-8').encode('utf-8')).hexdigest(),
        },
        'visits': visits,
        'side_areas': side_areas,
    }

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    result = json.dumps(generate(), ensure_ascii=False, indent=2) + '\n'
    if args.check:
        if OUTPUT.read_text(encoding='utf-8') != result:
            raise SystemExit('Act 1 route differs; regenerate and review the changes')
        print('Act 1 route matches pinned source')
    else:
        OUTPUT.write_text(result, encoding='utf-8')
        print(f'wrote {OUTPUT}')

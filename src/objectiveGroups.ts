// The pinned guide expresses optionality with a leading marker. Translate
// that source notation into display groups without changing route progression.
export function groupObjectives(steps: string[]) {
  const campaign: string[] = [];
  const optional: string[] = [];
  const optionalPrefix = /^\s*\((?:opt|optional)\)\s*/i;
  for (const step of steps) {
    if (!step.trim()) continue;
    if (optionalPrefix.test(step)) {
      const text = step.replace(optionalPrefix, "").trim();
      if (text) optional.push(text);
    } else {
      campaign.push(step);
    }
  }
  return { campaign, optional };
}

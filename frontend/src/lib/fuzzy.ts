/**
 * Scores how well `query` matches `text` for the command palette: a
 * contiguous substring beats a scattered subsequence, and earlier matches
 * beat later ones. `null` means no match at all.
 */
export function fuzzyScore(query: string, text: string): number | null {
	const needle = query.trim().toLowerCase();
	const haystack = text.toLowerCase();
	if (!needle) return 0;

	const substringAt = haystack.indexOf(needle);
	if (substringAt !== -1) return 1000 - substringAt;

	// Every query character must appear, in order; gaps cost points.
	let score = 500;
	let from = 0;
	for (const character of needle) {
		const found = haystack.indexOf(character, from);
		if (found === -1) return null;
		score -= found - from;
		from = found + 1;
	}
	return score;
}

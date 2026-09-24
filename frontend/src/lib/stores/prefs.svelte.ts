import type { Grouping } from '$lib/graph/grouping';
import type { NodeFocus } from '$lib/types/NodeFocus';
import type { NodeKind } from '$lib/types/NodeKind';

/**
 * Per-browser preferences (Settings > Appearance / Functionality). Kept in
 * localStorage - nothing here needs to follow the account across devices.
 */
export type Prefs = {
	theme: 'system' | 'light' | 'dark';
	density: 'comfortable' | 'compact';
	defaultBoardView: 'organized' | 'canvas' | 'timeline';
	captureKind: NodeKind;
	captureFocus: NodeFocus;
	pokeFromCards: boolean;
	organizedGrouping: Grouping;
};

export const PREFS_STORAGE_KEY = 'loom.prefs';

const DEFAULTS: Prefs = {
	theme: 'system',
	density: 'comfortable',
	defaultBoardView: 'organized',
	captureKind: 'idea',
	captureFocus: 'secondary',
	pokeFromCards: true,
	organizedGrouping: 'focus'
};

function loadPrefs(): Prefs {
	try {
		const stored = JSON.parse(localStorage.getItem(PREFS_STORAGE_KEY) ?? '{}');
		return { ...DEFAULTS, ...stored };
	} catch {
		return { ...DEFAULTS };
	}
}

export const prefs = $state<Prefs>(loadPrefs());

export function savePrefs() {
	try {
		localStorage.setItem(PREFS_STORAGE_KEY, JSON.stringify(prefs));
	} catch {
		// Storage unavailable (private mode, quota) - prefs still apply for this session.
	}
	applyAppearance();
}

const darkQuery = window.matchMedia('(prefers-color-scheme: dark)');

/** Mirrors theme/density onto <html>; app.html does the same before first paint. */
export function applyAppearance() {
	const dark = prefs.theme === 'dark' || (prefs.theme === 'system' && darkQuery.matches);
	document.documentElement.dataset.theme = dark ? 'dark' : 'light';
	document.documentElement.dataset.density = prefs.density;
}

darkQuery.addEventListener('change', applyAppearance);

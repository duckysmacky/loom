import { settingsApi } from '$lib/api/endpoints';
import type { Grouping } from '$lib/graph/grouping';
import type { NodeFocus } from '$lib/types/NodeFocus';
import type { NodeKind } from '$lib/types/NodeKind';
import type { PathStaleMode } from '$lib/types/PathStaleMode';
import type { PathStatusMode } from '$lib/types/PathStatusMode';
import type { StaleMode } from '$lib/types/StaleMode';
import type { StalePeriodEnd } from '$lib/types/StalePeriodEnd';

/**
 * Preferences (Settings page). Everything except `LOCAL_ONLY` is saved to the
 * account (`GET/PATCH /api/settings`) and mirrored in localStorage, which is
 * only a first-paint cache (app.html reads it before the app loads).
 */
export type Prefs = {
	theme: 'system' | 'light' | 'dark';
	density: 'comfortable' | 'compact';
	/** Lucide icons across the UI, or the older typographic glyphs (also picks the favicon). */
	iconStyle: 'icons' | 'glyphs';
	defaultBoardView: 'organized' | 'canvas' | 'timeline';
	captureKind: NodeKind;
	captureFocus: NodeFocus;
	pokeFromCards: boolean;
	organizedGrouping: Grouping;
	/** Custom colors picked via the native color input, most recent first. */
	recentColors: string[];
	/** A poke also switches the node's status to Active. */
	pokeSetsActive: boolean;
	/** Pausing closes the active period, reactivating opens a new one. */
	trackActivePeriods: boolean;
	/** A focus change closes the open active period and starts a new one under the new focus. */
	splitPeriodOnFocusChange: boolean;
	/**
	 * Paths the user expanded/collapsed, keyed `view:pathId` - only toggled
	 * ones; the rest follow the view's default. Flat, so the shallow merge in
	 * `loadPrefs` keeps working. ponytail: keys of deleted paths linger.
	 */
	collapsedPaths: Record<string, boolean>;
	/** How a path's card shows what's inside it. */
	pathDetailView: 'list' | 'canvas';
	/** The canvas lays out and pins unplaced nodes itself; off: they wait in its Unplaced panel. */
	autoPlace: boolean;
	/** Snap canvas dragging to the visible grid; remembered per browser. */
	canvasSnapToGrid: boolean;
	unplacedPanelOpen: boolean;
	/** Dropping a node into a path box too small for it grows the box. */
	pathAutoExpand: boolean;
	/** Canvas connections carry a small text label (requires / next / related). */
	showEdgeLabels: boolean;
	/** Days without a poke (else since creation) before a node goes stale. */
	staleDays: number;
	/** The dashboard warns this many days ahead; 0 = off. */
	staleWarnDays: number;
	/** Kinds that can go stale (project / study). */
	staleKinds: NodeKind[];
	/** Whether queued / paused / idea-status nodes can go stale (active ones always can). */
	staleQueued: StaleMode;
	stalePaused: StaleMode;
	staleIdeaStatus: StaleMode;
	/** An active node that goes stale is paused. */
	staleAutoPause: boolean;
	/** Where auto-pause ends the active period. */
	stalePausePeriodEnd: StalePeriodEnd;
	/** How paths inherit staleness from the nodes inside. */
	pathStaleMode: PathStaleMode;
	/** Path status: hidden, set by hand, or derived from inside. */
	pathStatusMode: PathStatusMode;
};

/** Views where paths expand and collapse, each remembering its own state. */
export type PathView = 'organized' | 'dashboard' | 'canvas' | 'detail';

export const PREFS_STORAGE_KEY = 'loom.prefs';
/** Set once this browser's old local prefs were uploaded to the account. */
const MIGRATED_KEY = 'loom.prefs.synced';
/** Per-browser UI state that stays out of the account settings. */
const LOCAL_ONLY: (keyof Prefs)[] = ['collapsedPaths', 'unplacedPanelOpen', 'canvasSnapToGrid'];
const SAVE_DELAY_MS = 400;
const MAX_RECENT_COLORS = 8;

const DEFAULTS: Prefs = {
	theme: 'system',
	density: 'comfortable',
	iconStyle: 'icons',
	defaultBoardView: 'organized',
	captureKind: 'idea',
	captureFocus: 'secondary',
	pokeFromCards: true,
	organizedGrouping: 'focus',
	recentColors: [],
	pokeSetsActive: false,
	trackActivePeriods: true,
	splitPeriodOnFocusChange: false,
	collapsedPaths: {},
	pathDetailView: 'list',
	autoPlace: false,
	canvasSnapToGrid: false,
	unplacedPanelOpen: true,
	pathAutoExpand: true,
	showEdgeLabels: false,
	staleDays: 14,
	staleWarnDays: 3,
	staleKinds: ['project', 'study'],
	staleQueued: 'never',
	stalePaused: 'never',
	staleIdeaStatus: 'never',
	staleAutoPause: false,
	stalePausePeriodEnd: 'last_poke',
	pathStaleMode: 'any',
	pathStatusMode: 'auto'
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

/** Records a color picked via the native picker - most-recent-first, deduped, capped. */
export function addRecentColor(color: string) {
	prefs.recentColors = [
		color,
		...prefs.recentColors.filter((existing) => existing !== color)
	].slice(0, MAX_RECENT_COLORS);
	savePrefs();
}

/** Paths start collapsed on the organized board, dashboard and path cards, expanded on the canvas. */
export function isPathCollapsed(view: PathView, pathId: string): boolean {
	return prefs.collapsedPaths[`${view}:${pathId}`] ?? view !== 'canvas';
}

export function togglePathCollapsed(view: PathView, pathId: string) {
	prefs.collapsedPaths[`${view}:${pathId}`] = !isPathCollapsed(view, pathId);
	savePrefs();
}

/** Save progress for the settings page; `saves` ticks after each successful save. */
export const syncState = $state({
	status: 'idle' as 'idle' | 'saving' | 'saved' | 'error',
	saves: 0
});

const toSnake = (key: string) => key.replace(/[A-Z]/g, (char) => `_${char.toLowerCase()}`);
const toCamel = (key: string) => key.replace(/_([a-z])/g, (_, char) => char.toUpperCase());
const mapKeys = (source: object, convert: (key: string) => string) =>
	Object.fromEntries(Object.entries(source).map(([key, value]) => [convert(key), value]));

/** Last values known to be on the server, to send only what changed. */
let serverValues: Partial<Prefs> = $state.snapshot(prefs);
let saveTimer: ReturnType<typeof setTimeout> | undefined;
let saving = false;

/** Synced keys whose local value differs from the server's. */
function dirtyKeys(): (keyof Prefs)[] {
	return (Object.keys(prefs) as (keyof Prefs)[]).filter(
		(key) =>
			!LOCAL_ONLY.includes(key) && JSON.stringify(prefs[key]) !== JSON.stringify(serverValues[key])
	);
}

function cachePrefs() {
	try {
		localStorage.setItem(PREFS_STORAGE_KEY, JSON.stringify(prefs));
	} catch {
		// Storage unavailable (private mode, quota) - prefs still apply for this session.
	}
}

/** Applies and caches locally right away; the account save follows after a short pause. */
export function savePrefs() {
	cachePrefs();
	applyAppearance();
	clearTimeout(saveTimer);
	saveTimer = setTimeout(flushPrefs, SAVE_DELAY_MS);
}

async function flushPrefs() {
	if (saving) return void (saveTimer = setTimeout(flushPrefs, SAVE_DELAY_MS));
	const keys = dirtyKeys();
	if (!keys.length) return;
	saving = true;
	syncState.status = 'saving';
	const sent = Object.fromEntries(keys.map((key) => [key, $state.snapshot(prefs[key])]));
	try {
		await settingsApi.patch(mapKeys(sent, toSnake));
		serverValues = { ...serverValues, ...sent };
		syncState.status = 'saved';
		syncState.saves++;
	} catch {
		// Kept locally and retried on the next change; the settings page shows the failure.
		syncState.status = 'error';
	} finally {
		saving = false;
	}
}

/**
 * Pulls the account's settings. Local edits not yet saved win. The first time
 * a browser with old local prefs loads, its values seed whatever the account
 * still has at the defaults (the first browser to upgrade wins).
 */
export async function loadSettings() {
	try {
		const remote = mapKeys(await settingsApi.get(), toCamel) as Partial<Prefs>;
		const remoteKeys = Object.keys(remote) as (keyof Prefs)[];
		const same = (left: unknown, right: unknown) => JSON.stringify(left) === JSON.stringify(right);
		const migrate =
			localStorage.getItem(PREFS_STORAGE_KEY) !== null && !localStorage.getItem(MIGRATED_KEY);
		const keepLocal = new Set<keyof Prefs>(
			migrate ? remoteKeys.filter((key) => same(remote[key], DEFAULTS[key])) : dirtyKeys()
		);
		serverValues = remote;
		try {
			localStorage.setItem(MIGRATED_KEY, '1');
		} catch {
			// Storage unavailable: the upload just repeats next load.
		}
		for (const key of remoteKeys) {
			if (!keepLocal.has(key) && !LOCAL_ONLY.includes(key)) {
				(prefs as Record<string, unknown>)[key] = remote[key];
			}
		}
		cachePrefs();
		applyAppearance();
		if (dirtyKeys().length) flushPrefs();
	} catch {
		// Offline: keep the cached prefs.
	}
}

const darkQuery = window.matchMedia('(prefers-color-scheme: dark)');

/** Mirrors theme/density onto <html> and picks the favicon; app.html does the same before first paint. */
export function applyAppearance() {
	const dark = prefs.theme === 'dark' || (prefs.theme === 'system' && darkQuery.matches);
	document.documentElement.dataset.theme = dark ? 'dark' : 'light';
	document.documentElement.dataset.density = prefs.density;
	const favicon = document.querySelector<HTMLLinkElement>('link#favicon');
	if (favicon) favicon.href = prefs.iconStyle === 'glyphs' ? '/favicon-glyph.svg' : '/favicon.svg';
}

darkQuery.addEventListener('change', applyAppearance);

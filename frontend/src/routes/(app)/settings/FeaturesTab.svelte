<script lang="ts">
	import KindMark, { KIND_ICON } from '$lib/components/KindMark.svelte';
	import './settings.css';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { prefs, savePrefs } from '$lib/stores/prefs.svelte';
	import type { NodeKind } from '$lib/types/NodeKind';
	import type { StaleMode } from '$lib/types/StaleMode';

	const STALE_KINDS: { kind: NodeKind; label: string }[] = [
		{ kind: 'project', label: 'Projects' },
		{ kind: 'study', label: 'Studies' }
	];
	const STALE_MODE_ROWS: {
		key: 'staleQueued' | 'stalePaused' | 'staleIdeaStatus';
		label: string;
		help: string;
	}[] = [
		{
			key: 'staleQueued',
			label: 'Queued nodes',
			help: 'Never-poked ones count from their creation date.'
		},
		{ key: 'stalePaused', label: 'Paused nodes', help: 'Nodes you paused yourself.' },
		{ key: 'staleIdeaStatus', label: 'Idea-status nodes', help: 'Projects and studies at Idea.' }
	];

	/** Saves a day count typed into a number field, clamped to its range; junk snaps back. */
	function saveDays(
		key: 'staleDays' | 'staleWarnDays',
		input: HTMLInputElement,
		min: number,
		max: number
	) {
		const days = Math.round(Number(input.value));
		if (input.value.trim() === '' || !Number.isFinite(days)) {
			input.value = String(prefs[key]);
			return;
		}
		prefs[key] = Math.min(max, Math.max(min, days));
		input.value = String(prefs[key]);
		savePrefs();
	}

	function toggleStaleKind(kind: NodeKind) {
		prefs.staleKinds = prefs.staleKinds.includes(kind)
			? prefs.staleKinds.filter((existing) => existing !== kind)
			: [...prefs.staleKinds, kind];
		savePrefs();
	}
</script>

<section class="settings-section">
	<h2>Board</h2>
	<p class="explain">Where the board opens.</p>
	<div class="settings-row">
		<span>
			<span class="row-label">Default board view</span>
			<span class="row-help">Where the Board link in the sidebar lands.</span>
		</span>
		<SegmentedControl
			label="Default board view"
			value={prefs.defaultBoardView}
			onchange={(view) => {
				prefs.defaultBoardView = view;
				savePrefs();
			}}
			options={[
				{ value: 'organized', label: 'Organized' },
				{ value: 'canvas', label: 'Canvas' },
				{ value: 'timeline', label: 'Timeline' }
			]}
		/>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">Path status</span>
			<span class="row-help"
				>Auto: derived from the nodes inside. Manual: you set it, like any other node. Off: not
				shown.</span
			>
		</span>
		<SegmentedControl
			label="Path status"
			value={prefs.pathStatusMode}
			onchange={(mode) => {
				prefs.pathStatusMode = mode;
				savePrefs();
			}}
			options={[
				{ value: 'auto', label: 'Auto' },
				{ value: 'manual', label: 'Manual' },
				{ value: 'off', label: 'Off' }
			]}
		/>
	</div>
</section>

<section class="settings-section">
	<h2>Quick capture</h2>
	<p class="explain">Defaults for new nodes when you press N.</p>
	<div class="settings-row">
		<span>
			<span class="row-label">Quick capture kind</span>
			<span class="row-help">Preselected kind when you press N.</span>
		</span>
		<SegmentedControl
			label="Quick capture kind"
			value={prefs.captureKind}
			onchange={(kind) => {
				prefs.captureKind = kind;
				savePrefs();
			}}
			options={[
				{ value: 'idea', label: 'Idea', icon: KIND_ICON.idea },
				{ value: 'project', label: 'Project', icon: KIND_ICON.project },
				{ value: 'study', label: 'Study', icon: KIND_ICON.study },
				{ value: 'path', label: 'Path', icon: KIND_ICON.path }
			]}
		/>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">Quick capture focus</span>
			<span class="row-help">Preselected focus tier for new nodes.</span>
		</span>
		<SegmentedControl
			label="Quick capture focus"
			value={prefs.captureFocus}
			onchange={(focus) => {
				prefs.captureFocus = focus;
				savePrefs();
			}}
			options={[
				{ value: 'primary', label: 'Primary' },
				{ value: 'secondary', label: 'Secondary' },
				{ value: 'background', label: 'Background' }
			]}
		/>
	</div>
</section>

<section class="settings-section">
	<h2>Canvas</h2>
	<p class="explain">How the canvas places and draws things.</p>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">Auto-place on canvas</span>
				<span class="row-help"
					>Lay out new and unplaced nodes on the canvas automatically and pin them where they land.
					Off: they wait in the canvas's Unplaced panel until you drag them in.</span
				>
			</span>
			<input
				type="checkbox"
				checked={prefs.autoPlace}
				onchange={(event) => {
					prefs.autoPlace = event.currentTarget.checked;
					savePrefs();
				}}
			/>
		</label>
	</div>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">Grow paths to fit</span>
				<span class="row-help"
					>Dropping a node into a path box that's too small for it makes the box bigger.</span
				>
			</span>
			<input
				type="checkbox"
				checked={prefs.pathAutoExpand}
				onchange={(event) => {
					prefs.pathAutoExpand = event.currentTarget.checked;
					savePrefs();
				}}
			/>
		</label>
	</div>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">Connection labels</span>
				<span class="row-help"
					>Name each connection on the canvas (requires, next, related) in small text above its
					line.</span
				>
			</span>
			<input
				type="checkbox"
				checked={prefs.showEdgeLabels}
				onchange={(event) => {
					prefs.showEdgeLabels = event.currentTarget.checked;
					savePrefs();
				}}
			/>
		</label>
	</div>
</section>

<section class="settings-section">
	<h2>Pokes &amp; staleness</h2>
	<p class="explain">
		The "worked on this" log, and what happens to nodes it hasn't seen in a while.
	</p>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">Poke from cards</span>
				<span class="row-help">Show the Poke button on dashboard and board cards.</span>
			</span>
			<input
				type="checkbox"
				checked={prefs.pokeFromCards}
				onchange={(event) => {
					prefs.pokeFromCards = event.currentTarget.checked;
					savePrefs();
				}}
			/>
		</label>
	</div>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">Poke sets active</span>
				<span class="row-help"
					>A poke also switches the node's status to Active, and a poke that un-stales a paused node
					resumes it.</span
				>
			</span>
			<input
				type="checkbox"
				checked={prefs.pokeSetsActive}
				onchange={(event) => {
					prefs.pokeSetsActive = event.currentTarget.checked;
					savePrefs();
				}}
			/>
		</label>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">Stale after</span>
			<span class="row-help"
				>Days without a poke (or since creation) before a node goes stale. Only a poke within this
				window - live or backdated - un-stales it.</span
			>
		</span>
		<label class="days">
			<input
				class="field"
				type="number"
				min="1"
				max="365"
				aria-label="Stale after (days)"
				value={prefs.staleDays}
				onchange={(event) => saveDays('staleDays', event.currentTarget, 1, 365)}
			/>
			<span>days</span>
		</label>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">Warn before stale</span>
			<span class="row-help"
				>The dashboard lists nodes this many days before they go stale. 0 turns the warning off.</span
			>
		</span>
		<label class="days">
			<input
				class="field"
				type="number"
				min="0"
				max="365"
				aria-label="Warn before stale (days)"
				value={prefs.staleWarnDays}
				onchange={(event) => saveDays('staleWarnDays', event.currentTarget, 0, 365)}
			/>
			<span>days</span>
		</label>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">Kinds that can go stale</span>
			<span class="row-help"
				>Ideas never do - they're just ideas. Done and archived nodes never do.</span
			>
		</span>
		<div class="kinds" role="group" aria-label="Kinds that can go stale">
			{#each STALE_KINDS as { kind, label } (kind)}
				<button
					type="button"
					class="kind"
					class:selected={prefs.staleKinds.includes(kind)}
					aria-pressed={prefs.staleKinds.includes(kind)}
					onclick={() => toggleStaleKind(kind)}
				>
					<KindMark {kind} size={12} />
					{label}
				</button>
			{/each}
		</div>
	</div>
	{#each STALE_MODE_ROWS as row (row.key)}
		<div class="settings-row">
			<span>
				<span class="row-label">Stale {row.label.toLowerCase()}</span>
				<span class="row-help">{row.help} Active nodes always can go stale.</span>
			</span>
			<select
				aria-label="Stale {row.label.toLowerCase()}"
				value={prefs[row.key]}
				onchange={(event) => {
					prefs[row.key] = event.currentTarget.value as StaleMode;
					savePrefs();
				}}
			>
				<option value="always">Always</option>
				<option value="poked">Only if poked before</option>
				<option value="never">Never</option>
			</select>
		</div>
	{/each}
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">Pause stale nodes</span>
				<span class="row-help"
					>An active node that goes stale is paused (once - resuming it by hand doesn't re-pause
					it). It stays marked stale until poked.</span
				>
			</span>
			<input
				type="checkbox"
				checked={prefs.staleAutoPause}
				onchange={(event) => {
					prefs.staleAutoPause = event.currentTarget.checked;
					savePrefs();
				}}
			/>
		</label>
	</div>
	{#if prefs.staleAutoPause}
		<div class="settings-row">
			<span>
				<span class="row-label">End the active period</span>
				<span class="row-help"
					>With active periods tracked: at the last poke (only real work counts), or when the node
					went stale (the idle stretch stays in).</span
				>
			</span>
			<SegmentedControl
				label="End the active period"
				value={prefs.stalePausePeriodEnd}
				onchange={(end) => {
					prefs.stalePausePeriodEnd = end;
					savePrefs();
				}}
				options={[
					{ value: 'last_poke', label: 'At last poke' },
					{ value: 'stale_moment', label: 'When stale' }
				]}
			/>
		</div>
	{/if}
	<div class="settings-row">
		<span>
			<span class="row-label">Stale paths</span>
			<span class="row-help"
				>Any: the latest poke inside is past the threshold. All: every node in play inside is stale
				(done ones, and unstale idea/queued/paused ones, don't count). Off: paths never go stale.</span
			>
		</span>
		<SegmentedControl
			label="Stale paths"
			value={prefs.pathStaleMode}
			onchange={(mode) => {
				prefs.pathStaleMode = mode;
				savePrefs();
			}}
			options={[
				{ value: 'any', label: 'Any' },
				{ value: 'all', label: 'All' },
				{ value: 'off', label: 'Off' }
			]}
		/>
	</div>
</section>

<section class="settings-section">
	<h2>Active periods</h2>
	<p class="explain">How status and focus changes are recorded on the timeline.</p>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">Track active periods</span>
				<span class="row-help">
					Pausing closes the current active period; reactivating starts a new one. Off: reactivating
					continues the last period.
				</span>
			</span>
			<input
				type="checkbox"
				checked={prefs.trackActivePeriods}
				onchange={(event) => {
					prefs.trackActivePeriods = event.currentTarget.checked;
					savePrefs();
				}}
			/>
		</label>
	</div>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">New period on focus change</span>
				<span class="row-help">
					Changing the focus of a node with an open active period ends it and starts a new one under
					the new focus, so the timeline shows focus over time. Off: periods keep their focus until
					you edit it.
				</span>
			</span>
			<input
				type="checkbox"
				checked={prefs.splitPeriodOnFocusChange}
				onchange={(event) => {
					prefs.splitPeriodOnFocusChange = event.currentTarget.checked;
					savePrefs();
				}}
			/>
		</label>
	</div>
</section>

<style>
	.days {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		font: 500 12px/1 var(--font-display);
		color: var(--ink-2);
	}

	.days input {
		width: 72px;
	}

	.kinds {
		display: flex;
		gap: 6px;
	}

	.kind {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 10px;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		font: 600 11px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.kind.selected {
		border-color: var(--ink);
		background: var(--ink);
		color: var(--on-ink);
	}
</style>

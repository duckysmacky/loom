<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import KindMark, { KIND_ICON } from '$lib/components/KindMark.svelte';
	import './settings.css';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { prefs, savePrefs } from '$lib/stores/prefs.svelte';
	import type { NodeKind } from '$lib/types/NodeKind';
	import type { StaleMode } from '$lib/types/StaleMode';

	const STALE_KINDS: { kind: NodeKind; label: string }[] = [
		{ kind: 'project', label: m.features_stale_projects() },
		{ kind: 'study', label: m.features_stale_studies() }
	];
	const STALE_MODE_ROWS: {
		key: 'staleQueued' | 'stalePaused' | 'staleIdeaStatus';
		label: string;
		help: string;
	}[] = [
		{
			key: 'staleQueued',
			label: m.features_stale_queued(),
			help: m.features_stale_queued_help()
		},
		{
			key: 'stalePaused',
			label: m.features_stale_paused(),
			help: m.features_stale_paused_help()
		},
		{
			key: 'staleIdeaStatus',
			label: m.features_stale_idea_status(),
			help: m.features_stale_idea_status_help()
		}
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
	<h2>{m.nav_board()}</h2>
	<p class="explain">{m.features_where_board_opens()}</p>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.features_default_board_view()}</span>
			<span class="row-help">{m.features_where_board_link_sidebar()}</span>
		</span>
		<SegmentedControl
			label={m.features_default_board_view()}
			value={prefs.defaultBoardView}
			onchange={(view) => {
				prefs.defaultBoardView = view;
				savePrefs();
			}}
			options={[
				{ value: 'organized', label: m.nav_organized() },
				{ value: 'canvas', label: m.nav_canvas() },
				{ value: 'timeline', label: m.nav_timeline() }
			]}
		/>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.features_path_status()}</span>
			<span class="row-help">{m.features_auto_derived_from_nodes()}</span>
		</span>
		<SegmentedControl
			label={m.features_path_status()}
			value={prefs.pathStatusMode}
			onchange={(mode) => {
				prefs.pathStatusMode = mode;
				savePrefs();
			}}
			options={[
				{ value: 'auto', label: m.features_auto() },
				{ value: 'manual', label: m.features_manual() },
				{ value: 'off', label: m.features_off() }
			]}
		/>
	</div>
</section>

<section class="settings-section">
	<h2>{m.quick_capture()}</h2>
	<p class="explain">{m.features_defaults_new_nodes_when()}</p>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.features_quick_capture_kind()}</span>
			<span class="row-help">{m.features_preselected_kind_when_you()}</span>
		</span>
		<SegmentedControl
			label={m.features_quick_capture_kind()}
			value={prefs.captureKind}
			onchange={(kind) => {
				prefs.captureKind = kind;
				savePrefs();
			}}
			options={[
				{ value: 'idea', label: m.kind_idea(), icon: KIND_ICON.idea },
				{ value: 'project', label: m.kind_project(), icon: KIND_ICON.project },
				{ value: 'study', label: m.kind_study(), icon: KIND_ICON.study },
				{ value: 'path', label: m.kind_path(), icon: KIND_ICON.path }
			]}
		/>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.features_quick_capture_focus()}</span>
			<span class="row-help">{m.features_preselected_focus_tier_new()}</span>
		</span>
		<SegmentedControl
			label={m.features_quick_capture_focus()}
			value={prefs.captureFocus}
			onchange={(focus) => {
				prefs.captureFocus = focus;
				savePrefs();
			}}
			options={[
				{ value: 'primary', label: m.focus_primary() },
				{ value: 'secondary', label: m.focus_secondary() },
				{ value: 'background', label: m.focus_background() }
			]}
		/>
	</div>
</section>

<section class="settings-section">
	<h2>{m.nav_canvas()}</h2>
	<p class="explain">{m.features_how_canvas_places_draws()}</p>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">{m.features_auto_place_canvas()}</span>
				<span class="row-help">{m.features_lay_out_new_unplaced()}</span>
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
				<span class="row-label">{m.features_snap_to_grid()}</span>
				<span class="row-help">{m.features_snap_to_grid_help()}</span>
			</span>
			<input
				type="checkbox"
				checked={prefs.canvasSnapToGrid}
				onchange={(event) => {
					prefs.canvasSnapToGrid = event.currentTarget.checked;
					savePrefs();
				}}
			/>
		</label>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.features_grid_snap_step()}</span>
			<span class="row-help">{m.features_grid_snap_step_help()}</span>
		</span>
		<select
			aria-label={m.controls_snap_step_label()}
			value={prefs.canvasSnapStep}
			disabled={!prefs.canvasSnapToGrid}
			onchange={(event) => {
				prefs.canvasSnapStep = Number(event.currentTarget.value);
				savePrefs();
			}}
		>
			<option value={2}>{m.snap_step_2()}</option>
			<option value={1}>{m.snap_step_1()}</option>
			<option value={0.5}>{m.snap_step_half()}</option>
			<option value={0.25}>{m.snap_step_quarter()}</option>
			<option value={0.125}>{m.snap_step_eighth()}</option>
		</select>
	</div>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">{m.features_grow_paths_fit()}</span>
				<span class="row-help">{m.features_dropping_node_into_path()}</span>
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
				<span class="row-label">{m.features_connection_labels()}</span>
				<span class="row-help">{m.features_name_each_connection_canvas()}</span>
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
	<h2>{m.features_pokes_staleness()}</h2>
	<p class="explain">
		{m.features_worked_this_log_what()}
	</p>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">{m.features_poke_from_cards()}</span>
				<span class="row-help">{m.features_show_poke_button_dashboard()}</span>
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
				<span class="row-label">{m.features_poke_sets_active()}</span>
				<span class="row-help">{m.features_poke_also_switches_node()}</span>
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
			<span class="row-label">{m.features_stale_after()}</span>
			<span class="row-help">{m.features_days_without_poke_since()}</span>
		</span>
		<label class="days">
			<input
				class="field"
				type="number"
				min="1"
				max="365"
				aria-label={m.features_stale_after_days()}
				value={prefs.staleDays}
				onchange={(event) => saveDays('staleDays', event.currentTarget, 1, 365)}
			/>
			<span>{m.features_days()}</span>
		</label>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.features_warn_before_stale()}</span>
			<span class="row-help">{m.features_dashboard_lists_nodes_this()}</span>
		</span>
		<label class="days">
			<input
				class="field"
				type="number"
				min="0"
				max="365"
				aria-label={m.features_warn_before_stale_days()}
				value={prefs.staleWarnDays}
				onchange={(event) => saveDays('staleWarnDays', event.currentTarget, 0, 365)}
			/>
			<span>{m.features_days()}</span>
		</label>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.features_kinds_that_can_go()}</span>
			<span class="row-help">{m.features_ideas_never_do_they()}</span>
		</span>
		<div class="kinds" role="group" aria-label={m.features_kinds_that_can_go()}>
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
				<span class="row-label">{row.label}</span>
				<span class="row-help">{row.help} {m.features_stale_active_always()}</span>
			</span>
			<select
				aria-label={row.label}
				value={prefs[row.key]}
				onchange={(event) => {
					prefs[row.key] = event.currentTarget.value as StaleMode;
					savePrefs();
				}}
			>
				<option value="always">{m.features_always()}</option>
				<option value="poked">{m.features_only_if_poked_before()}</option>
				<option value="never">{m.features_never()}</option>
			</select>
		</div>
	{/each}
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">{m.features_pause_stale_nodes()}</span>
				<span class="row-help">{m.features_active_node_that_goes()}</span>
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
				<span class="row-label">{m.features_end_active_period()}</span>
				<span class="row-help">{m.features_active_periods_tracked_at()}</span>
			</span>
			<SegmentedControl
				label={m.features_end_active_period()}
				value={prefs.stalePausePeriodEnd}
				onchange={(end) => {
					prefs.stalePausePeriodEnd = end;
					savePrefs();
				}}
				options={[
					{ value: 'last_poke', label: m.features_at_last_poke() },
					{ value: 'stale_moment', label: m.features_when_stale() }
				]}
			/>
		</div>
	{/if}
	<div class="settings-row">
		<span>
			<span class="row-label">{m.features_stale_paths()}</span>
			<span class="row-help">{m.features_any_latest_poke_inside()}</span>
		</span>
		<SegmentedControl
			label={m.features_stale_paths()}
			value={prefs.pathStaleMode}
			onchange={(mode) => {
				prefs.pathStaleMode = mode;
				savePrefs();
			}}
			options={[
				{ value: 'any', label: m.features_any() },
				{ value: 'all', label: m.features_all() },
				{ value: 'off', label: m.features_off() }
			]}
		/>
	</div>
</section>

<section class="settings-section">
	<h2>{m.periods_title()}</h2>
	<p class="explain">{m.features_how_status_focus_changes()}</p>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">{m.features_track_active_periods()}</span>
				<span class="row-help">
					{m.features_pausing_closes_current_active()}
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
				<span class="row-label">{m.features_new_period_focus_change()}</span>
				<span class="row-help">
					{m.features_changing_focus_node_open()}
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

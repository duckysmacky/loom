<script lang="ts">
	import './settings.css';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { prefs, savePrefs } from '$lib/stores/prefs.svelte';
	import type { StaleMode } from '$lib/types/StaleMode';
</script>

<section class="settings-section">
	<h2>Functionality</h2>
	<p class="explain">Saved in this browser only.</p>
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
				{ value: 'idea', label: 'Idea' },
				{ value: 'project', label: 'Project' },
				{ value: 'study', label: 'Study' },
				{ value: 'path', label: 'Path' }
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
				<span class="row-help">A poke also switches the node's status to Active.</span>
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
	<div class="settings-row">
		<span>
			<span class="row-label">Stale after</span>
			<span class="row-help"
				>Days without a poke before a node shows under "Haven't touched in a while" on the
				dashboard. Active nodes always can.</span
			>
		</span>
		<select
			aria-label="Stale after"
			value={prefs.staleDays}
			onchange={(event) => {
				prefs.staleDays = Number(event.currentTarget.value);
				savePrefs();
			}}
		>
			<option value="7">7 days</option>
			<option value="14">14 days</option>
			<option value="21">21 days</option>
			<option value="30">30 days</option>
			<option value="60">60 days</option>
		</select>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">Stale queued nodes</span>
			<span class="row-help"
				>Whether queued nodes appear there. Never-poked ones count from their creation date.</span
			>
		</span>
		<select
			aria-label="Stale queued nodes"
			value={prefs.staleQueued}
			onchange={(event) => {
				prefs.staleQueued = event.currentTarget.value as StaleMode;
				savePrefs();
			}}
		>
			<option value="always">Always</option>
			<option value="poked">Only if poked before</option>
			<option value="never">Never</option>
		</select>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">Stale paused nodes</span>
			<span class="row-help">Whether paused nodes appear there.</span>
		</span>
		<select
			aria-label="Stale paused nodes"
			value={prefs.stalePaused}
			onchange={(event) => {
				prefs.stalePaused = event.currentTarget.value as StaleMode;
				savePrefs();
			}}
		>
			<option value="always">Always</option>
			<option value="poked">Only if poked before</option>
			<option value="never">Never</option>
		</select>
	</div>
</section>

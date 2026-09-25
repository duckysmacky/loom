<script lang="ts">
	import './settings.css';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { prefs, savePrefs } from '$lib/stores/prefs.svelte';
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
					Pausing and reactivating a node records a period on its timeline row, instead of just the
					first start → finish span.
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
</section>

<style>
	.toggle {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		cursor: pointer;
	}

	.toggle input {
		width: 18px;
		height: 18px;
		accent-color: var(--accent);
	}
</style>

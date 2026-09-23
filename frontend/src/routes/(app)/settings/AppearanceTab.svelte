<script lang="ts">
	import './settings.css';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { prefs, savePrefs } from '$lib/stores/prefs.svelte';
</script>

<section class="settings-section">
	<h2>Appearance</h2>
	<p class="explain">Saved in this browser only.</p>
	<div class="settings-row">
		<span>
			<span class="row-label">Theme</span>
			<span class="row-help">System follows your OS light/dark setting.</span>
		</span>
		<SegmentedControl
			label="Theme"
			value={prefs.theme}
			onchange={(theme) => {
				prefs.theme = theme;
				savePrefs();
			}}
			options={[
				{ value: 'system', label: 'System' },
				{ value: 'light', label: 'Light' },
				{ value: 'dark', label: 'Dark' }
			]}
		/>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">Density</span>
			<span class="row-help">Compact tightens card and table padding.</span>
		</span>
		<SegmentedControl
			label="Density"
			value={prefs.density}
			onchange={(density) => {
				prefs.density = density;
				savePrefs();
			}}
			options={[
				{ value: 'comfortable', label: 'Comfortable' },
				{ value: 'compact', label: 'Compact' }
			]}
		/>
	</div>
</section>

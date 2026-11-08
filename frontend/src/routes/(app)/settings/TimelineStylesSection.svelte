<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import './settings.css';
	import AccentPicker from '$lib/components/ui/AccentPicker.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { DEFAULT_TIMELINE_STYLES } from '$lib/graph/timeline';
	import { prefs, savePrefs } from '$lib/stores/prefs.svelte';
	import type { FocusStyle } from '$lib/types/FocusStyle';
	import type { NodeFocus } from '$lib/types/NodeFocus';
	import type { TimelineStyles } from '$lib/types/TimelineStyles';

	const tiers: { tier: NodeFocus; label: () => string; help: () => string }[] = [
		{ tier: 'primary', label: m.timeline_style_primary, help: m.timeline_style_primary_help },
		{ tier: 'secondary', label: m.timeline_style_secondary, help: m.timeline_style_secondary_help },
		{
			tier: 'background',
			label: m.timeline_style_background,
			help: m.timeline_style_background_help
		}
	];

	// Replaced, never edited in place: the defaults object is shared.
	function update(changes: Partial<TimelineStyles>) {
		prefs.timelineStyles = { ...prefs.timelineStyles, ...changes };
		savePrefs();
	}

	function updateTier(tier: NodeFocus, changes: Partial<FocusStyle>) {
		update({ [tier]: { ...prefs.timelineStyles[tier], ...changes } });
	}

	function reset() {
		prefs.timelineStyles = DEFAULT_TIMELINE_STYLES;
		savePrefs();
	}
</script>

<section class="settings-section">
	<h2>{m.timeline_style_title()}</h2>
	<p class="explain">{m.timeline_style_explain()}</p>
	{#each tiers as { tier, label, help } (tier)}
		{@const style = prefs.timelineStyles[tier]}
		<div class="settings-row">
			<span>
				<span class="row-label">{label()}</span>
				<span class="row-help">{help()}</span>
			</span>
			<div class="controls">
				<AccentPicker
					size="small"
					value={style.color}
					onchange={(color) => updateTier(tier, { color })}
				/>
				<SegmentedControl
					label={m.timeline_style_fill()}
					value={style.fill}
					onchange={(fill) => updateTier(tier, { fill })}
					options={[
						{ value: 'solid', label: m.timeline_style_fill_solid() },
						{ value: 'striped', label: m.timeline_style_fill_striped() },
						{ value: 'outline', label: m.timeline_style_fill_outline() }
					]}
				/>
			</div>
		</div>
	{/each}
	<div class="settings-row">
		<span>
			<span class="row-label">{m.timeline_style_done()}</span>
			<span class="row-help">{m.timeline_style_done_help()}</span>
		</span>
		<div class="controls">
			<AccentPicker
				size="small"
				value={prefs.timelineStyles.done_color}
				onchange={(done_color) => update({ done_color })}
			/>
		</div>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.timeline_style_blocked()}</span>
			<span class="row-help">{m.timeline_style_blocked_help()}</span>
		</span>
		<div class="controls">
			<AccentPicker
				size="small"
				value={prefs.timelineStyles.blocked_color}
				onchange={(blocked_color) => update({ blocked_color })}
			/>
		</div>
	</div>
	<div class="settings-row">
		<label class="toggle">
			<span>
				<span class="row-label">{m.timeline_style_paused()}</span>
				<span class="row-help">{m.timeline_style_paused_help()}</span>
			</span>
			<input
				type="checkbox"
				checked={prefs.timelineStyles.show_paused}
				onchange={(event) => update({ show_paused: event.currentTarget.checked })}
			/>
		</label>
	</div>
	<div class="settings-row">
		<span class="row-help">{m.edges_color_hint()}</span>
		<Button onclick={reset}>{m.edges_reset()}</Button>
	</div>
</section>

<style>
	.controls {
		display: flex;
		align-items: center;
		gap: 12px 16px;
		flex-wrap: wrap;
		flex-basis: 100%;
	}
</style>

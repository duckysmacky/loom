<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import './settings.css';
	import AccentPicker from '$lib/components/ui/AccentPicker.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { DEFAULT_EDGE_STYLES, type EdgeStyleKey } from '$lib/graph/edgeStyle';
	import { prefs, savePrefs } from '$lib/stores/prefs.svelte';
	import type { EdgeStyle } from '$lib/types/EdgeStyle';

	const rows: { key: EdgeStyleKey; label: () => string; help: () => string }[] = [
		{ key: 'related', label: m.edges_related, help: m.edges_related_help },
		{ key: 'precedes', label: m.edges_precedes, help: m.edges_precedes_help },
		{ key: 'requires_unmet', label: m.edges_requires_unmet, help: m.edges_requires_unmet_help },
		{ key: 'requires_met', label: m.edges_requires_met, help: m.edges_requires_met_help }
	];

	// Replaced, never edited in place: the defaults object is shared.
	function update(key: EdgeStyleKey, changes: Partial<EdgeStyle>) {
		prefs.edgeStyles = { ...prefs.edgeStyles, [key]: { ...prefs.edgeStyles[key], ...changes } };
		savePrefs();
	}

	function reset() {
		prefs.edgeStyles = DEFAULT_EDGE_STYLES;
		savePrefs();
	}
</script>

<section class="settings-section">
	<h2>{m.edges_title()}</h2>
	<p class="explain">{m.edges_explain()}</p>
	{#each rows as { key, label, help } (key)}
		{@const style = prefs.edgeStyles[key]}
		<div class="settings-row">
			<span>
				<span class="row-label">{label()}</span>
				<span class="row-help">{help()}</span>
			</span>
			<div class="controls">
				<AccentPicker
					size="small"
					value={style.color}
					onchange={(color) => update(key, { color })}
				/>
				<SegmentedControl
					label={m.edges_line()}
					value={style.line}
					onchange={(line) => update(key, { line })}
					options={[
						{ value: 'solid', label: m.edges_line_solid() },
						{ value: 'dashed', label: m.edges_line_dashed() },
						{ value: 'dotted', label: m.edges_line_dotted() }
					]}
				/>
				<SegmentedControl
					label={m.edges_width()}
					value={style.width}
					onchange={(width) => update(key, { width })}
					options={[
						{ value: 'thin', label: m.edges_width_thin() },
						{ value: 'normal', label: m.edges_width_normal() },
						{ value: 'thick', label: m.edges_width_thick() }
					]}
				/>
				<label class="arrow-toggle">
					<input
						type="checkbox"
						checked={style.arrow}
						onchange={(event) => update(key, { arrow: event.currentTarget.checked })}
					/>
					{m.edges_arrow()}
				</label>
			</div>
		</div>
	{/each}
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

	.arrow-toggle {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font: 600 12px/1 var(--font-display);
		cursor: pointer;
	}

	.arrow-toggle input {
		width: 16px;
		height: 16px;
		accent-color: var(--accent);
	}
</style>

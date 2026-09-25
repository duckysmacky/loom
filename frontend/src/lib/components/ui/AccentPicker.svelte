<script lang="ts">
	import { ACCENT_PALETTE } from '$lib/graph/display';
	import { addRecentColor, prefs } from '$lib/stores/prefs.svelte';

	/**
	 * Accent color picker: the fixed design-system palette, then any custom
	 * colors picked via the native input (most recent first, shared across
	 * every picker in the app since they read the same prefs), then the
	 * native picker itself. Clicking the already-chosen swatch clears it.
	 */
	let {
		value,
		onchange,
		size = 'default'
	}: {
		value: string | null;
		onchange: (color: string | null) => void;
		size?: 'default' | 'small';
	} = $props();

	const recent = $derived(
		prefs.recentColors.filter((color) => !(ACCENT_PALETTE as readonly string[]).includes(color))
	);

	function toggle(color: string) {
		onchange(value === color ? null : color);
	}

	function pickCustom(event: Event) {
		const color = (event.currentTarget as HTMLInputElement).value;
		addRecentColor(color);
		onchange(color);
	}
</script>

<div class="swatches" class:small={size === 'small'}>
	{#each ACCENT_PALETTE as color (color)}
		<button
			type="button"
			class="swatch"
			class:chosen={value === color}
			style:background={color}
			aria-label="Accent {color}"
			aria-pressed={value === color}
			onclick={() => toggle(color)}
		></button>
	{/each}
	{#each recent as color (color)}
		<button
			type="button"
			class="swatch"
			class:chosen={value === color}
			style:background={color}
			aria-label="Recent accent {color}"
			aria-pressed={value === color}
			onclick={() => toggle(color)}
		></button>
	{/each}
	<label class="swatch custom" aria-label="Pick a custom color">
		<input type="color" value={value ?? '#888888'} onchange={pickCustom} />
	</label>
</div>

<style>
	.swatches {
		display: flex;
		gap: 5px;
		flex-wrap: wrap;
	}

	.swatch {
		width: 20px;
		height: 20px;
		border: var(--border-width) solid transparent;
		padding: 0;
	}

	.swatches.small .swatch {
		width: 16px;
		height: 16px;
	}

	.swatch.chosen {
		border-color: var(--ink);
		outline: 1px solid var(--surface);
		outline-offset: -4px;
	}

	.custom {
		position: relative;
		overflow: hidden;
		cursor: pointer;
		background: conic-gradient(from 90deg, #e05d5d, #e0c15d, #7bc96f, #5d9de0, #a05de0, #e05d5d);
	}

	.custom input {
		position: absolute;
		inset: -4px;
		width: calc(100% + 8px);
		height: calc(100% + 8px);
		padding: 0;
		border: none;
		opacity: 0;
		cursor: pointer;
	}
</style>

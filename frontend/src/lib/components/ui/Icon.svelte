<script lang="ts">
	import type { LucideIcon } from '@lucide/svelte';
	import { prefs } from '$lib/stores/prefs.svelte';

	/**
	 * A Lucide icon, following the Icons / Glyphs appearance pref: in glyph
	 * mode it shows `glyph` (a typographic mark) or nothing. `always` keeps the
	 * icon in both modes - for icon-only buttons that have no text to fall back on.
	 */
	let {
		icon: IconComponent,
		glyph,
		always = false,
		size = 14
	}: { icon: LucideIcon; glyph?: string; always?: boolean; size?: number } = $props();
</script>

{#if always || prefs.iconStyle === 'icons'}
	<IconComponent {size} strokeWidth={1.75} aria-hidden="true" />
{:else if glyph}
	<span class="glyph" aria-hidden="true">{glyph}</span>
{/if}

<style>
	/* Square caps and mitred joins: Lucide in Loom's blueprint line. */
	:global(.lucide) {
		flex: none;
		vertical-align: -0.15em;
		stroke-linecap: square;
		stroke-linejoin: miter;
	}
</style>

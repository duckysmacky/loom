<script lang="ts">
	let {
		label,
		color = null,
		onremove
	}: { label: string; color?: string | null; onremove?: () => void } = $props();
</script>

<span class="chip">
	{#if color}<span class="swatch" style:background={color}></span>{/if}
	{label}
	{#if onremove}
		<button type="button" class="remove" aria-label="Remove {label}" onclick={onremove}>✕</button>
	{/if}
</span>

<style>
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 5px 8px;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		font: 400 10px/1 var(--font-mono);
		color: var(--ink-2);
		white-space: nowrap;
	}

	.swatch {
		width: 7px;
		height: 7px;
		flex: none;
	}

	.remove {
		border: none;
		background: none;
		padding: 0 0 0 2px;
		font-size: 9px;
		color: var(--ink-2);
	}
</style>

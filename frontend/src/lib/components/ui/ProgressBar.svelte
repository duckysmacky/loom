<script lang="ts">
	let { value, total, height = 6 }: { value: number; total: number; height?: number } = $props();

	const percent = $derived(total > 0 ? Math.min(100, (value / total) * 100) : 0);
</script>

<!-- Two flat segments split at the exact percentage; always pair with a text
readout, the bar alone doesn't carry the count. -->
<div
	class="track"
	style:height="{height}px"
	role="progressbar"
	aria-valuemin={0}
	aria-valuemax={total}
	aria-valuenow={value}
>
	<div class="fill" style:width="{percent}%"></div>
</div>

<style>
	.track {
		flex: 1;
		min-width: 40px;
		background: var(--surface-2);
		box-shadow: inset 0 0 0 1px var(--chip);
	}

	.fill {
		height: 100%;
		background: var(--accent);
		transition: width 400ms var(--ease);
	}
</style>

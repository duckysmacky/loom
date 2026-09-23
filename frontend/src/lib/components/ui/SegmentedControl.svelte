<script lang="ts" generics="Value extends string">
	let {
		options,
		value,
		onchange,
		label
	}: {
		options: { value: Value; label: string }[];
		value: Value;
		onchange: (value: Value) => void;
		label: string;
	} = $props();
</script>

<div class="segmented" role="radiogroup" aria-label={label}>
	{#each options as option (option.value)}
		<button
			type="button"
			role="radio"
			aria-checked={option.value === value}
			class:selected={option.value === value}
			onclick={() => onchange(option.value)}>{option.label}</button
		>
	{/each}
</div>

<style>
	.segmented {
		display: inline-flex;
		background: var(--chip);
		padding: 3px;
	}

	button {
		border: none;
		background: none;
		padding: 7px 13px;
		font: 600 12px/1 var(--font-display);
		color: var(--ink-2);
		transition:
			background-color var(--fast) var(--ease),
			color var(--fast) var(--ease);
	}

	.selected {
		background: var(--surface);
		color: var(--ink);
		font-weight: 700;
	}
</style>

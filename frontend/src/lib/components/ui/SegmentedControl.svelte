<script lang="ts" generics="Value extends string">
	import type { LucideIcon } from '@lucide/svelte';
	import Icon from './Icon.svelte';

	let {
		options,
		value,
		onchange,
		label
	}: {
		options: { value: Value; label: string; icon?: LucideIcon }[];
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
			onclick={() => onchange(option.value)}
			>{#if option.icon}<Icon icon={option.icon} size={13} />{/if}{option.label}</button
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
		display: inline-flex;
		align-items: center;
		gap: 6px;
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

<script lang="ts" generics="Value extends string">
	let {
		tabs,
		value,
		onchange
	}: {
		tabs: { value: Value; label: string; count?: number }[];
		value: Value;
		onchange: (value: Value) => void;
	} = $props();
</script>

<div class="tabs" role="tablist">
	{#each tabs as tab (tab.value)}
		<button
			type="button"
			role="tab"
			aria-selected={tab.value === value}
			class:selected={tab.value === value}
			onclick={() => onchange(tab.value)}
		>
			{tab.label}
			{#if tab.count !== undefined}<span class="count">{tab.count}</span>{/if}
		</button>
	{/each}
</div>

<style>
	.tabs {
		display: flex;
		align-items: flex-end;
		gap: 24px;
		padding: 0 22px;
		background: var(--surface);
		border-bottom: var(--border-width-hair) solid var(--line);
		overflow-x: auto;
	}

	button {
		border: none;
		background: none;
		padding: 12px 0;
		border-bottom: var(--border-width) solid transparent;
		transition:
			border-color var(--normal) var(--ease),
			color var(--normal) var(--ease);
		font: 600 13.5px/1 var(--font-display);
		color: var(--ink-2);
		white-space: nowrap;
	}

	.selected {
		border-bottom-color: var(--accent);
		color: var(--ink);
		font-weight: 700;
	}

	.count {
		color: var(--ink-2);
		font-weight: 600;
		margin-left: 2px;
	}
</style>

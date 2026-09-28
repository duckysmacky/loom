<script lang="ts">
	/**
	 * Expand/collapse switch for a path. Stops the click from reaching the
	 * card, header or canvas node it sits in (those open the node).
	 */
	let { collapsed, ontoggle }: { collapsed: boolean; ontoggle: () => void } = $props();
</script>

<button
	type="button"
	class="collapse-toggle nodrag"
	aria-expanded={!collapsed}
	aria-label={collapsed ? 'Expand path' : 'Collapse path'}
	onclick={(event) => {
		event.stopPropagation();
		ontoggle();
	}}
>
	<span class="glyph" aria-hidden="true">{collapsed ? '+' : '−'}</span>
</button>

<style>
	.collapse-toggle {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 6px;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		font: 600 10.5px/1 var(--font-mono);
		color: var(--ink-2);
		cursor: pointer;
	}

	.collapse-toggle:hover {
		border-color: var(--ink-2);
		color: var(--ink);
	}

	.glyph {
		font-size: 11px;
	}
</style>

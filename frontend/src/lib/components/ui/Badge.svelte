<script lang="ts">
	import type { NodeStatus } from '$lib/types/NodeStatus';

	/** `blocked` is derived (never stored) and overrides the stored status. */
	let { status, blocked = false }: { status: NodeStatus; blocked?: boolean } = $props();

	const shown = $derived(blocked && status !== 'done' ? 'blocked' : status);
</script>

<span class="badge {shown}">{shown}</span>

<style>
	.badge {
		display: inline-block;
		padding: 4px 9px;
		font: 700 10px/1 var(--font-mono);
		letter-spacing: 0.1em;
		text-transform: uppercase;
		white-space: nowrap;
		border: var(--border-width-hair) solid var(--line);
		color: var(--ink-2);
	}

	.active {
		background: var(--ink);
		border-color: var(--ink);
		color: var(--on-ink);
	}

	.done {
		background: var(--ok);
		border-color: var(--ok);
		color: var(--on-ok);
	}

	.blocked {
		background: var(--warn);
		border-color: var(--warn);
		color: var(--on-warn);
	}

	.idea {
		border-style: dashed;
	}

	.archived {
		color: var(--ink-2);
		background: var(--surface-2);
	}
</style>

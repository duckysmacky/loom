<script lang="ts">
	import { Panel } from '@xyflow/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { page } from '$app/state';
	import { nodesApi } from '$lib/api/endpoints';
	import { closeNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { notify } from '$lib/stores/toasts.svelte';
	import type { LoomFlowNode } from './types';

	/** Actions for the nodes selected on the canvas. */
	let { selected }: { selected: LoomFlowNode[] } = $props();

	let confirming = $state(false);
	const ids = $derived(selected.map((flowNode) => flowNode.id));
	const count = $derived(ids.length === 1 ? '1 node' : `${ids.length} nodes`);

	async function unplace() {
		const done = await graph.mutate(() =>
			Promise.all(ids.map((id) => nodesApi.update(id, { canvas_x: null, canvas_y: null })))
		);
		if (done) notify(`Sent ${count} to Unplaced`);
	}

	async function remove() {
		confirming = false;
		const doomed = ids;
		const done = await graph.mutate(async () => {
			for (const id of doomed) await nodesApi.remove(id);
			return true;
		});
		if (!done) return;
		if (doomed.includes(page.url.searchParams.get('node') ?? '')) closeNode();
		notify(`Deleted ${doomed.length === 1 ? '1 node' : `${doomed.length} nodes`}`);
	}
</script>

{#if selected.length}
	<Panel position="top-center">
		<div class="bar">
			<span class="count">{count} selected</span>
			<!-- With auto-place on, an unplaced node would be laid out again at once. -->
			{#if !prefs.autoPlace}
				<Button variant="quiet" onclick={unplace}>Unplace</Button>
			{/if}
			<Button variant="quiet" onclick={() => (confirming = true)}>Delete</Button>
		</div>
	</Panel>
{/if}

<Modal open={confirming} onclose={() => (confirming = false)} label="Delete nodes" width={440}>
	<div class="confirm">
		<div class="label">Delete {count}</div>
		<p>
			Deletes {ids.length === 1 ? 'it' : 'them'} for good, with {ids.length === 1 ? 'its' : 'their'}
			connections, checklists and pokes. A deleted path lets go of what's inside it.
		</p>
		<div class="actions">
			<Button variant="quiet" onclick={() => (confirming = false)}>Keep</Button>
			<Button variant="primary" onclick={remove}>Delete for good</Button>
		</div>
	</div>
</Modal>

<style>
	.bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 8px 6px 12px;
		background: var(--surface);
		border: var(--border-width) solid var(--frame);
	}

	.count {
		margin-right: 4px;
		font: 600 11.5px/1 var(--font-mono);
		color: var(--ink);
	}

	.confirm {
		padding: 20px 22px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.confirm p {
		margin: 0;
		font: 500 13.5px/1.45 var(--font-display);
		color: var(--ink-2);
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}
</style>

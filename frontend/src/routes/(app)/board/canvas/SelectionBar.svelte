<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Trash2, Undo2 } from '@lucide/svelte';
	import { Panel } from '@xyflow/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { page } from '$app/state';
	import { nodesApi } from '$lib/api/endpoints';
	import { closeNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { notify } from '$lib/stores/toasts.svelte';
	import type { LoomCardNode } from '$lib/components/canvas/types';

	/** Actions for the nodes selected on the canvas. */
	let { selected }: { selected: LoomCardNode[] } = $props();

	let confirming = $state(false);
	const ids = $derived(selected.map((flowNode) => flowNode.id));
	const count = $derived(m.node_count({ count: ids.length }));

	async function unplace() {
		const done = await graph.mutate(() =>
			Promise.all(ids.map((id) => nodesApi.update(id, { canvas_x: null, canvas_y: null })))
		);
		if (done) notify(m.selection_sent_unplaced({ nodes: count }));
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
		notify(m.selection_deleted({ nodes: m.node_count({ count: doomed.length }) }));
	}
</script>

{#if selected.length}
	<Panel position="top-center">
		<div class="bar">
			<span class="count">{m.selection_count({ nodes: count })}</span>
			<!-- With auto-place on, an unplaced node would be laid out again at once. -->
			{#if !prefs.autoPlace}
				<Button variant="quiet" onclick={unplace} icon={Undo2}>{m.selection_unplace()}</Button>
			{/if}
			<Button variant="quiet" onclick={() => (confirming = true)} icon={Trash2}
				>{m.action_delete()}</Button
			>
		</div>
	</Panel>
{/if}

<Modal
	open={confirming}
	onclose={() => (confirming = false)}
	label={m.selection_delete_title()}
	width={440}
>
	<div class="confirm">
		<div class="label">{m.selection_delete_heading({ nodes: count })}</div>
		<p>{m.selection_delete_body({ count: ids.length })}</p>
		<div class="actions">
			<Button variant="quiet" onclick={() => (confirming = false)}>{m.action_keep()}</Button>
			<Button variant="primary" onclick={remove} icon={Trash2}>{m.action_delete_for_good()}</Button>
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

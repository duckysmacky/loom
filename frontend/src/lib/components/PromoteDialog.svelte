<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { nodesApi } from '$lib/api/endpoints';
	import { graph } from '$lib/stores/graph.svelte';
	import { notify } from '$lib/stores/toasts.svelte';
	import type { NodeFocus } from '$lib/types/NodeFocus';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	/**
	 * Promotes a backlog idea in place - same row, same id, same edges - by
	 * setting its kind, status and focus tier in one PATCH.
	 */
	let { node, onclose }: { node: NodeResponse | null; onclose: () => void } = $props();

	let kind = $state<'project' | 'course'>('project');
	let status = $state<'queued' | 'active'>('queued');
	let focus = $state<NodeFocus>('secondary');
	let saving = $state(false);

	async function promote() {
		if (!node) return;
		saving = true;
		const updated = await graph.mutate(() => nodesApi.update(node!.id, { kind, status, focus }));
		saving = false;
		if (updated) {
			notify(`Promoted “${updated.title}” to ${kind}`);
			onclose();
		}
	}
</script>

<Modal open={node !== null} {onclose} label="Promote idea" width={420}>
	<div class="dialog">
		<div class="label">Promote idea</div>
		<div class="title">{node?.title}</div>

		<div class="field-group">
			<span class="label">Kind</span>
			<SegmentedControl
				label="Kind"
				value={kind}
				onchange={(value) => (kind = value)}
				options={[
					{ value: 'project', label: 'Project' },
					{ value: 'course', label: 'Course' }
				]}
			/>
		</div>
		<div class="field-group">
			<span class="label">Status</span>
			<SegmentedControl
				label="Status"
				value={status}
				onchange={(value) => (status = value)}
				options={[
					{ value: 'queued', label: 'Queued' },
					{ value: 'active', label: 'Active' }
				]}
			/>
		</div>
		<div class="field-group">
			<span class="label">Focus tier</span>
			<SegmentedControl
				label="Focus tier"
				value={focus}
				onchange={(value) => (focus = value)}
				options={[
					{ value: 'primary', label: 'Primary' },
					{ value: 'secondary', label: 'Secondary' },
					{ value: 'background', label: 'Background' }
				]}
			/>
		</div>

		<div class="actions">
			<Button variant="quiet" onclick={onclose}>Cancel</Button>
			<Button variant="primary" onclick={promote} disabled={saving}>Promote</Button>
		</div>
	</div>
</Modal>

<style>
	.dialog {
		padding: 20px 22px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.title {
		margin-top: -8px;
		font: 700 20px/1.2 var(--font-display);
	}

	.field-group {
		display: flex;
		flex-direction: column;
		gap: 8px;
		align-items: flex-start;
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		padding-top: 14px;
		border-top: var(--border-width-hair) solid var(--line);
	}
</style>

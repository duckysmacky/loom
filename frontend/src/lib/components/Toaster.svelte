<script lang="ts">
	import { dismiss, toasts } from '$lib/stores/toasts.svelte';
</script>

<div class="toaster" aria-live="polite">
	{#each toasts as toast (toast.id)}
		<div class="toast {toast.tone}" role={toast.tone === 'error' ? 'alert' : 'status'}>
			<span>{toast.message}</span>
			<button type="button" aria-label="Dismiss" onclick={() => dismiss(toast.id)}>✕</button>
		</div>
	{/each}
</div>

<style>
	.toaster {
		position: fixed;
		right: 16px;
		bottom: 16px;
		z-index: 100;
		display: flex;
		flex-direction: column;
		gap: 8px;
		max-width: min(380px, calc(100vw - 32px));
	}

	.toast {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 11px 13px;
		background: var(--ink);
		color: var(--on-ink);
		border: var(--border-width) solid var(--ink);
		font: 600 12.5px/1.4 var(--font-display);
	}

	.error {
		background: var(--warn-tint);
		color: var(--ink);
		border-color: var(--warn);
	}

	button {
		margin-left: auto;
		border: none;
		background: none;
		color: inherit;
		font-size: 11px;
		padding: 0;
	}
</style>

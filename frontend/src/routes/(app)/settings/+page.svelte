<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Tabs from '$lib/components/ui/Tabs.svelte';
	import { withParam } from '$lib/navigation';
	import AccountTab from './AccountTab.svelte';
	import AppearanceTab from './AppearanceTab.svelte';
	import FunctionalityTab from './FunctionalityTab.svelte';
	import TopicsTab from './TopicsTab.svelte';

	const TABS = [
		{ value: 'account', label: 'Account' },
		{ value: 'appearance', label: 'Appearance' },
		{ value: 'topics', label: 'Topics & tags' },
		{ value: 'functionality', label: 'Functionality' },
		{ value: 'connections', label: 'Connections' }
	] as const;
	type Tab = (typeof TABS)[number]['value'];

	const tab = $derived<Tab>(
		TABS.find((candidate) => candidate.value === page.url.searchParams.get('tab'))?.value ??
			'account'
	);
</script>

<Tabs
	tabs={[...TABS]}
	value={tab}
	onchange={(next) => goto(withParam('tab', next), { noScroll: true, keepFocus: true })}
/>

<div class="page">
	{#if tab === 'account'}
		<AccountTab />
	{:else if tab === 'appearance'}
		<AppearanceTab />
	{:else if tab === 'topics'}
		<TopicsTab />
	{:else if tab === 'functionality'}
		<FunctionalityTab />
	{:else}
		<section class="panel">
			<h2>Connections</h2>
			<p class="muted">Reserved for linking Loom to outside services. Nothing to connect yet.</p>
		</section>
	{/if}
</div>

<style>
	.page {
		padding: var(--page-pad);
		max-width: 760px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	h2 {
		margin: 0 0 6px;
		font: 700 15px/1.2 var(--font-display);
	}

	p {
		margin: 0;
	}
</style>

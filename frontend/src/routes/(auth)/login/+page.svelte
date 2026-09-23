<script lang="ts">
	import { login } from '$lib/stores/session.svelte';
	import AuthForm from '../AuthForm.svelte';
</script>

<svelte:head><title>Sign in · Loom</title></svelte:head>

<AuthForm
	heading="Sign in"
	submitLabel="Sign in"
	passwordAutocomplete="current-password"
	onsubmit={login}
>
	{#snippet footer()}No account yet? <a href="/signup">Create one</a>{/snippet}
</AuthForm>

<script lang="ts">
	import { signup } from '$lib/stores/session.svelte';
	import AuthForm from '../AuthForm.svelte';
</script>

<svelte:head><title>Create account · Loom</title></svelte:head>

<AuthForm
	heading="Create account"
	submitLabel="Create account"
	passwordAutocomplete="new-password"
	onsubmit={signup}
>
	{#snippet footer()}
		Already have an account? <a href="/login">Sign in</a>
		<br />Password: 8–128 characters.
	{/snippet}
</AuthForm>

import { goto } from '$app/navigation';
import { refreshAccessToken, setAccessToken, setSessionExpiredHandler } from '$lib/api/client';
import { authApi } from '$lib/api/endpoints';
import type { AuthResponse } from '$lib/types/AuthResponse';
import type { AuthUserView } from '$lib/types/AuthUserView';
import { graph } from './graph.svelte';

/** Signed-in user; `ready` flips once the boot-time refresh attempt settles. */
export const session = $state({
	user: null as AuthUserView | null,
	ready: false
});

setSessionExpiredHandler(() => {
	session.user = null;
	graph.reset();
	goto('/login');
});

/** Restores a session from the refresh cookie, if there is a valid one. */
export async function bootstrapSession() {
	try {
		if (await refreshAccessToken()) session.user = await authApi.me();
	} catch {
		session.user = null;
	} finally {
		session.ready = true;
	}
}

function startSession(response: AuthResponse) {
	setAccessToken(response.access_token);
	session.user = response.user;
}

export async function login(email: string, password: string) {
	startSession(await authApi.login({ email, password }));
}

export async function signup(email: string, password: string) {
	startSession(await authApi.signup({ email, password }));
}

export async function logout() {
	await authApi.logout().catch(() => {});
	setAccessToken(null);
	session.user = null;
	graph.reset();
	await goto('/login');
}

/**
 * Thin fetch wrapper for the Loom REST API.
 *
 * The access token lives only in this module's memory - never in storage - and
 * is re-obtained from the httpOnly refresh cookie via `refreshAccessToken()` on
 * boot and whenever a protected request comes back 401.
 */

export class ApiError extends Error {
	constructor(
		readonly status: number,
		message: string
	) {
		super(message);
	}
}

let accessToken: string | null = null;
let refreshInFlight: Promise<string | null> | null = null;
let onSessionExpired: () => void = () => {};

export function setAccessToken(token: string | null) {
	accessToken = token;
}

export function setSessionExpiredHandler(handler: () => void) {
	onSessionExpired = handler;
}

async function postRefresh(): Promise<string | null> {
	try {
		const response = await fetch('/api/auth/refresh', { method: 'POST' });
		if (!response.ok) return null;
		const body: { access_token: string } = await response.json();
		return body.access_token;
	} catch {
		return null;
	}
}

/**
 * Exchanges the refresh cookie for a new access token. Single-flight: the
 * backend rotates the cookie on every call and treats a rotated-out cookie
 * being presented again as theft (revoking every session), so concurrent 401s
 * must share one refresh. `navigator.locks` extends that across browser tabs -
 * a second tab waits, then sends the already-rotated cookie.
 */
export function refreshAccessToken(): Promise<string | null> {
	refreshInFlight ??= (
		typeof navigator !== 'undefined' && navigator.locks
			? navigator.locks.request('loom-token-refresh', postRefresh)
			: postRefresh()
	)
		.then((token) => {
			accessToken = token;
			return token;
		})
		.finally(() => {
			refreshInFlight = null;
		});
	return refreshInFlight;
}

function send(method: string, path: string, body: unknown): Promise<Response> {
	const headers: Record<string, string> = {};
	if (body !== undefined) headers['Content-Type'] = 'application/json';
	if (accessToken) headers.Authorization = `Bearer ${accessToken}`;
	return fetch(`/api${path}`, {
		method,
		headers,
		body: body === undefined ? undefined : JSON.stringify(body)
	});
}

/**
 * Credential endpoints whose 401 means "bad credentials / no session", not "the
 * access token expired" - replaying them after a refresh would be pointless
 * (or, for refresh itself, recursive).
 */
const NO_RETRY_PATHS = new Set(['/auth/login', '/auth/signup', '/auth/refresh', '/auth/logout']);

/**
 * Sends one API request and returns the parsed JSON body (`undefined` for
 * 204). A 401 on a protected route triggers one refresh + replay.
 */
export async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
	let response = await send(method, path, body);

	if (response.status === 401 && !NO_RETRY_PATHS.has(path)) {
		const token = await refreshAccessToken();
		if (!token) {
			onSessionExpired();
			throw new ApiError(401, 'session expired');
		}
		response = await send(method, path, body);
	}

	if (!response.ok) {
		const errorBody: { error?: string } | null = await response.json().catch(() => null);
		throw new ApiError(response.status, errorBody?.error ?? response.statusText);
	}

	if (response.status === 204) return undefined as T;
	return response.json();
}

/** Builds `?a=1&b=2` from the defined, non-empty entries of `params`. */
export function queryString(params: Record<string, string | null | undefined>): string {
	const search = new URLSearchParams();
	for (const [key, value] of Object.entries(params)) {
		if (value) search.set(key, value);
	}
	const encoded = search.toString();
	return encoded ? `?${encoded}` : '';
}

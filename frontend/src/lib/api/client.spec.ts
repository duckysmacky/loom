import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
	ApiError,
	refreshAccessToken,
	request,
	setAccessToken,
	setSessionExpiredHandler
} from './client';

type Route = (init: RequestInit) => Response;

function json(status: number, body: unknown): Response {
	return new Response(body === null ? null : JSON.stringify(body), { status });
}

/** Stubs `fetch` with per-URL handlers and records every call made. */
function stubFetch(routes: Record<string, Route | Route[]>) {
	const calls: { url: string; authorization: string | undefined }[] = [];
	vi.stubGlobal(
		'fetch',
		vi.fn(async (url: string, init: RequestInit = {}) => {
			const headers = (init.headers ?? {}) as Record<string, string>;
			calls.push({ url, authorization: headers.Authorization });
			const route = routes[url];
			const handler = Array.isArray(route) ? route.shift() : route;
			if (!handler) throw new Error(`unexpected fetch ${url}`);
			return handler(init);
		})
	);
	return calls;
}

describe('request', () => {
	beforeEach(() => {
		setAccessToken('stale-token');
		setSessionExpiredHandler(() => {});
	});
	afterEach(() => vi.unstubAllGlobals());

	it('refreshes once on 401 and replays with the new token', async () => {
		const calls = stubFetch({
			'/api/nodes': [() => json(401, { error: 'expired' }), () => json(200, [{ id: 'n1' }])],
			'/api/auth/refresh': () => json(200, { access_token: 'fresh-token' })
		});

		const nodes = await request<{ id: string }[]>('GET', '/nodes');

		expect(nodes).toEqual([{ id: 'n1' }]);
		expect(calls.map((call) => call.url)).toEqual([
			'/api/nodes',
			'/api/auth/refresh',
			'/api/nodes'
		]);
		expect(calls[0].authorization).toBe('Bearer stale-token');
		expect(calls[2].authorization).toBe('Bearer fresh-token');
	});

	it('shares one refresh between concurrent 401s', async () => {
		const calls = stubFetch({
			'/api/nodes': [
				() => json(401, { error: 'expired' }),
				() => json(401, { error: 'expired' }),
				() => json(200, []),
				() => json(200, [])
			],
			'/api/auth/refresh': [() => json(200, { access_token: 'fresh-token' })]
		});

		await Promise.all([request('GET', '/nodes'), request('GET', '/nodes')]);

		expect(calls.filter((call) => call.url === '/api/auth/refresh')).toHaveLength(1);
	});

	it('never retries credential endpoints', async () => {
		const calls = stubFetch({
			'/api/auth/login': () => json(401, { error: 'invalid email or password' })
		});

		await expect(request('POST', '/auth/login', {})).rejects.toMatchObject({
			status: 401,
			message: 'invalid email or password'
		});
		expect(calls).toHaveLength(1);
	});

	it('reports an expired session when the refresh fails', async () => {
		const expired = vi.fn();
		setSessionExpiredHandler(expired);
		stubFetch({
			'/api/nodes': () => json(401, { error: 'expired' }),
			'/api/auth/refresh': () => json(401, { error: 'invalid token' })
		});

		await expect(request('GET', '/nodes')).rejects.toBeInstanceOf(ApiError);
		expect(expired).toHaveBeenCalledOnce();
	});

	it('surfaces the API error message and returns undefined for 204', async () => {
		stubFetch({
			'/api/edges': () => json(409, { error: 'edge would create a cycle' }),
			'/api/edges/e1': () => json(204, null)
		});

		await expect(request('POST', '/edges', {})).rejects.toMatchObject({
			status: 409,
			message: 'edge would create a cycle'
		});
		expect(await request('DELETE', '/edges/e1')).toBeUndefined();
	});
});

describe('refreshAccessToken', () => {
	afterEach(() => vi.unstubAllGlobals());

	it('resolves null when there is no valid refresh cookie', async () => {
		stubFetch({ '/api/auth/refresh': () => json(401, { error: 'invalid token' }) });
		expect(await refreshAccessToken()).toBeNull();
	});
});

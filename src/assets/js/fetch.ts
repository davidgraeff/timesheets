export type RequestInitTimeout =
    Omit<RequestInit, 'headers'>
    & { timeout?: number, no_auth?: boolean, headers?: Record<string, string> }

let authToken: string | null = null;

function setAuthHeader(token: string | null) {
    authToken = token;
}

/**
 * @param {RequestInfo} resource
 * @param {RequestInit | undefined} options
 */
async function fetchWithTimeout(resource: RequestInfo, options: RequestInitTimeout) {
    const {timeout = 2000} = options;
    if (!options.headers) {
        options.headers = {};
    }
    if (authToken && !options.no_auth) {
        options.headers["authorization"] = "Bearer " + authToken;
    }

    delete options.no_auth;

    const controller = new AbortController();
    const id = setTimeout(() => {
        console.error("fetch timeout", resource);
        controller.abort();
    }, timeout);

    const response = await fetch(resource, {
        ...options,
        signal: controller.signal
    });
    clearTimeout(id);

    if (!response.ok && response.status === 401) {
        setAuthHeader(null);
    }

    if (!response.ok) throw {code: response.status, err: await response.text()};

    return response;
}

async function post(url: string, mime: string | null = null, body: BodyInit | null = null, options: RequestInitTimeout = {}) {
    if (!options.headers) {
        options.headers = {};
    }
    if (mime) {
        options.headers['Content-Type'] = mime;
    }

    options.method = "POST";
    options.body = body;

    return await fetchWithTimeout(url, options).then(async val => {
        if (!val.ok) throw {code: val.status, err: await val.text()};
        return val.text();
    });
}


async function del(url: string, options: RequestInitTimeout = {}) {
    if (!options.headers) {
        options.headers = {};
    }
    options.method = "DELETE";

    return await fetchWithTimeout(url, options).then(async val => {
        if (!val.ok) throw {code: val.status, err: await val.text()};
        return val.text();
    });
}


async function getjson<T>(url: string, options: RequestInitTimeout = {}): Promise<T> {
    return await fetchWithTimeout(url, options).then(async val => {
        if (!val.ok) throw {code: val.status, err: await val.text()};
        return val.json();
    });
}

export {
    setAuthHeader,
    fetchWithTimeout,
    getjson,
    post,
    del
};

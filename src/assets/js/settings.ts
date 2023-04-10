import type {Writable} from 'svelte/store';
import {writable, get} from 'svelte/store'
import {browser} from '$app/environment';
import {getjson, post, setAuthHeader} from "./fetch";

const persistentStore = <T>(key: string, initValue: T): Writable<T> => {
    if (!browser) return writable(initValue);

    const storedValueStr = localStorage.getItem(key);
    const store = (storedValueStr != null) ? writable(JSON.parse(storedValueStr)) : writable(initValue);

    store.subscribe((val) => {
        // console.log("STORE WRITE", key, val);
        if ([null, undefined].includes(val)) {
            localStorage.removeItem(key)
        } else {
            localStorage.setItem(key, JSON.stringify(val))
        }
    })

    window.addEventListener('storage', (event) => {
        if (event.key != key) return;
        const storedValueStr = localStorage.getItem(key);
        if (storedValueStr == null) return;

        const localValue: T = JSON.parse(storedValueStr)
        if (localValue !== get(store)) store.set(localValue);
    });

    return store;
}


interface CloudSettings {
    cloud_url: string,
    cloud_api_key: string,
}

interface Settings {
    ics_url: string,
    ics_filter: string[],
    projects: string[],
    tags: string[],
    name: string,
    company: string,
    client: string,
    last_updated?: number,
    gitlab_url: string,
    gitlab_access_token: string
}

export const localSettings = persistentStore<Settings>("settings", {
    client: "",
    company: "",
    ics_filter: [],
    ics_url: "",
    name: "",
    projects: ["Agami", "Falco", "Rowi"],
    tags: ["meeting"],
    gitlab_url: "",
    gitlab_access_token: "",
    last_updated: 0
});

export const cloudSettings = persistentStore<CloudSettings>("cloud_settings", {
    cloud_api_key: "", cloud_url: ""
});

cloudSettings.subscribe(value => {
    setAuthHeader(value.cloud_api_key);
})


async function syncSettingsToCloud(cloudSettingsChanged: (arg0: CloudSettings) => Promise<void>) {
    const s = get(cloudSettings);
    const local: Settings = get(localSettings);
    local.last_updated = Math.round(Date.now() / 1000);
    await post(s.cloud_url + "/api/settings", "application/json", JSON.stringify(local));
    await cloudSettingsChanged(s);
}

async function syncSettings(cloudSettingsChanged: (arg0: CloudSettings) => Promise<void>) {
    const s = get(cloudSettings);
    const remoteSettings: Settings = await getjson(s.cloud_url + "/api/settings");
    localSettings.set(remoteSettings);
    await cloudSettingsChanged(s);
}

export {syncSettingsToCloud, syncSettings};

export type {Settings, CloudSettings};
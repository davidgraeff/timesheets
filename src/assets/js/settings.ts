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

function init_cloud_url() {
    let cloudUrl = "";
    if (browser) {
        const path = window.location.pathname.replace("/settings", "").replace("/print", "") + "/api";
        cloudUrl = document.location.protocol + "//" + window.location.hostname + ":" + window.location.port + path.replace("//", "/");
    }
    return cloudUrl;
}

export const cloudSettings = persistentStore<CloudSettings>("cloud_settings", {
    cloud_url: init_cloud_url(),
    cloud_api_key: ""
});

cloudSettings.subscribe(value => {
    if (!value.cloud_url)
        value.cloud_url = init_cloud_url();
    setAuthHeader(value.cloud_api_key);
})


async function syncSettingsToCloud(cloudSettingsChanged: (arg0: CloudSettings) => Promise<void>) {
    const local: Settings = get(localSettings);
    local.last_updated = Math.round(Date.now() / 1000);
    let cloud = get(cloudSettings);
    await post(cloud.cloud_url + "/settings", "application/json", JSON.stringify(local));
    await cloudSettingsChanged(cloud);
}

async function syncSettings(cloudSettingsChanged: (arg0: CloudSettings) => Promise<void>) {
    let cloud = get(cloudSettings);
    const remoteSettings: Settings = await getjson(cloud.cloud_url + "/settings");
    localSettings.set(remoteSettings);
    await cloudSettingsChanged(get(cloudSettings));
}

export {syncSettingsToCloud, syncSettings};

export type {Settings, CloudSettings};
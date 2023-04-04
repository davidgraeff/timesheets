<script lang="ts">
    import {Form, FormGroup, Input} from "sveltestrap";
    import MultiSelect from 'svelte-multiselect'
    import {onDestroy, onMount} from "svelte";
    import {localSettings, cloudSettings, type CloudSettings} from "../../assets/js/settings";
    import type {Settings} from "../../assets/js/settings";
    import {getjson, post, setAuthHeader} from "../../assets/js/fetch";
    import {get} from "svelte/store";
    import {syncSettings, syncSettingsToCloud} from "../../assets/js/settings.js";

    let newerCloudSettings = false;
    let newerLocalSettings = false;

    async function cloudSettingsChanged(s: CloudSettings) {
        const url = s.cloud_url + "/api/settings";
        const key = s.cloud_api_key;

        setAuthHeader(key);

        if (!key || !url) {
            newerCloudSettings = false;
            return;
        }

        const remoteSettings: Settings = await getjson<Settings>(url).catch(() => {
            return {
                client: "",
                company: "",
                ics_filter: [],
                ics_url: "",
                name: "",
                projects: [],
                tags: [],
                last_updated: 0
            };
        });
        const local: Settings = get(localSettings);
        if (!local.last_updated) local.last_updated = 0;
        newerCloudSettings = remoteSettings.last_updated !== undefined
            && remoteSettings.last_updated > local.last_updated;
        newerLocalSettings = remoteSettings.last_updated === undefined || remoteSettings.last_updated < local.last_updated;
    }

    onMount(() => {
        const s = get(cloudSettings);
        cloudSettingsChanged(s);
    });

</script>
<Form class="container container-lg">
    <div class="border p-3 mb-3">
        <h4>Cloud</h4>
        <FormGroup floating label="Cloud URL">
            <Input placeholder="Enter a value" bind:value={$cloudSettings.cloud_url}
                   on:blur={() => cloudSettingsChanged($cloudSettings)}/>
        </FormGroup>
        <FormGroup floating label="Cloud API Key">
            <Input placeholder="Enter a value" bind:value={$cloudSettings.cloud_api_key}
                   on:blur={() => cloudSettingsChanged($cloudSettings)}/>
        </FormGroup>
        {#if newerCloudSettings}
            <button type="button" class="btn btn-primary" on:click={() => syncSettings(cloudSettingsChanged)}>Sync
                Settings from Cloud
            </button>
        {/if}
    </div>
    <div class="border p-3 mb-3">
        <h4>Legal</h4>
        <FormGroup floating label="Full Name">
            <Input placeholder="Enter a value" bind:value={$localSettings.name}/>
        </FormGroup>
        <FormGroup floating label="Company Name">
            <Input placeholder="Enter a value" bind:value={$localSettings.company}/>
        </FormGroup>
        <FormGroup floating label="Client Name">
            <Input placeholder="Enter a value" bind:value={$localSettings.client}/>
        </FormGroup>
    </div>
    <div class="border p-3">
        <h4>Timesheets</h4>
        <FormGroup floating label="ICS Outlook Calendar Link">
            <Input placeholder="https://URL" bind:value={$localSettings.ics_url} on:change={() => newerLocalSettings = true}/>
        </FormGroup>
        <FormGroup floating label="Gitlab URL">
            <Input placeholder="https://URL" bind:value={$localSettings.gitlab_url} on:change={() => newerLocalSettings = true}/>
        </FormGroup>
        <FormGroup floating label="Gitlab API Token">
            <Input placeholder="https://URL" bind:value={$localSettings.gitlab_access_token} on:change={() => newerLocalSettings = true}/>
        </FormGroup>
        <div class="form-group mb-3">
            <label>ICS Filter</label>
            <MultiSelect allowUserOptions="append" bind:selected={$localSettings.ics_filter}
                         bind:options={$localSettings.ics_filter} on:change={() => newerLocalSettings = true}/>
        </div>
        <div class="form-group mb-3">
            <label>Projects</label>
            <MultiSelect allowUserOptions="append" bind:selected={$localSettings.projects}
                         bind:options={$localSettings.projects} on:change={() => newerLocalSettings = true}/>
        </div>
        <div class="form-group mb-3">
            <label>Tags</label>
            <MultiSelect allowUserOptions="append" bind:selected={$localSettings.tags}
                         bind:options={$localSettings.tags} on:change={() => newerLocalSettings = true}/>
        </div>
    </div>
    <div class="p-3">
        {#if newerLocalSettings}
            <button type="button" class="btn btn-primary" on:click={() => syncSettingsToCloud(cloudSettingsChanged)}>
                Sync Settings to Cloud
            </button>
        {/if}
    </div>
</Form>

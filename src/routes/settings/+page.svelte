<script lang="ts">
    import {Form, FormGroup, Input} from "sveltestrap";
    import MultiSelect from 'svelte-multiselect'
    import {onMount} from "svelte";
    import {localSettings, cloudSettings, type CloudSettings, cloudUrl} from "../../assets/js/settings";
    import type {Settings} from "../../assets/js/settings";
    import {getjson, setAuthHeader} from "../../assets/js/fetch";
    import {get} from "svelte/store";
    import {syncSettings, syncSettingsToCloud} from "../../assets/js/settings.js";

    let newerCloudSettings = false;
    let newerLocalSettings = false;

    async function cloudSettingsChanged(s: CloudSettings) {
        const key = s.cloud_api_key;

        setAuthHeader(key);

        if (!key || !cloudUrl) {
            newerCloudSettings = false;
            return;
        }

        const remoteSettings: Settings = await getjson<Settings>(cloudUrl + "/settings").catch(() => {
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
        if (!remoteSettings.last_updated) remoteSettings.last_updated = 0;
        newerCloudSettings = remoteSettings.last_updated > local.last_updated;
        newerLocalSettings = remoteSettings.last_updated < local.last_updated;
    }

    function modifiedLocalSettings() {
        const local: Settings = get(localSettings);
        local.last_updated = Math.round(Date.now() / 1000);
        newerLocalSettings = true;
        localSettings.set(local);
        console.log("MODIFIED", newerLocalSettings, local.last_updated);
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
            <Input placeholder="Enter a value" value={cloudUrl} disabled/>
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
            <Input placeholder="Enter a value" bind:value={$localSettings.name} on:change={modifiedLocalSettings}/>
        </FormGroup>
        <FormGroup floating label="Company Name">
            <Input placeholder="Enter a value" bind:value={$localSettings.company} on:change={modifiedLocalSettings}/>
        </FormGroup>
        <FormGroup floating label="Client Name">
            <Input placeholder="Enter a value" bind:value={$localSettings.client} on:change={modifiedLocalSettings}/>
        </FormGroup>
    </div>
    <div class="border p-3">
        <h4>Timesheets</h4>
        <FormGroup floating label="ICS Outlook Calendar Link">
            <Input placeholder="https://URL" bind:value={$localSettings.ics_url}
                   on:change={modifiedLocalSettings}/>
        </FormGroup>
        <FormGroup floating label="Gitlab URL">
            <Input placeholder="https://URL" bind:value={$localSettings.gitlab_url}
                   on:change={modifiedLocalSettings}/>
        </FormGroup>
        <FormGroup floating label="Gitlab API Token">
            <Input placeholder="https://URL" bind:value={$localSettings.gitlab_access_token}
                   on:change={modifiedLocalSettings}/>
        </FormGroup>
        <div class="form-group mb-3">
            <div>ICS Filter</div>
            <MultiSelect allowUserOptions="append" bind:selected={$localSettings.ics_filter}
                         bind:options={$localSettings.ics_filter} on:change={modifiedLocalSettings}/>
        </div>
        <div class="form-group mb-3">
            <div>Projects</div>
            <MultiSelect allowUserOptions="append" bind:selected={$localSettings.projects}
                         bind:options={$localSettings.projects} on:change={modifiedLocalSettings}/>
        </div>
        <div class="form-group mb-3">
            <div>Tags</div>
            <MultiSelect allowUserOptions="append" bind:selected={$localSettings.tags}
                         bind:options={$localSettings.tags} on:change={modifiedLocalSettings}/>
        </div>
    </div>
    <div class="p-3">
        {#if $cloudSettings.cloud_url && $cloudSettings.cloud_api_key}
            <button type="button" class="btn" class:btn-secondary={!newerLocalSettings}
                    class:btn-primary={newerLocalSettings}
                    on:click={() => syncSettingsToCloud(cloudSettingsChanged)}>
                Sync Settings to Cloud
            </button>
        {/if}
    </div>
</Form>

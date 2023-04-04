<script lang="ts">
    import '../app.scss';
    import {
        Collapse,
        NavbarToggler,
        Nav,
        Dropdown,
        DropdownToggle,
        DropdownMenu,
        DropdownItem
    } from 'sveltestrap';
    import {shortcut} from "../assets/js/shortcut"
    import {page} from '$app/stores';
    import {onMount} from "svelte";
    import {selectedDate} from "../assets/js/data";

    let isOpen = false;

    function handleUpdate(event) {
        isOpen = event.detail.isOpen;
    }

    function setCurrentDate(unixTime: number) {
        console.log("SET", unixTime);
        selectedDate.set({date: unixTime});
    }

    let currentPath = "";
    onMount(() => {
        return page.subscribe(val => {
            currentPath = val.url.pathname;
        });
    })

</script>

<header class="d-print-none container container-lg pt-3">
    <nav class="navbar navbar-expand-md">
        <ul class="nav nav-tabs me-auto">
            <li class="nav-item">
                <a use:shortcut={{alt: true, code: 'KeyE'}} class="nav-link" class:active={$page.url.pathname === "/"}
                   aria-current="page" href="/">Edit (E)</a>
            </li>
            <li class="nav-item">
                <a use:shortcut={{alt: true, code: 'KeyP'}} class="nav-link"
                   class:active={$page.url.pathname === "/print"} aria-current="page" href="/print">Print (P)</a>
            </li>
            <li class="nav-item">
                <a use:shortcut={{alt: true, code: 'KeyC'}} class="nav-link"
                   class:active={$page.url.pathname === "/settings"} aria-current="page" href="/settings">Configuration
                    (C)</a>
            </li>
        </ul>
        <NavbarToggler on:click={() => (isOpen = !isOpen)}/>
        <Collapse {isOpen} navbar expand="md" on:update={handleUpdate}>
            <Nav navbar>
                <input type="date" class="form-control"
                       value={new Date($selectedDate.date).toISOString().split('T')[0]}
                       on:input={e => setCurrentDate(e.target.valueAsNumber) }/>
                {#if $page.url.pathname === "/"}
                    <Dropdown nav inNavbar>
                        <DropdownToggle nav caret>Export</DropdownToggle>
                        <DropdownMenu right>
                            <DropdownItem>Export .sheet</DropdownItem>
                        </DropdownMenu>
                    </Dropdown>
                    <Dropdown nav inNavbar>
                        <DropdownToggle nav caret>Import</DropdownToggle>
                        <DropdownMenu right>
                            <DropdownItem>Import .sheet</DropdownItem>
                            <DropdownItem divider/>
                            <DropdownItem header>This month</DropdownItem>
                            <DropdownItem>Add entries from ICS</DropdownItem>
                            <DropdownItem>Clear by filter</DropdownItem>
                        </DropdownMenu>
                    </Dropdown>
                {/if}
            </Nav>
        </Collapse>
    </nav>
</header>
<main>
    <slot/>
</main>
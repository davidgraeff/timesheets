<script lang="ts">
    import {Button, Modal, ModalBody, ModalFooter, ModalHeader} from "sveltestrap";

    export let title = "Please confirm";
    export let desc = "";
    let show = false;
    let callback;

    export function open(callback_) {
        callback = callback_;
        show = true;
    }

    function confirm() {
        if (callback)
            callback();
        show = false;
    }
</script>

<Modal isOpen={show} backdrop="static">
    <ModalHeader>{title}</ModalHeader>
    <ModalBody>
        {desc}
    </ModalBody>
    <ModalFooter>
        <Button color="danger" on:click={confirm}>Confirm</Button>
        <Button color="secondary" on:click={() => show = false}>Cancel</Button>
    </ModalFooter>
</Modal>

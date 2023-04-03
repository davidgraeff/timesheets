<script lang="ts">

    import {selectedDate} from "../../assets/js/data.js";
    import {localSettings} from "../../assets/js/settings.js";

    let header_date = "2022-12";
</script>

<h1>Timesheet {header_date} / {$localSettings.name}</h1>

<div class="border border-1 p-3 mt-2 d-flex justify-content-evenly">
    <div class="">Client: {$localSettings.client}</div>
    <div class="">Worker: {$localSettings.name}</div>
    <div class="">Agency: {$localSettings.company}</div>
</div>
<div class="p-3 mt-2 d-flex">
    <div style="flex: min-content">
        <table class="table" style="width: 250px">
            <thead>
            <tr>
                <th style="width: 120px">Total hours</th>
                <th style="text-align: right">123:00 hh:mm</th>
            </tr>
            </thead>
            <tbody>
            <tr>
                <td>Project 1</td>
                <td style="text-align: right">34:00 hh:mm</td>
            </tr>
            <tr>
                <td>Project 1</td>
                <td style="text-align: right">3:00 hh:mm</td>
            </tr>
            <tr>
                <td>Project 1</td>
                <td style="text-align: right">64:00 hh:mm</td>
            </tr>
            </tbody>
        </table>
    </div>
    <div style="width: 250px">diagram</div>
    <div class="border border-1 p-2 mt-2"
         style="min-height: 120px; flex: 2; display: flex; flex-direction: column; justify-content: end;align-items: end;">
        <div class="border border-1 w-100"></div>
        <div class="p2">Signature</div>
    </div>
</div>

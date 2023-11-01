const { exec } = require('child_process');
const { cubeFromHex, sleep } = require('./utility');
const { createClient } = require('redis');
const path = require('path');

let ENDPOINT_LOCAL = "http://localhost:8082";

async function register_building(id, readable_id, is_tower, message, location) {
    var myHeaders = new Headers();
    myHeaders.append("Content-Type", "application/json");

    var raw = JSON.stringify({
        "id": id,
        "is_tower": is_tower,
        "task_message": message,
        "readable_id": readable_id,
        location,
    });

    var requestOptions = {
    method: 'POST',
    headers: myHeaders,
    body: raw,
    redirect: 'follow'
    };

    let endpoint = ENDPOINT_LOCAL; 
    return fetch(`${endpoint}/building`, requestOptions)
        .then(response => response.text())
        .then(result => console.log(result))
        .catch(error => console.log('error', error))

}

const HEX_DUMP_MESSAGE = "Pick up a big pile of garbage data at the Hex Dump"
const MEME_GEN_MESSAGE = "Prepare your steamed hams at the Meme Generator"
const SELFIE_POINT_MESSAGE = "Find your best angle at Selfie Point"

const building_coords = {
    "TOWER": cubeFromHex("0x01", "0x0", "0xffff"),
    "HEX_DUMP": cubeFromHex("0xfff9", "0x0a", "0xfffd"),
    "MEME_GEN": cubeFromHex("0x05", "0x02", "0xfff9"),
    "SELFIE_POINT": cubeFromHex("0xfffb", "0xfffe", "0x07")
};

async function register_all() {
    await register_building("0x34cf8a7e000000000000000000000000000000010000ffff", "TOWER", true, "", [
        '0x0', '0x01', '0x0', '0xffff'
    ])
    await register_building("0x34cf8a7e0000000000000000000000000000fff9000afffd", "HEX_DUMP", false, HEX_DUMP_MESSAGE, [
        '0x0', '0xfff9', '0x0a', '0xfffd'
    ])
    await register_building("0x34cf8a7e000000000000000000000000000000050002fff9", "MEME_GEN", false, MEME_GEN_MESSAGE, [
        '0x0', '0x05', '0x02', '0xfff9'
    ])
    await register_building("0x34cf8a7e0000000000000000000000000000fffbfffe0007", "SELFIE_POINT", false, SELFIE_POINT_MESSAGE, [
        '0x0', '0xfffb', '0xfffe', '0x07'
    ])
}

async function setup_world() {
    const client = await createClient()
    .on('error', err => console.log('Redis Client Error', err))
    .connect();
    await client.sendCommand(['FLUSHALL']);
    await client.disconnect();
    // Change the working directory to the specified path

    const directoryPath = path.resolve(process.cwd(), '../packages/tonk-state-service');
    const options = { cwd: directoryPath };


    // // Run the 'cargo test' command
    // exec('cargo test', options, (error, stdout, stderr) => {
    //     if (error) {
    //     console.error(`Error: ${error.message}`);
    //     return;
    //     }
    //     if (stderr) {
    //     console.error(`Stderr: ${stderr}`);
    //     return;
    //     }
    //     console.log(`Stdout: ${stdout}`);
    // });

    // this gives some time for the game state to re-initialize the game
    await sleep(5000);

    await register_all();
}

async function teardown_world() {
    const client = await createClient()
    .on('error', err => console.log('Redis Client Error', err))
    .connect();
    await client.sendCommand(['FLUSHALL']);
    await client.disconnect();

}

module.exports = {
    setup_world,
    teardown_world,
    building_coords
};
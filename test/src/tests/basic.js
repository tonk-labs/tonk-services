const { getGame, requestStart, requestJoin, sendVote, getPlayer, isInGame, getPlayers, registerPlayer, getTask, postTask, postAction, getLastRoundResult } = require('../api');
const { i32ToHexTwosComplement, hexTwosComplementToI32, Cube, getRandomCoordinateAtDistance, cubeFromHex, sleep } = require('../utility');
const { building_coords } = require('../setup');
const { createClient } = require('redis');

let game = {};
let nodes = [{
        id: "node0",
        player: {
            id: "0x0",
            addr: "0x0",
        },
        location: {
            id: "",
            tile: {
                id: "",
                coords: ["0x0", i32ToHexTwosComplement(0), i32ToHexTwosComplement(0), i32ToHexTwosComplement(0)]
            }
        }
    }, {
        id: "node1",
        player: {
            id: "0x1",
            addr: "0x1"
        },
        location: {
            id: "",
            tile: {
                id: "",
                coords: ["0x0", i32ToHexTwosComplement(0), i32ToHexTwosComplement(0), i32ToHexTwosComplement(0)]
            }
        }
    }, {
        id: "node2",
        player: {
            id: "0x2",
            addr: "0x2"
        },
        location: {
            id: "",
            tile: {
                id: "",
                coords: ["0x0", i32ToHexTwosComplement(0), i32ToHexTwosComplement(0), i32ToHexTwosComplement(0)]
            }
        }
    }, {
        id: "node3",
        player: {
            id: "0x3",
            addr: "0x3",
        },
        location: {
            id: "",
            tile: {
                id: "",
                coords: ["0x0", i32ToHexTwosComplement(0), i32ToHexTwosComplement(0), i32ToHexTwosComplement(0)]
            }
        }
    }, {
        id: "node4",
        player: {
            id: "0x4",
            addr: "0x4",
        },
        location: {
            id: "",
            tile: {
                id: "",
                coords: ["0x0", i32ToHexTwosComplement(0), i32ToHexTwosComplement(0), i32ToHexTwosComplement(0)]
            }
        }
    },
    {
        id: "node5",
        player: {
            id: "0x5",
            addr: "0x5",
        },
        location: {
            id: "",
            tile: {
                id: "",
                coords: ["0x0", i32ToHexTwosComplement(0), i32ToHexTwosComplement(0), i32ToHexTwosComplement(0)]
            }
        }
    },
    {
        id: "node6",
        player: {
            id: "0x6",
            addr: "0x6",
        },
        location: {
            id: "",
            tile: {
                id: "",
                coords: ["0x0", i32ToHexTwosComplement(0), i32ToHexTwosComplement(0), i32ToHexTwosComplement(0)]
            }
        }
    },
    {
        id: "node7",
        player: {
            id: "0x7",
            addr: "0x7",
        },
        location: {
            id: "",
            tile: {
                id: "",
                coords: ["0x0", i32ToHexTwosComplement(0), i32ToHexTwosComplement(0), i32ToHexTwosComplement(0)]
            }
        }
    },
    {
        id: "node8",
        player: {
            id: "0x8",
            addr: "0x8",
        },
        location: {
            id: "",
            tile: {
                id: "",
                coords: ["0x0", i32ToHexTwosComplement(0), i32ToHexTwosComplement(0), i32ToHexTwosComplement(0)]
            }
        }
    },
];

async function movePlayer(id, coords) {
    const index = nodes.findIndex(n => n.id == id);
    const ccoords = ["0x0", i32ToHexTwosComplement(coords.q), i32ToHexTwosComplement(coords.r), i32ToHexTwosComplement(coords.s)]
    nodes[index].location.tile.coords = ccoords;
    const client = await createClient()
        .on('error', err => console.log('Redis Client Error', err))
        .connect();
    await client.set(`locations:${nodes[index].id}`, JSON.stringify(nodes[index]));
    await client.disconnect();
}

async function setup() {
    const client = await createClient()
        .on('error', err => console.log('Redis Client Error', err))
        .connect();
    await client.set(`locations:node0`, JSON.stringify(nodes[0]));
    await client.set(`locations:node1`, JSON.stringify(nodes[1]));
    await client.set(`locations:node2`, JSON.stringify(nodes[2]));
    await client.set(`locations:node3`, JSON.stringify(nodes[3]));
    await client.set(`locations:node4`, JSON.stringify(nodes[4]));
    await client.set(`locations:node5`, JSON.stringify(nodes[5]));
    await client.set(`locations:node6`, JSON.stringify(nodes[6]));
    await client.set(`locations:node7`, JSON.stringify(nodes[7]));
    await client.set(`locations:node8`, JSON.stringify(nodes[8]));
    await client.disconnect();
}

async function moveToTaskLocation(player, task, secondDestination) {
    const loc = secondDestination ? task.second_destination.location : task.destination.location;
    const p_loc = getRandomCoordinateAtDistance(cubeFromHex(loc[1], loc[2], loc[3]), 1);
    await movePlayer(player.mobile_unit_id, p_loc);
}

async function moveToTower(player) {
    const tower_loc = getRandomCoordinateAtDistance(building_coords["TOWER"]);
    await movePlayer(player.mobile_unit_id, tower_loc);
}

async function moveToPlayerLocation(player, targetPlayer) {
    let node = nodes.find(n => n.id == targetPlayer.mobile_unit_id);
    let loc = node.location.tile.coords
    const p_loc = getRandomCoordinateAtDistance(cubeFromHex(loc[1], loc[2], loc[3]), 1);
    await movePlayer(player.mobile_unit_id, p_loc);
}

async function clockToZero(game) {
    const client = await createClient()
        .on('error', err => console.log('Redis Client Error', err))
        .connect();
    await client.set("clock", JSON.stringify({
        status: game.status,
        time: {
            round: game.time.round,
            timer: 0,
        }
    }))
    await client.disconnect();
}

async function run() {
    let game = await getGame();
    await registerPlayer("0x0", "node0", "TheHackz0r");
    await registerPlayer("0x1", "node1", "GoblinOats");
    await registerPlayer("0x2", "node2", "Baz");
    await registerPlayer("0x3", "node3", "SecretCat");
    await registerPlayer("0x4", "node4", "BouncyCastle");
    await registerPlayer("0x5", "node5", "MegaLord");
    await registerPlayer("0x6", "node6", "MegaLord2");
    await registerPlayer("0x7", "node7", "MegaLord3");
    await registerPlayer("0x8", "node8", "MegaLord4");
    await requestJoin(game.id, "0x0");
    await requestJoin(game.id, "0x1");
    await requestJoin(game.id, "0x2");
    await requestJoin(game.id, "0x3");
    await requestJoin(game.id, "0x4");
    await requestJoin(game.id, "0x5");
    await requestJoin(game.id, "0x6");
    await requestJoin(game.id, "0x7");
    await requestJoin(game.id, "0x8");
    await requestStart();

    const player = await getPlayer("0x0");
    const player1 = await getPlayer("0x1");
    const player2 = await getPlayer("0x2");
    const player3 = await getPlayer("0x3");
    const player4 = await getPlayer("0x4");
    const player5 = await getPlayer("0x5");
    const player6 = await getPlayer("0x6");
    const player7 = await getPlayer("0x7");
    const player8 = await getPlayer("0x8");
    const players = [player, player1, player2, player3, player4, player5, player6, player7, player8];

    let bugs = players.filter(p => p.role == "Bugged");
    let civilians = players.filter(p => p.role == "Normal");
    console.log(`civilians: ${civilians.map(c => c.id).join(",")}`);
    console.log(`bugs: ${bugs.map(b => b.id).join(",")}`);

    let tasks = [];
    let num_bugs = bugs.length;

    // TASK ROUND
    for (let i = 0; i < civilians.length; i++) {
        let civilian = civilians[i];
        let task = await getTask({ id: civilian.id });
        tasks.push(task);
        await moveToTaskLocation(civilian, task, false);
        await sleep(2000);
        await postTask(task, civilian);

        await moveToTaskLocation(civilian, task, true);
    }
    await sleep(2000);

    for (let i = 0; i < bugs.length; i++) {
        let bug = bugs[i];
        await moveToPlayerLocation(bug, civilians[i]);
    }
    await sleep(2000);

    for (let i = 0; i < bugs.length; i++) {
        let bug = bugs[i];
        let response = await postAction({
            id: civilians[i].id,
            display_name: civilians[i].display_name
        }, game, bug, false)

        await sleep(2000);
        let updated_bug = await getPlayer(bug.id);
        if (updated_bug.used_action !== "ReturnToTower") {
            console.log(`Used action was not set correctly for ${updated_bug.id}, it was set to: ${updated_bug.used_action}`)
            process.exit(-1);
        }
        await moveToTower(bug);
    }

    await sleep(2000);
    for (let i = 0; i < bugs.length; i++) {
        let bug = bugs[i];
        let response = await postAction({
            id: civilians[i].id,
            display_name: civilians[i].display_name
        }, game, bug, true);
    }

    for (let i = 0; i < civilians.length; i++) {
        let civilian = civilians[i];
        let task = await getTask({ id: civilian.id });
        await postTask(task, civilian);
        await moveToTower(civilian);
    }

    await sleep(2000);
    for (let i = 0; i < civilians.length; i++) {
        let civilian = civilians[i];
        let task = await getTask({ id: civilian.id });
        await postTask(task, civilian);
    }

    // await clockToZero(game);
    await sleep(10000);

    game = await getGame();
    console.log(JSON.stringify(game, null, 2));
    if (game.status == "Lobby" || game.status == "Task") {
        console.log("State didn't transition :/");
        return;
    }

    const lastRound = await getLastRoundResult(game);
    civilians = civilians.filter(c => typeof lastRound.eliminated.find(p => p.player.id == c.id) == 'undefined');
    if (lastRound.eliminated.findIndex(p => p.player.role == null) >= 0) {
        console.log(JSON.stringify(lastRound))
        console.log("Quitting because role should not be null in the eliminated list");
        return;
    }

    // VOTE STAGE
    // At this stage, one or two of the civilians are killed which leaves 1,7 or 2,6 remaining
    for (let i = 0; i < bugs.length; i++) {
        let bug = bugs[i];
        await sendVote(civilians[0].id, bug);
    }

    for (let i = 0; i < civilians.length; i++) {
        let civilian = civilians[i];
        if ( i == 0 ) {
            // await sendVote(civilians[1]);
        } else {
            await sendVote(civilians[0].id, civilian);
        }
    }

    game = await getGame();
    await clockToZero(game);

    // let task_1 = await getTask({ id: civilians[0].id });
    // let task_2 = await getTask({ id: civilians[1].id });
    // let task_3 = await getTask({ id: civilians[2].id });
    // const bug_task = await getTask({ id: bugs[0].id });

    // await moveToTaskLocation(civilians[0], task_1, false);
    // await moveToTaskLocation(civilians[1], task_2, false);
    // await moveToTaskLocation(civilians[2], task_3, false);

    // await sleep(2000);

    // await postTask(task_1, civilians[0]);
    // await postTask(task_2, civilians[1]);
    // await postTask(task_3, civilians[2]);

    // await moveToTaskLocation(civilians[0], task_1, true);
    // await moveToTaskLocation(civilians[1], task_2, true);
    // await moveToTaskLocation(civilians[2], task_3, true);

    // await sleep(2000);

    // await postTask(task_1, civilians[0]);
    // await postTask(task_2, civilians[1]);
    // await postTask(task_3, civilians[2]);

    // await moveToPlayerLocation(bugs[0], civilians[0]);
    // await sleep(2000);

    // await postAction({
    //     id: civilians[0].id,
    //     display_name: civilians[0].display_name
    // }, game, bugs[0], false)

    // await moveToTower(civilians[0]);
    // await moveToTower(civilians[1]);
    // await moveToTower(civilians[2]);
    // await moveToTower(bugs[0]);

    // await sleep(3000);

    // await postTask(task_1, civilians[0]);
    // await postTask(task_2, civilians[1]);
    // await postTask(task_3, civilians[2]);
    // await postAction({
    //     id: civilians[0].id,
    //     display_name: civilians[0].display_name
    // }, game, bugs[0], true)

}

module.exports = {
    setup,
    run,
}
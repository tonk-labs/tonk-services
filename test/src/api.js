const ENDPOINT = "http://localhost:8082"

async function getGame() {
    try {
        let response = await fetch(`${ENDPOINT}/game`);
        let raw = await response.text();
        return JSON.parse(raw);
    } catch (e) {
        console.log(e);
        return (`{ "status": "GameServerDown" }`)
    }
}

async function requestStart() {
    // var myHeaders = new Headers();
    // myHeaders.append("Content-Type", "application/json");

    var requestOptions = {
        method: 'POST',
    };

    try {
        let response = await fetch(`${ENDPOINT}/game`, requestOptions)
        let text = await response.text();
        console.log(text);
    } catch (e) {
        console.log(e);
    }
}


async function requestJoin(gameId, playerId, secretKey) {
    // var myHeaders = new Headers();
    // myHeaders.append("Content-Type", "application/json");

    var raw = JSON.stringify({
        "id": playerId,
    });

    var requestOptions = {
        method: 'POST',
        headers: {
            "Content-Type": "application/json",
            "Sec-Fetch-Mode": "cors"
        },
        mode: "cors",
        body: raw,
    };

    try {
        let response = await fetch(`${ENDPOINT}/game/${gameId}/player`, requestOptions)
        let text = await response.text();
        console.log(text);
    } catch (e) {
        console.log(e);
    }
}

async function sendVote(candidateId, player) {
    var raw = JSON.stringify({
        candidate: {
            id: candidateId
        }
    });

    var requestOptions = {
        method: 'POST',
        headers: {
            "Content-Type": "application/json",
            "Sec-Fetch-Mode": "cors"
        },
        mode: "cors",
        body: raw,
    };

    try {
        let response = await fetch(`${ENDPOINT}/vote?player_id=${player.id}&secret_key=fff`, requestOptions)
        let text = await response.text();
        console.log(text);
    } catch (e) {
        console.log(e);
    }
}

async function getPlayer(id) {
    try {
        let response = await fetch(`${ENDPOINT}/player/${id}`)
        let raw = await response.text();
        return JSON.parse(raw);
    } catch (e) {
        console.log(e);
    }
}

function isInGame(players, playerId) {
    return players.findIndex((p) => p.id == playerId) !== -1;
}

async function getPlayers(gameId, playerId) {
    try {
        let response = await fetch(`${ENDPOINT}/game/${gameId}/player?player_id=${playerId}`);
        let raw = await response.text();
        return JSON.parse(raw);
    } catch (e) {
        console.error(e);
        return [];
    }
}


async function getLastRoundResult(game) {
    //TODO implement 
    console.log("tonk getLastRound", game.time.round-1)
    try {
        let lastRound = game.time.round - 1;
        let response = await fetch(`${ENDPOINT}/game/result/${lastRound}`)
        let text = await response.text();
        return JSON.parse(text);
    } catch (e) {
        console.log(e);
    }
}

async function registerPlayer(id, mobileUnitId, displayName, hash, secret) {
    var raw = JSON.stringify({
        id: id, 
        mobile_unit_id: mobileUnitId,
        display_name: displayName 
    })
    var requestOptions = {
        method: 'POST',
        headers: {
            "Content-Type": "application/json",
        },
        body: raw
      };
      
      try {
        // let response = await fetch(`${ENDPOINT}/player/${id}?secret_key=${secret}&onchain_hash=${hash}`, requestOptions)
        let response = await fetch(`${ENDPOINT}/player/${id}`, requestOptions)
        // let text = await response.text();
      } catch (e) {
        console.log(e);
      }
}

async function getTask(player) {
    try {
        let response = await fetch(`${ENDPOINT}/task?player_id=${player.id}&secret_key=fff`);
        let text = await response.text();
        return JSON.parse(text);
    } catch (e) {
        console.log(e);
    }
}

async function postTask(task, player) {
    var raw = JSON.stringify(task);
    var requestOptions = {
        method: 'POST',
        headers: {
            "Content-Type": "application/json",
        },
        body: raw
      };
      
      try {
        let response = await fetch(`${ENDPOINT}/task?player_id=${player.id}&secret_key=fff`, requestOptions)
      } catch (e) {
        console.log(e);
      }
}

async function postAction(target, game, player, confirmed) {
    var raw = JSON.stringify({
        poison_target: target,
        round: game.time.round,
        confirmed,
        interrupted_task: false 
    })
    var requestOptions = {
        method: 'POST',
        headers: {
            "Content-Type": "application/json",
        },
        body: raw
      };
      
      try {
        let response = await fetch(`${ENDPOINT}/action?player_id=${player.id}&secret_key=fff`, requestOptions)
      } catch (e) {
        console.log(e);
      }
}

module.exports = {
    getGame,
    requestStart,
    requestJoin,
    sendVote,
    getPlayer,
    isInGame,
    getPlayers,
    getLastRoundResult,
    registerPlayer,
    getTask,
    postTask,
    postAction
};
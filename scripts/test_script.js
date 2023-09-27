
async function getGame() {
    var requestOptions = {
    method: 'GET',
    redirect: 'follow'
    };

    try {
        let response = await fetch("http://0.0.0.0:8082/game", requestOptions);
        return await response.text();
    } catch (e) {
        console.log(error);
    }
}

async function addBuilding(isTower, id) {
    var myHeaders = new Headers();
    myHeaders.append("Content-Type", "application/json");

    var raw = JSON.stringify({
    "id": id,
    "is_tower": isTower 
    });

    var requestOptions = {
    method: 'POST',
    headers: myHeaders,
    body: raw,
    redirect: 'follow'
    };

    try {
        let response = await fetch("http://0.0.0.0:8082/building", requestOptions);
        let text = await response.text();
        console.log(text);
    } catch (e) {
        console.log(e);
    }
}

async function registerPlayer(id, hash, secret) {
    var requestOptions = {
        method: 'POST',
        redirect: 'follow'
      };
      
      try {

        // let response = await fetch(`http://0.0.0.0:8082/player/${id}?secret_key=${secret}&onchain_hash=${hash}`, requestOptions)
        let response = await fetch(`http://0.0.0.0:8082/player/${id}`, requestOptions)
        let text = await response.text();
      } catch (e) {
        console.log(e);
      }
}

async function joinGame(game_id, id, key) {
    var myHeaders = new Headers();
    myHeaders.append("Content-Type", "application/json");

    var raw = JSON.stringify({
    "id": id,
    "secret_key": key
    });

    var requestOptions = {
    method: 'POST',
    headers: myHeaders,
    body: raw,
    redirect: 'follow'
    };

    try {
        let response = await fetch(`http://0.0.0.0:8082/game/${game_id}/player`, requestOptions)
        let text = await response.text();
        console.log(text);
    } catch (e) {
        console.log(e);
    }
}

async function startGame() {
    var requestOptions = {
        method: 'POST',
        redirect: 'follow'
      };
      
      try {
        let response = await fetch("http://0.0.0.0:8082/game", requestOptions)
        let text = await response.text();
        console.log(text);
      } catch (e) {
        console.log(e);
      }
}



var key = "123456678901234567890";
var id = "2846bc30d997410a97365419ce5145e7"
var hash = "1369c4f2a54f859b73b198d1e24ac69088540444968df1480530a807211e0861";

var id_2 = "af0j9w83j0a983jf0a9jw3f"
var key_2 = "09876543210987654321";
var hash_2 = "aee0cead9c5ff8864ed64b39c32d081111f45c3b8e4a4ef40cf944490081c589";

var id_3 = "af0j9w8fja83jajfhf832h";
var key_3 = "1337133713371337";
var hash_3 = "52cb23e48906c3bf9f72f960295bb0c72a466c631ce813541753baeb853bee7b";

async function test() {
    var gameJSON = await getGame();
    var game = JSON.parse(gameJSON);

    await addBuilding(false, "20389jr09q8j23f")
    await addBuilding(false, "09fj3049ujw4f")
    await addBuilding(false, "09awifj09asidjf0aw")
    await addBuilding(false, "djif09aisdjf09asijdf")
    await addBuilding(true, "a0s9idfj09asijdf")

    await registerPlayer(id, hash, key)
    await registerPlayer(id_2, hash_2, key_2)
    await registerPlayer(id_3, hash_3, key_3)

    await joinGame(game.id, id, key)
    await joinGame(game.id, id_2, key_2)
    await joinGame(game.id, id_3, key_3)

    await startGame()
}

test();
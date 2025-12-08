let ws;
let currentRoom = "";
let currentName = "";

function log(msg) {
    const logElement = document.getElementById("log");
    logElement.textContent += msg + "\n";
    logElement.scrollTop = logElement.scrollHeight;
}

function connectWS() {
    const server = document.getElementById("server").value;
    const room = document.getElementById("room").value;
    const name = document.getElementById("name").value;
    
    if (!server || !room || !name) {
        log("Erreur: Tous les champs doivent être remplis");
        return;
    }

    try {
        ws = new WebSocket(server);
    } catch (error) {
        log("Erreur de connexion: " + error);
        return;
    }

    currentRoom = room;
    currentName = name;

    ws.onopen = () => {
        log("Connecté au serveur !");
        ws.send(JSON.stringify({
            type: "Join",
            room: room,
            name: name
        }));
        document.getElementById("connectBtn").textContent = "Déconnexion";
        document.getElementById("connectBtn").onclick = disconnectWS;
        document.getElementById("chatInput").disabled = false;
        document.getElementById("sendChatBtn").disabled = false;
    };

    ws.onmessage = (ev) => {
        try {
            const data = JSON.parse(ev.data);
            handleServerMessage(data);
        } catch (error) {
            log("Message reçu: " + ev.data);
        }
    };

    ws.onerror = (err) => {
        log("Erreur WebSocket: " + err);
    };

    ws.onclose = () => {
        log("Déconnecté du serveur");
        document.getElementById("connectBtn").textContent = "Connexion";
        document.getElementById("connectBtn").onclick = connectWS;
        document.getElementById("chatInput").disabled = true;
        document.getElementById("sendChatBtn").disabled = true;
    };
}

function disconnectWS() {
    if (ws) {
        ws.close();
    }
}

function handleServerMessage(msg) {
    switch (msg.type) {
        case "Joined":
            log(`→ ${msg.name} a rejoint la partie`);
            break;
        case "Moved":
            log(`→ Coup joué: ${msg.from} → ${msg.to}`);
            break;
        case "Chatted":
            log(`[${msg.name}] ${msg.message}`);
            break;
        case "Error":
            log(`❌ Erreur: ${msg.message}`);
            break;
        default:
            log(`Message inconnu: ${JSON.stringify(msg)}`);
    }
}

function sendMove(from, to, promotion = null) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
            type: "Move",
            room: currentRoom,
            from: from,
            to: to,
            promotion: promotion
        }));
    } else {
        log("Erreur: Non connecté au serveur");
    }
}

function sendChat() {
    const chatInput = document.getElementById("chatInput");
    const message = chatInput.value.trim();
    
    if (message && ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
            type: "Chat",
            room: currentRoom,
            message: message
        }));
        chatInput.value = "";
    }
}

// Exemple de coups pour tester
function testMove1() {
    sendMove("e2", "e4");
}

function testMove2() {
    sendMove("e7", "e5");
}

// Gestion de la touche Entrée pour le chat
document.getElementById("chatInput").addEventListener("keypress", function(e) {
    if (e.key === "Enter") {
        sendChat();
    }
});

const testIP = "ws://100.65.161.131:5000/";
const socket = new WebSocket(testIP);

 socket.addEventListener('open', function (event) {
    socket.send(JSON.stringify({
        Move: {
            x: 10,
            y: 10,
        }
    }));    
 })

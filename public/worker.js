import * as rustWasm from "./pkg/modelwasm.js"

async function start() {
    console.log("started");
    await rustWasm.default()
    await rustWasm.initThreadPool(navigator.hardwareConcurrency);
}

onmessage = async function (e) {
    // receive the file from the main thread 
    console.log(file);
    var file = e.data.file;
    let buffer = await e.data.file.arrayBuffer() 
    let arr = new Uint8Array(buffer)

    if(e.data.func == "analyze_file"){
        let out = rustWasm.analyze_file(arr)
        this.postMessage(out)
    }
    if(e.data.func == "process_file"){
        let out = rustWasm.process_file(arr, e.data.data)
        this.postMessage(out)
    }
}

await start()
console.log("started worker") 
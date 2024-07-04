const worker = new Worker("./worker.js", { type: "module" });

worker.addEventListener("message", async ({ data }) => {
    document.getElementById("output").innerHTML = data;
    var button = document.getElementById('upload');
    button.innerText = button.textContent = 'Upload or try again';
    setTimeout(() => {
        // Yeah this breaks when the response doesn't have a valform element sorry
        document.getElementById("valform").addEventListener("submit", runPrediction, false);
    }, 10);
});

worker.onerror = function (event) {
    console.log(event);
    throw new Error("s" + event.message + " (" + event.filename + ":" + event.lineno + ")");
}

// define a function to send a message to the worker
window.sendMessage = function (args) {
    console.log("Sending message to worker");
    worker.postMessage(args);
};

async function loadFile() {
    var files = document.getElementById('file').files;
    var file = files[0];
    // check if file was selected
    if (!file) {
        alert("Please select a file");
        return;
    }
    document.getElementById("output").innerHTML = '<div><div class="lds-ring"><div></div><div></div><div></div><div></div></div><span>Loading!</span></div>'
    // send the file to the worker
    sendMessage({"file":file, "data":null, "func":"analyze_file"})
} 

async function runPrediction(event) { 
    event.preventDefault();
    var formData = new FormData(event.target); 
    
    var objForm = {};
    for (var pair of formData.entries()) {
        var val = pair[1];
        if(pair[0] == "iterations"){
            var val = parseInt(pair[1])
        }
        if(pair[0] == "threshold"){
            var val = parseFloat(pair[1]);
        }
        if(pair[0] in objForm){
            objForm[pair[0]].push(val)
        } else {

            objForm[pair[0]] = [val]
        } 
    } 
    var files = document.getElementById('file').files;
    var file = files[0];

    // check if file was selected
    if (!file) {
        alert("Please select a file");
        return;
    }
 
    document.getElementById("output").innerHTML = '<div><div class="lds-ring"><div></div><div></div><div></div><div></div></div><span>Creating your prediction!</span></div>'
    sendMessage({"file":file, "data":JSON.stringify(objForm), "func":"process_file"})
}
 
document.getElementById("upload").addEventListener("click", loadFile, false);
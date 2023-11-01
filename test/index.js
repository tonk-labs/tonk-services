const { Worker, isMainThread, parentPort, workerData } = require('worker_threads');
const { setup_world, teardown_world } = require('./src/setup');
const basic = require('./src/tests/basic');

const NUM_THREADS = 8;

// function myFunction(data) {
//   // Replace the content of this function with your own function's logic.
//   console.log(`Worker: ${workerData.id} – Executing with data:`, data);
//   // Simulate some work with a delay.
//   return new Promise((resolve) => setTimeout(() => resolve(workerData.id), 1000));
// }

async function run() {
    await setup_world();
    await basic.setup();
    await basic.run();
    // await teardown_world();

    // if (isMainThread) {
    // // This code is executed in the main thread.
    // const numThreads = 8; // Configurable number of threads.
    // const functionData = { /* Data to pass to the function */ };

    // for (let i = 0; i < numThreads; i++) {
    //     const worker = new Worker(__filename, { workerData: { id: i, ...functionData } });
    //     worker.on('message', (result) => {
    //     console.log(`Main Thread: Received result from worker ${result}`);
    //     });
    //     worker.on('error', (err) => {
    //     console.error(err);
    //     });
    //     worker.on('exit', (code) => {
    //     if (code !== 0) {
    //         console.error(new Error(`Worker stopped with exit code ${code}`));
    //     }
    //     });
    // }
    // } else {
    //     // This code is executed in worker threads.
    //     myFunction(workerData).then((result) => {
    //         parentPort.postMessage(result);
    //     });
    // }
}

run();
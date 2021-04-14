// This worker.js file must be served along side of the index.html of for Bevy Retro games.

importScripts("$example.js");

$example("$example_bg.wasm").then(() => {
  const { start_worker_pool_worker } = $example;

  start_worker_pool_worker();
});

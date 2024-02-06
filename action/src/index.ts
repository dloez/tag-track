import * as core from "@actions/core";

async function run() {
  const input = core.getInput("test-input");

  if (!input) {
    core.setFailed("Input is required");
  }
  console.log(`Hello, ${input}!`);
  core.setOutput("myOutput", "Hello, world!");
}

run();

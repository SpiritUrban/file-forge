import { spawn } from 'child_process';
import { writeFileSync } from 'fs';

const config = {
  build: {
    devUrl: "http://localhost:1425"
  }
};
writeFileSync('.tauri.config.override.json', JSON.stringify(config));
console.log("Config written.");

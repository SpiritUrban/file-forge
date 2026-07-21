import { createServer } from 'node:net';
import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';

function getFreePort(port) {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.unref();
    server.on('error', (e) => {
      if (e.code === 'EADDRINUSE') {
        resolve(getFreePort(port + 1));
      } else {
        reject(e);
      }
    });
    server.listen(port, '127.0.0.1', () => {
      const actualPort = server.address().port;
      server.close(() => {
        resolve(actualPort);
      });
    });
  });
}

async function run() {
  const port = await getFreePort(1420);
  console.log(`[dev.mjs] Found free port: ${port}`);
  
  process.env.VITE_PORT = port.toString();
  
  const config = {
    build: {
      devUrl: `http://127.0.0.1:${port}`
    }
  };
  writeFileSync('.tauri.config.override.json', JSON.stringify(config));
  
  const tauri = spawn('npx', ['tauri', 'dev', '--config', '.tauri.config.override.json'], {
    stdio: 'inherit',
    shell: true
  });
  
  tauri.on('close', (code) => {
    process.exit(code);
  });
}

run();

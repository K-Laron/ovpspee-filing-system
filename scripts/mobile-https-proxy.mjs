import fs from 'node:fs';
import http from 'node:http';
import https from 'node:https';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');

const defaultCertDir = path.join(repoRoot, '.local', 'mobile-https');
const certPath = process.env.OVPSPEE_MOBILE_HTTPS_CERT ?? path.join(defaultCertDir, 'ovpspee-mobile.crt');
const keyPath = process.env.OVPSPEE_MOBILE_HTTPS_KEY ?? path.join(defaultCertDir, 'ovpspee-mobile.key');
const target = new URL(process.env.OVPSPEE_MOBILE_HTTP_TARGET ?? 'http://127.0.0.1:1421');
const host = process.env.OVPSPEE_MOBILE_HTTPS_HOST ?? '0.0.0.0';
const port = Number(process.env.OVPSPEE_MOBILE_HTTPS_PORT ?? '1443');

const loadTls = () => ({
  cert: fs.readFileSync(certPath),
  key: fs.readFileSync(keyPath)
});

const proxyHandler = (targetUrl) => (req, res) => {
  const upstreamUrl = new URL(req.url ?? '/', targetUrl);
  const headers = { ...req.headers, host: upstreamUrl.host };
  const upstream = http.request(
    upstreamUrl,
    {
      method: req.method,
      headers
    },
    (upstreamRes) => {
      res.writeHead(upstreamRes.statusCode ?? 502, upstreamRes.headers);
      upstreamRes.pipe(res);
    }
  );
  upstream.on('error', () => {
    if (!res.headersSent) {
      res.writeHead(502, { 'content-type': 'application/json' });
    }
    res.end(JSON.stringify({ error: 'Mobile HTTP hub is not reachable.' }));
  });
  req.pipe(upstream);
};

const startProxy = ({ listenHost = host, listenPort = port, targetUrl = target } = {}) =>
  new Promise((resolve, reject) => {
    const server = https.createServer(loadTls(), proxyHandler(targetUrl));
    server.once('error', reject);
    server.listen(listenPort, listenHost, () => resolve(server));
  });

const requestJson = (url) =>
  new Promise((resolve, reject) => {
    const req = https.request(
      url,
      { rejectUnauthorized: false },
      (res) => {
        let body = '';
        res.setEncoding('utf8');
        res.on('data', (chunk) => {
          body += chunk;
        });
        res.on('end', () => resolve({ statusCode: res.statusCode, body }));
      }
    );
    req.on('error', reject);
    req.end();
  });

const selfTest = async () => {
  const upstream = http.createServer((_req, res) => {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(JSON.stringify({ status: 'ok' }));
  });
  await new Promise((resolve) => upstream.listen(0, '127.0.0.1', resolve));
  const upstreamPort = upstream.address().port;
  const server = await startProxy({
    listenHost: '127.0.0.1',
    listenPort: 0,
    targetUrl: new URL(`http://127.0.0.1:${upstreamPort}`)
  });
  const proxyPort = server.address().port;
  const response = await requestJson(`https://127.0.0.1:${proxyPort}/api/mobile/health`);
  server.close();
  upstream.close();
  if (response.statusCode !== 200 || !response.body.includes('"status":"ok"')) {
    throw new Error(`HTTPS proxy self-test failed with HTTP ${response.statusCode}.`);
  }
  console.log('HTTPS proxy self-test passed.');
};

if (process.argv.includes('--self-test')) {
  await selfTest();
} else {
  const server = await startProxy();
  const address = server.address();
  console.log(`OVPSPEE mobile HTTPS proxy listening on https://${host}:${address.port}`);
  console.log(`Forwarding to ${target.href}`);
}

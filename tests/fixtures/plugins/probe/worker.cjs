const readline = require('node:readline');
let history = [];
readline.createInterface({input:process.stdin}).on('line', line => {
  const r = JSON.parse(line); history.push(r.op);
  if (r.method === 'stall') return;
  if (r.method === 'invalidReply') { process.stdout.write('[]\n'); return; }
  let response = {};
  if (r.op === 'mount' || r.op === 'update' || r.op === 'resize') response = {svg:`<svg xmlns="http://www.w3.org/2000/svg" width="${r.width}" height="${r.height}"><rect width="100%" height="100%" fill="red"/></svg>`};
  if (r.op === 'input') response = {events:[{name:'changed',payload:r.event.text}]};
  if (r.method === 'history') response = {result:history};
  if (r.method === 'badResult') response = {result:'not boolean'};
  process.stdout.write(JSON.stringify(response)+'\n');
});

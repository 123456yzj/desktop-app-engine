'use strict';
// Runs on the SSH host. stdout is one JSON document; command diagnostics go to stderr.
const fs = require('fs');
const path = require('path');
const cp = require('child_process');
const request = JSON.parse(process.argv[1]);
const root = fs.realpathSync(process.cwd());
const ROOT_ID = '@root';
function git(cwd, args) {
  return cp.execFileSync('git', args, {cwd, encoding:'utf8', maxBuffer:4*1024*1024, env:{...process.env,GIT_TERMINAL_PROMPT:'0',GIT_MERGE_AUTOEDIT:'no'}}).trim();
}
function repository(id) {
  if (id === ROOT_ID) {
    if (!fs.existsSync(path.join(root,'.git')) || fs.realpathSync(git(root,['rev-parse','--show-toplevel'])) !== root) throw Error('Turbo 根目录不是独立 Git 仓库');
    return root;
  }
  if (typeof id !== 'string' || !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(id) || id === '..') throw Error('非法业务目录');
  const cwd = fs.realpathSync(path.join(root, 'apps', id));
  if (!fs.existsSync(path.join(cwd,'.git')) || fs.realpathSync(git(cwd,['rev-parse','--show-toplevel'])) !== cwd) throw Error(id + ' 不是独立 Git 仓库');
  return cwd;
}
function snapshot(cwd) {
  let branch = '';
  try { branch = git(cwd,['symbolic-ref','--quiet','--short','HEAD']); } catch (_) {}
  return {branch, head:git(cwd,['rev-parse','HEAD'])};
}
function branches(cwd) {
  const local=git(cwd,['for-each-ref','--format=%(refname:lstrip=2)','refs/heads']).split('\n');
  const remote=git(cwd,['for-each-ref','--format=%(if)%(symref)%(then)%(else)%(refname:lstrip=3)%(end)','refs/remotes']).split('\n');
  return [...new Set([...local,...remote].filter(Boolean))].sort();
}
function scripts() {
  const pkg=JSON.parse(fs.readFileSync(path.join(root,'package.json'),'utf8'));
  return pkg.scripts || {};
}
function verify() {
  const available=scripts();
  for (const expected of [...(request.rootProject ? [request.rootProject] : []), ...request.projects]) {
    const cwd=repository(expected.id), actual=snapshot(cwd);
    if (!expected.branch || actual.branch!==expected.branch || actual.head!==expected.head) throw Error((expected.id===ROOT_ID?'Turbo 根仓库':expected.id)+' 分支或提交已变化，请重新扫描后构建');
    if (expected.script && typeof available[expected.script] !== 'string') throw Error(expected.id+' 构建脚本已不存在');
  }
  if (request.script && typeof available[request.script] !== 'string') throw Error('构建脚本不存在：'+request.script);
  return {ok:true};
}
function fetchBranches(cwd) {
  cp.execFileSync('git',['fetch','--all','--prune'],{cwd,stdio:['ignore',2,2],timeout:120000});
  const values=branches(cwd);
  if(values.length>10000) throw Error('分支超过 10000 个');
  return values;
}
function main() {
  if(request.operation==='scan') {
    process.stderr.write('[scan] 正在扫描工作区 apps/…\n');
    const available=scripts(), projects=[], branchLists={};
    function refresh(id,cwd) {
      if(!request.fetchBranches) return;
      const label=id===ROOT_ID?'Turbo 根仓库':id;
      process.stderr.write('[fetch] '+label+' · 刷新分支…\n');
      try {
        branchLists[id]={branches:fetchBranches(cwd)};
        process.stderr.write('[fetch] '+label+' · 完成\n');
      } catch(e) {
        branchLists[id]={error:e.message};
        process.stderr.write('[fetch] '+label+' · 失败: '+e.message+'\n');
      }
    }
    let rootProject=null;
    if(fs.existsSync(path.join(root,'.git'))) {
      const cwd=repository(ROOT_ID);
      refresh(ROOT_ID,cwd);
      rootProject={id:ROOT_ID,name:'Turbo 根仓库',...snapshot(cwd),script:null};
      process.stderr.write('[scan] Turbo 根仓库 · '+rootProject.branch+'\n');
    }
    for(const entry of fs.readdirSync(path.join(root,'apps'),{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name))) {
      if(!entry.isDirectory() && !entry.isSymbolicLink()) continue;
      const id=entry.name, guessed=path.join(root,'apps',id);
      if(!fs.existsSync(path.join(guessed,'.git')) || !fs.existsSync(path.join(guessed,'package.json'))) continue;
      try {
        const cwd=repository(id), pkg=JSON.parse(fs.readFileSync(path.join(cwd,'package.json'),'utf8'));
        const script='build:dev:'+id.replace(/^web-/,'');
        refresh(id,cwd);
        projects.push({id,name:typeof pkg.name==='string'?pkg.name:id,...snapshot(cwd),script:typeof available[script]==='string'?script:null});
        process.stderr.write('[scan] '+id+' · '+projects[projects.length-1].branch+'\n');
      } catch(e) { throw Error(id+': '+e.message); }
    }
    if(projects.length>256) throw Error('业务仓库超过 256 个');
    process.stderr.write('[scan] 发现 '+projects.length+' 个独立业务仓库\n');
    return {rootProject,projects,allScript:typeof available['build:dev']==='string',branchLists};
  }
  if(request.operation==='verify') return verify();
  const cwd=repository(request.id);
  if(request.operation==='branches') {
    const values=fetchBranches(cwd);
    return {...snapshot(cwd),branches:values};
  }
  if(request.operation==='switch') {
    if(typeof request.branch!=='string' || !branches(cwd).includes(request.branch)) throw Error('目标分支已不存在，请点击 Fetch 刷新分支列表');
    git(cwd,['check-ref-format','--branch',request.branch]);
    cp.execFileSync('git',['checkout',request.branch],{cwd,stdio:['ignore',2,2]});
  } else if(request.operation==='pull') {
    cp.execFileSync('git',['pull'],{cwd,stdio:['ignore',2,2]});
  } else throw Error('未知操作');
  return snapshot(cwd);
}
try { process.stdout.write(JSON.stringify(main())+'\n'); }
catch(e) {
  process.stderr.write((e.stderr?String(e.stderr):e.message)+'\n');
  if(request.id) { try { process.stdout.write(JSON.stringify(snapshot(repository(request.id)))+'\n'); } catch(_) {} }
  process.exitCode=1;
}
